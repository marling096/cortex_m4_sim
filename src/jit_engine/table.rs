use std::fmt;
use std::sync::Arc;

use crate::arch::{ArmCC, ArmInsn};
use crate::cpu::Cpu;
use capstone::{Capstone, Insn};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::jit_engine::clif::instructions::{self, InsDef};
use crate::opcodes::decoded::{
    DecodedArmOperands, DecodedInstruction, DecodedMemOperand, DecodedOperand,
    DecodedOperandKind, DecodedShift, jit_execute_cycles,
    normalize_for_jit,
};
use crate::opcodes::opcode::ArmOpcode;

#[derive(Debug)]
pub enum JitTableBuildError {
    MissingOpcodeDefinition { insn_id: u32 },
    DecodeFailed { address: u64 },
}

impl fmt::Display for JitTableBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOpcodeDefinition { insn_id } => {
                write!(f, "missing opcode definition for instruction id {insn_id}")
            }
            Self::DecodeFailed { address } => {
                write!(f, "failed to decode instruction at address 0x{address:08X}")
            }
        }
    }
}

impl std::error::Error for JitTableBuildError {}

#[derive(Clone)]
pub struct JitInstruction {
    pub insn_id: u32,
    pub execute_cycles: u32,
    pub data: DecodedInstruction,
    pub def: Option<&'static dyn InsDef>,
}

impl JitInstruction {
    pub fn new(insn_id: u32, data: DecodedInstruction, execute_cycles: u32) -> Self {
        Self {
            insn_id,
            execute_cycles,
            data,
            def: None,
        }
    }

    fn is_it_instruction(&self) -> bool {
        self.insn_id == ArmInsn::ARM_INS_IT as u32
    }

    fn it_following_count(&self) -> usize {
        if self.is_it_instruction() {
            self.data.it_following_count() as usize
        } else {
            0
        }
    }

    fn is_branch_instruction(&self) -> bool {
        matches!(
            self.insn_id,
            x if x == ArmInsn::ARM_INS_B as u32
                || x == ArmInsn::ARM_INS_BL as u32
                || x == ArmInsn::ARM_INS_BX as u32
                || x == ArmInsn::ARM_INS_BLX as u32
                || x == ArmInsn::ARM_INS_CBZ as u32
                || x == ArmInsn::ARM_INS_CBNZ as u32
        )
    }

    fn static_branch_target(&self) -> Option<u32> {
        if !matches!(
            self.insn_id,
            x if x == ArmInsn::ARM_INS_B as u32
                || x == ArmInsn::ARM_INS_BL as u32
                || x == ArmInsn::ARM_INS_CBZ as u32
                || x == ArmInsn::ARM_INS_CBNZ as u32
        ) {
            return None;
        }

        match self.data.arm_operands.op2.as_ref().map(|op| &op.op_type) {
            Some(DecodedOperandKind::Imm(imm)) => Some(*imm as u32),
            _ => None,
        }
    }

    fn has_exception_return_path(&self) -> bool {
        if self.insn_id == ArmInsn::ARM_INS_BX as u32 {
            return true;
        }

        self.insn_id == ArmInsn::ARM_INS_POP as u32 && self.data.transed_operands.contains(&15)
    }

    fn modifies_pc(&self) -> bool {
        if self.is_branch_instruction() || self.has_exception_return_path() {
            return true;
        }

        if self.data.writes_pc() {
            return true;
        }

        if self.data.arm_operands.rd == 15 {
            return true;
        }

        matches!(
            self.insn_id,
            x if x == ArmInsn::ARM_INS_LDM as u32 || x == ArmInsn::ARM_INS_POP as u32
        ) && self.data.transed_operands.contains(&15)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JitBlockTerminator {
    Branch,
    BranchTarget,
    PcWrite,
    ItBlockEnd,
    ExceptionReturn,
    Gap,
    EndOfTable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JitBlockRange {
    pub start_pc: u32,
    pub end_pc: u32,
    pub instruction_count: usize,
    pub terminator: JitBlockTerminator,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JitBlockStats {
    pub block_count: usize,
    pub total_instruction_count: usize,
    pub branch_blocks: usize,
    pub branch_target_blocks: usize,
    pub pc_write_blocks: usize,
    pub it_block_end_blocks: usize,
    pub exception_return_blocks: usize,
    pub gap_blocks: usize,
    pub end_of_table_blocks: usize,
}

impl JitBlockStats {
    pub fn average_block_len(&self) -> f64 {
        if self.block_count == 0 {
            0.0
        } else {
            self.total_instruction_count as f64 / self.block_count as f64
        }
    }
}

pub struct JitBlockBuilder;

impl JitBlockBuilder {
    pub fn build(table: &JitBlockTable) -> Vec<JitBlockRange> {
        let mut entries: Vec<_> = table.iter_entries().collect();
        entries.sort_unstable_by_key(|(pc, _)| *pc);

        if entries.is_empty() {
            return Vec::new();
        }

        let branch_targets: FxHashSet<u32> = entries
            .iter()
            .filter_map(|(_, ins)| ins.static_branch_target())
            .filter(|target| table.get(*target).is_some())
            .collect();

        let mut blocks = Vec::new();
        let mut block_start = entries[0].0;
        let mut instruction_count = 0usize;
        let mut pending_it_following = 0usize;
        let mut previous_pc = entries[0].0;

        for (index, (pc, ins)) in entries.iter().enumerate() {
            if instruction_count > 0 && *pc != block_start && branch_targets.contains(pc) {
                blocks.push(JitBlockRange {
                    start_pc: block_start,
                    end_pc: previous_pc,
                    instruction_count,
                    terminator: JitBlockTerminator::BranchTarget,
                });
                block_start = *pc;
                instruction_count = 0;
            }

            instruction_count += 1;

            let was_inside_it_block = pending_it_following > 0;
            if was_inside_it_block {
                pending_it_following -= 1;
            }

            if ins.is_it_instruction() {
                pending_it_following = ins.it_following_count();
            }

            let it_block_ends_here = was_inside_it_block && pending_it_following == 0;
            let next_pc = entries.get(index + 1).map(|(next_pc, _)| *next_pc);
            let expected_next_pc = pc.wrapping_add(ins.data.size());

            let terminator = if ins.has_exception_return_path() {
                Some(JitBlockTerminator::ExceptionReturn)
            } else if ins.is_branch_instruction() {
                Some(JitBlockTerminator::Branch)
            } else if ins.modifies_pc() {
                Some(JitBlockTerminator::PcWrite)
            } else if it_block_ends_here {
                Some(JitBlockTerminator::ItBlockEnd)
            } else {
                match next_pc {
                    Some(next_pc) if next_pc != expected_next_pc => Some(JitBlockTerminator::Gap),
                    Some(_) => None,
                    None => Some(JitBlockTerminator::EndOfTable),
                }
            };

            if let Some(terminator) = terminator {
                blocks.push(JitBlockRange {
                    start_pc: block_start,
                    end_pc: *pc,
                    instruction_count,
                    terminator,
                });

                if let Some((next_block_start, _)) = entries.get(index + 1) {
                    block_start = *next_block_start;
                }
                instruction_count = 0;
                pending_it_following = 0;
            }

            previous_pc = *pc;
        }

        blocks
    }
}

pub struct JitBlockTable {
    entries: FxHashMap<u32, Arc<JitInstruction>>,
    fast_table: Vec<Option<Arc<JitInstruction>>>,
    fast_base: u32,
    blocks: Vec<JitBlockRange>,
    block_starts: FxHashMap<u32, usize>,
    block_membership: FxHashMap<u32, usize>,
}

impl JitBlockTable {
    pub fn len(&self) -> usize {
        self.entries.len() + self.fast_table.iter().filter(|entry| entry.is_some()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.fast_table.is_empty()
    }

    pub fn blocks(&self) -> &[JitBlockRange] {
        &self.blocks
    }

    pub fn iter_blocks(&self) -> impl Iterator<Item = &JitBlockRange> {
        self.blocks.iter()
    }

    pub fn block_starting_at(&self, pc: u32) -> Option<&JitBlockRange> {
        self.block_starts
            .get(&pc)
            .and_then(|index| self.blocks.get(*index))
    }

    pub fn block_containing(&self, pc: u32) -> Option<&JitBlockRange> {
        self.block_membership
            .get(&pc)
            .and_then(|index| self.blocks.get(*index))
    }

    pub fn block_stats(&self) -> JitBlockStats {
        let mut stats = JitBlockStats::default();
        for block in &self.blocks {
            stats.block_count += 1;
            stats.total_instruction_count += block.instruction_count;
            match block.terminator {
                JitBlockTerminator::Branch => stats.branch_blocks += 1,
                JitBlockTerminator::BranchTarget => stats.branch_target_blocks += 1,
                JitBlockTerminator::PcWrite => stats.pc_write_blocks += 1,
                JitBlockTerminator::ItBlockEnd => stats.it_block_end_blocks += 1,
                JitBlockTerminator::ExceptionReturn => stats.exception_return_blocks += 1,
                JitBlockTerminator::Gap => stats.gap_blocks += 1,
                JitBlockTerminator::EndOfTable => stats.end_of_table_blocks += 1,
            }
        }
        stats
    }

    #[inline(always)]
    pub fn get(&self, pc: u32) -> Option<&JitInstruction> {
        let offset = (pc.wrapping_sub(self.fast_base)) >> 1;
        if (offset as usize) < self.fast_table.len() {
            unsafe {
                if let Some(instr) = self.fast_table.get_unchecked(offset as usize) {
                    return Some(instr.as_ref());
                }
            }
        }

        if self.entries.is_empty() {
            None
        } else {
            self.entries.get(&pc).map(AsRef::as_ref)
        }
    }

    pub fn iter_entries(&self) -> impl Iterator<Item = (u32, &JitInstruction)> + '_ {
        let fast_base = self.fast_base;
        let fast_iter = self.fast_table.iter().enumerate().filter_map(move |(index, entry)| {
            entry
                .as_ref()
                .map(|instr| (fast_base.wrapping_add((index as u32) << 1), instr.as_ref()))
        });

        fast_iter.chain(self.entries.iter().map(|(pc, ins)| (*pc, ins.as_ref())))
    }
}

#[derive(Default)]
pub struct JitBlockTableBuilder {
    entries: FxHashMap<u32, Arc<JitInstruction>>,
}

impl JitBlockTableBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_instruction(&mut self, mut instr: JitInstruction) {
        instr.def = instructions::find_def(instr.insn_id).filter(|def| def.supports(&instr));
        self.entries.insert(instr.data.address(), Arc::new(instr));
    }

    pub fn add_disassembled_instruction(
        &mut self,
        cs: &Capstone,
        insn: &Insn<'_>,
    ) -> Result<(), JitTableBuildError> {
        let instr = decode_jit_instruction(cs, insn)?;
        self.add_instruction(instr);
        Ok(())
    }

    pub fn extend_from_pc(
        &mut self,
        cpu: &Cpu,
        start_pc: u32,
        end_pc_exclusive: u32,
        max_instructions: usize,
    ) -> Result<usize, JitTableBuildError> {
        let mut current_pc = start_pc;
        let mut added = 0usize;
        let mut pending_it_following = 0usize;

        while added < max_instructions {
            if current_pc >= end_pc_exclusive {
                break;
            }
            if self.entries.contains_key(&current_pc) {
                break;
            }

            let jit_instr = decode_jit_instruction_from_thumb(cpu, current_pc)?;
            let size = jit_instr.data.size();

            let was_inside_it_block = pending_it_following > 0;
            if was_inside_it_block {
                pending_it_following -= 1;
            }
            if jit_instr.is_it_instruction() {
                pending_it_following = jit_instr.it_following_count();
            }

            let terminates_block = jit_instr.has_exception_return_path()
                || jit_instr.is_branch_instruction()
                || jit_instr.modifies_pc()
                || (was_inside_it_block && pending_it_following == 0);

            self.add_instruction(jit_instr);
            added += 1;

            if terminates_block {
                break;
            }

            current_pc = current_pc.wrapping_add(size);
        }

        Ok(added)
    }

    pub fn extend_disassembly<'a, I>(
        &mut self,
        cs: &Capstone,
        insns: I,
    ) -> Result<(), JitTableBuildError>
    where
        I: IntoIterator<Item = &'a Insn<'a>>,
    {
        for insn in insns {
            self.add_disassembled_instruction(cs, insn)?;
        }
        Ok(())
    }

    pub fn build(self) -> JitBlockTable {
        let mut table = optimize_entries(self.entries);
        rebuild_block_metadata(&mut table);
        table
    }

    pub fn build_snapshot(&self) -> JitBlockTable {
        let mut table = optimize_entries(self.entries.clone());
        rebuild_block_metadata(&mut table);
        table
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn build_from_disassembly<'a, I>(
        cs: &Capstone,
        insns: I,
    ) -> Result<JitBlockTable, JitTableBuildError>
    where
        I: IntoIterator<Item = &'a Insn<'a>>,
    {
        let mut builder = Self::new();
        builder.extend_disassembly(cs, insns)?;
        Ok(builder.build())
    }
}

fn build_decoded_only_instruction(
    insn_id: u32,
    arm_opcode: &ArmOpcode<'_>,
) -> Option<JitInstruction> {
    let data = normalize_for_jit(arm_opcode)?;
    let execute_cycles = jit_execute_cycles(insn_id, &data)?;
    let mut instr = JitInstruction::new(insn_id, data, execute_cycles);
    instr.def = instructions::find_def(insn_id).filter(|def| def.supports(&instr));
    instr.def?;
    Some(instr)
}

fn build_fallback_instruction(
    insn_id: u32,
    arm_opcode: &ArmOpcode<'_>,
) -> JitInstruction {
    let data = normalize_for_jit(arm_opcode).unwrap_or_else(|| DecodedInstruction::from_arm_opcode(arm_opcode));
    let execute_cycles = jit_execute_cycles(insn_id, &data).unwrap_or(1);
    JitInstruction::new(insn_id, data, execute_cycles)
}

fn decode_jit_instruction(
    cs: &Capstone,
    insn: &Insn<'_>,
) -> Result<JitInstruction, JitTableBuildError> {
    let insn_id = ArmInsn::from_raw(insn.id().0)
        .map(|id| id as u32)
        .ok_or(JitTableBuildError::MissingOpcodeDefinition { insn_id: insn.id().0 })?;
    let arm_opcode = ArmOpcode::new(cs, insn).ok_or(JitTableBuildError::DecodeFailed {
        address: insn.address(),
    })?;

    Ok(build_decoded_only_instruction(insn_id, &arm_opcode)
        .unwrap_or_else(|| build_fallback_instruction(insn_id, &arm_opcode)))
}

fn decode_jit_instruction_from_thumb(
    cpu: &Cpu,
    current_pc: u32,
) -> Result<JitInstruction, JitTableBuildError> {
    let hw1 = fetch16(cpu, current_pc);
    if is_thumb32(hw1) {
        let hw2 = fetch16(cpu, current_pc.wrapping_add(2));
        decode_jit_instruction_32(current_pc, hw1, hw2)
    } else {
        decode_jit_instruction_16(current_pc, hw1)
    }
}

fn decode_jit_instruction_16(current_pc: u32, hw: u16) -> Result<JitInstruction, JitTableBuildError> {
    if (hw & 0xF800) == 0xE000 {
        let imm11 = (hw & 0x07FF) as u32;
        let offset = sign_extend((imm11 << 1) as u32, 12);
        let target = current_pc.wrapping_add(4).wrapping_add_signed(offset);
        return Ok(make_branch_instruction(ArmInsn::ARM_INS_B, current_pc, 2, ArmCC::ARM_CC_AL, target, None));
    }

    if (hw & 0xF000) == 0xD000 && (hw & 0x0F00) != 0x0F00 {
        let cond = ArmCC::from_u8(((hw >> 8) & 0xF) as u8);
        let imm8 = (hw & 0xFF) as u32;
        let offset = sign_extend((imm8 << 1) as u32, 9);
        let target = current_pc.wrapping_add(4).wrapping_add_signed(offset);
        return Ok(make_branch_instruction(ArmInsn::ARM_INS_B, current_pc, 2, cond, target, None));
    }

    if (hw & 0xF500) == 0xB100 {
        let nonzero = (hw & 0x0800) != 0;
        let rn = (hw & 0x7) as u32;
        let i = ((hw >> 9) & 0x1) as u32;
        let imm5 = ((hw >> 3) & 0x1F) as u32;
        let offset = (i << 6) | (imm5 << 1);
        let target = current_pc.wrapping_add(4).wrapping_add(offset);
        return Ok(make_compare_branch_instruction(
            if nonzero { ArmInsn::ARM_INS_CBNZ } else { ArmInsn::ARM_INS_CBZ },
            current_pc,
            2,
            rn,
            target,
        ));
    }

    if (hw & 0xF800) == 0x4800 {
        let rt = ((hw >> 8) & 0x7) as u32;
        let imm = ((hw & 0xFF) as u32) << 2;
        return Ok(make_load_instruction(
            ArmInsn::ARM_INS_LDR,
            current_pc,
            2,
            rt,
            15,
            imm as i32,
            false,
            false,
            0,
            rt == 15,
        ));
    }

    if (hw & 0xF800) == 0x6800 {
        let imm5 = ((hw >> 6) & 0x1F) as u32;
        let rn = ((hw >> 3) & 0x7) as u32;
        let rt = (hw & 0x7) as u32;
        return Ok(make_load_instruction(
            ArmInsn::ARM_INS_LDR,
            current_pc,
            2,
            rt,
            rn,
            (imm5 << 2) as i32,
            false,
            false,
            0,
            rt == 15,
        ));
    }

    if (hw & 0xF800) == 0x6000 {
        let imm5 = ((hw >> 6) & 0x1F) as u32;
        let rn = ((hw >> 3) & 0x7) as u32;
        let rt = (hw & 0x7) as u32;
        return Ok(make_store_instruction(ArmInsn::ARM_INS_STR, current_pc, 2, rt, rn, (imm5 << 2) as i32, false));
    }

    if (hw & 0xF800) == 0x7000 {
        let imm5 = ((hw >> 6) & 0x1F) as u32;
        let rn = ((hw >> 3) & 0x7) as u32;
        let rt = (hw & 0x7) as u32;
        return Ok(make_store_instruction(ArmInsn::ARM_INS_STRB, current_pc, 2, rt, rn, imm5 as i32, false));
    }

    if (hw & 0xF800) == 0x7800 {
        let imm5 = ((hw >> 6) & 0x1F) as u32;
        let rn = ((hw >> 3) & 0x7) as u32;
        let rt = (hw & 0x7) as u32;
        return Ok(make_load_instruction(ArmInsn::ARM_INS_LDRB, current_pc, 2, rt, rn, imm5 as i32, false, false, 0, false));
    }

    if (hw & 0xF800) == 0x8000 {
        let imm5 = ((hw >> 6) & 0x1F) as u32;
        let rn = ((hw >> 3) & 0x7) as u32;
        let rt = (hw & 0x7) as u32;
        return Ok(make_store_instruction(ArmInsn::ARM_INS_STRH, current_pc, 2, rt, rn, (imm5 << 1) as i32, false));
    }

    if (hw & 0xF800) == 0x8800 {
        let imm5 = ((hw >> 6) & 0x1F) as u32;
        let rn = ((hw >> 3) & 0x7) as u32;
        let rt = (hw & 0x7) as u32;
        return Ok(make_load_instruction(ArmInsn::ARM_INS_LDRH, current_pc, 2, rt, rn, (imm5 << 1) as i32, false, false, 0, false));
    }

    if (hw & 0xF200) == 0x5000 {
        let op = (hw >> 9) & 0x7;
        let rm = ((hw >> 6) & 0x7) as u32;
        let rn = ((hw >> 3) & 0x7) as u32;
        let rt = (hw & 0x7) as u32;
        return Ok(match op {
            0 => make_store_reg_offset_instruction(ArmInsn::ARM_INS_STR, current_pc, 2, rt, rn, rm),
            1 => make_store_reg_offset_instruction(ArmInsn::ARM_INS_STRH, current_pc, 2, rt, rn, rm),
            2 => make_store_reg_offset_instruction(ArmInsn::ARM_INS_STRB, current_pc, 2, rt, rn, rm),
            3 => make_load_reg_offset_instruction(ArmInsn::ARM_INS_LDRSB, current_pc, 2, rt, rn, rm),
            4 => make_load_reg_offset_instruction(ArmInsn::ARM_INS_LDR, current_pc, 2, rt, rn, rm),
            5 => make_load_reg_offset_instruction(ArmInsn::ARM_INS_LDRH, current_pc, 2, rt, rn, rm),
            6 => make_load_reg_offset_instruction(ArmInsn::ARM_INS_LDRB, current_pc, 2, rt, rn, rm),
            7 => make_load_reg_offset_instruction(ArmInsn::ARM_INS_LDRSH, current_pc, 2, rt, rn, rm),
            _ => unreachable!(),
        });
    }

    if (hw & 0xF000) == 0x9000 {
        let load = (hw & 0x0800) != 0;
        let rt = ((hw >> 8) & 0x7) as u32;
        let imm = ((hw & 0xFF) as u32) << 2;
        return Ok(if load {
            make_load_instruction(ArmInsn::ARM_INS_LDR, current_pc, 2, rt, 13, imm as i32, false, false, 0, rt == 15)
        } else {
            make_store_instruction(ArmInsn::ARM_INS_STR, current_pc, 2, rt, 13, imm as i32, false)
        });
    }

    if (hw & 0xFF00) == 0xB000 {
        let subtract = (hw & 0x0080) != 0;
        let imm = ((hw & 0x7F) as u32) << 2;
        let op2 = DecodedOperand {
            op_type: DecodedOperandKind::Imm(imm as i64),
            shift: DecodedShift::Invalid,
        };
        return Ok(make_data_instruction(
            if subtract { ArmInsn::ARM_INS_SUB } else { ArmInsn::ARM_INS_ADD },
            current_pc,
            2,
            if subtract { "sub" } else { "add" },
            vec![reg_op(13), op2.clone()],
            false,
            false,
            0,
            false,
            Vec::new(),
            DecodedArmOperands {
                condition: ArmCC::ARM_CC_AL,
                rd: 13,
                rn: 13,
                op2: Some(op2),
                mem_disp: 0,
                mem_has_index: false,
                mem_writeback: false,
                mem_post_index: false,
                mem_post_imm: 0,
            },
        ));
    }

    if (hw & 0xF800) == 0xA000 {
        let rd = ((hw >> 8) & 0x7) as u32;
        let imm = ((hw & 0xFF) as u32) << 2;
        let target = (current_pc.wrapping_add(4) & !0x3).wrapping_add(imm);
        return Ok(make_adr_instruction(current_pc, 2, rd, target));
    }

    if (hw & 0xF800) == 0xA800 {
        let rd = ((hw >> 8) & 0x7) as u32;
        let imm = ((hw & 0xFF) as u32) << 2;
        let op2 = DecodedOperand {
            op_type: DecodedOperandKind::Imm(imm as i64),
            shift: DecodedShift::Invalid,
        };
        return Ok(make_data_instruction(
            ArmInsn::ARM_INS_ADD,
            current_pc,
            2,
            "add",
            vec![reg_op(rd), reg_op(13), op2.clone()],
            false,
            false,
            0,
            rd == 15,
            Vec::new(),
            DecodedArmOperands {
                condition: ArmCC::ARM_CC_AL,
                rd,
                rn: 13,
                op2: Some(op2),
                mem_disp: 0,
                mem_has_index: false,
                mem_writeback: false,
                mem_post_index: false,
                mem_post_imm: 0,
            },
        ));
    }

    if (hw & 0xFE00) == 0xB400 {
        let extra_lr = ((hw >> 8) & 1) != 0;
        let mut regs = low_reg_list(hw & 0xFF);
        if extra_lr {
            regs.push(14);
        }
        return Ok(make_multi_reg_instruction(ArmInsn::ARM_INS_PUSH, current_pc, 2, regs, false));
    }

    if (hw & 0xFE00) == 0xBC00 {
        let extra_pc = ((hw >> 8) & 1) != 0;
        let mut regs = low_reg_list(hw & 0xFF);
        if extra_pc {
            regs.push(15);
        }
        return Ok(make_multi_reg_instruction(ArmInsn::ARM_INS_POP, current_pc, 2, regs, false));
    }

    if (hw & 0xF000) == 0xC000 {
        let load = (hw & 0x0800) != 0;
        let rn = ((hw >> 8) & 0x7) as u32;
        let mut regs = vec![rn];
        regs.extend(low_reg_list(hw & 0xFF));
        return Ok(make_multi_reg_instruction(
            if load { ArmInsn::ARM_INS_LDM } else { ArmInsn::ARM_INS_STM },
            current_pc,
            2,
            regs,
            false,
        ));
    }

    if (hw & 0xFC00) == 0x4400 {
        let op = (hw >> 8) & 0x3;
        let h1 = ((hw >> 7) & 0x1) as u32;
        let h2 = ((hw >> 6) & 0x1) as u32;
        let rm = (((h2 << 3) | ((hw >> 3) & 0x7) as u32) & 0xF) as u32;
        let rdn = (((h1 << 3) | (hw & 0x7) as u32) & 0xF) as u32;
        return Ok(match op {
            0 => make_reg_binary_instruction(ArmInsn::ARM_INS_ADD, current_pc, 2, "add", rdn, rdn, rm, false, rdn == 15),
            1 => make_reg_binary_instruction(ArmInsn::ARM_INS_CMP, current_pc, 2, "cmp", 0, rdn, rm, true, false),
            2 => make_move_reg_instruction(ArmInsn::ARM_INS_MOV, current_pc, 2, rdn, rm, false, rdn == 15),
            3 => make_branch_instruction(
                if (hw & 0x0080) != 0 { ArmInsn::ARM_INS_BLX } else { ArmInsn::ARM_INS_BX },
                current_pc,
                2,
                ArmCC::ARM_CC_AL,
                0,
                Some(rm),
            ),
            _ => unreachable!(),
        });
    }

    if (hw & 0xFC00) == 0x4000 {
        let op = (hw >> 6) & 0xF;
        let rm = ((hw >> 3) & 0x7) as u32;
        let rdn = (hw & 0x7) as u32;
        return Ok(match op {
            0 => make_reg_binary_instruction(ArmInsn::ARM_INS_AND, current_pc, 2, "ands", rdn, rdn, rm, true, false),
            2 => make_shift_reg_instruction(ArmInsn::ARM_INS_LSL, current_pc, 2, rdn, rdn, rm, true),
            3 => make_shift_reg_instruction(ArmInsn::ARM_INS_LSR, current_pc, 2, rdn, rdn, rm, true),
            4 => make_shift_reg_instruction(ArmInsn::ARM_INS_ASR, current_pc, 2, rdn, rdn, rm, true),
            5 => make_reg_binary_instruction(ArmInsn::ARM_INS_ADC, current_pc, 2, "adcs", rdn, rdn, rm, true, false),
            6 => make_reg_binary_instruction(ArmInsn::ARM_INS_SBC, current_pc, 2, "sbcs", rdn, rdn, rm, true, false),
            7 => make_shift_reg_instruction(ArmInsn::ARM_INS_ROR, current_pc, 2, rdn, rdn, rm, true),
            8 => make_reg_binary_instruction(ArmInsn::ARM_INS_TST, current_pc, 2, "tst", 0, rdn, rm, true, false),
            9 => make_reg_binary_instruction(ArmInsn::ARM_INS_RSB, current_pc, 2, "rsbs", rdn, 0, rm, true, false),
            10 => make_reg_binary_instruction(ArmInsn::ARM_INS_CMP, current_pc, 2, "cmp", 0, rdn, rm, true, false),
            11 => make_reg_binary_instruction(ArmInsn::ARM_INS_CMN, current_pc, 2, "cmn", 0, rdn, rm, true, false),
            12 => make_reg_binary_instruction(ArmInsn::ARM_INS_ORR, current_pc, 2, "orrs", rdn, rdn, rm, true, false),
            13 => make_reg_binary_instruction(ArmInsn::ARM_INS_MUL, current_pc, 2, "muls", rdn, rdn, rm, true, false),
            14 => make_reg_binary_instruction(ArmInsn::ARM_INS_BIC, current_pc, 2, "bics", rdn, rdn, rm, true, false),
            15 => make_move_reg_instruction(ArmInsn::ARM_INS_MVN, current_pc, 2, rdn, rm, true, false),
            _ => make_unknown_instruction(current_pc, 2),
        });
    }

    if (hw & 0xE000) == 0x0000 {
        let op = (hw >> 11) & 0x3;
        let imm5 = ((hw >> 6) & 0x1F) as u32;
        let rm = ((hw >> 3) & 0x7) as u32;
        let rd = (hw & 0x7) as u32;
        return Ok(match op {
            0 => make_shift_imm_instruction(ArmInsn::ARM_INS_LSL, current_pc, 2, rd, rm, imm5, true),
            1 => make_shift_imm_instruction(ArmInsn::ARM_INS_LSR, current_pc, 2, rd, rm, if imm5 == 0 { 32 } else { imm5 }, true),
            2 => make_shift_imm_instruction(ArmInsn::ARM_INS_ASR, current_pc, 2, rd, rm, if imm5 == 0 { 32 } else { imm5 }, true),
            _ => make_unknown_instruction(current_pc, 2),
        });
    }

    if (hw & 0xFC00) == 0x1C00 {
        let op = (hw >> 9) & 0x1;
        let imm3 = ((hw >> 6) & 0x7) as u32;
        let rn = ((hw >> 3) & 0x7) as u32;
        let rd = (hw & 0x7) as u32;
        return Ok(make_imm_binary_instruction(
            if op == 0 { ArmInsn::ARM_INS_ADD } else { ArmInsn::ARM_INS_SUB },
            current_pc,
            2,
            if op == 0 { "adds" } else { "subs" },
            rd,
            rn,
            imm3,
            true,
            rd == 15,
        ));
    }

    if (hw & 0xF800) == 0x2000 {
        let op = (hw >> 11) & 0x3;
        let rdn = ((hw >> 8) & 0x7) as u32;
        let imm8 = (hw & 0xFF) as u32;
        return Ok(match op {
            0 => make_move_imm_instruction(ArmInsn::ARM_INS_MOVS, current_pc, 2, rdn, imm8, true, rdn == 15),
            1 => make_imm_binary_instruction(ArmInsn::ARM_INS_CMP, current_pc, 2, "cmp", 0, rdn, imm8, true, false),
            2 => make_imm_binary_instruction(ArmInsn::ARM_INS_ADD, current_pc, 2, "adds", rdn, rdn, imm8, true, rdn == 15),
            3 => make_imm_binary_instruction(ArmInsn::ARM_INS_SUB, current_pc, 2, "subs", rdn, rdn, imm8, true, rdn == 15),
            _ => make_unknown_instruction(current_pc, 2),
        });
    }

    if hw == 0xBF00 {
        return Ok(make_misc_instruction(ArmInsn::ARM_INS_NOP, current_pc, 2, "nop", 0, false));
    }

    if (hw & 0xFF00) == 0xBF00 && (hw & 0x000F) != 0 {
        return Ok(make_misc_instruction(ArmInsn::ARM_INS_IT, current_pc, 2, "it", (hw & 0xF) as u8, false));
    }

    if (hw & 0xFF00) == 0xBF00 {
        return Ok(make_misc_instruction(ArmInsn::ARM_INS_HINT, current_pc, 2, "hint", 0, false));
    }

    if (hw & 0xFF00) == 0xBE00 {
        return Ok(make_misc_instruction(ArmInsn::ARM_INS_BKPT, current_pc, 2, "bkpt", 0, true));
    }

    Ok(make_unknown_instruction(current_pc, 2))
}

fn decode_jit_instruction_32(current_pc: u32, hw1: u16, hw2: u16) -> Result<JitInstruction, JitTableBuildError> {
    if (hw1 & 0xF800) == 0xF000 && (hw2 & 0xD000) == 0xD000 {
        let s = ((hw1 >> 10) & 1) as u32;
        let j1 = ((hw2 >> 13) & 1) as u32;
        let j2 = ((hw2 >> 11) & 1) as u32;
        let i1 = (!(j1 ^ s)) & 1;
        let i2 = (!(j2 ^ s)) & 1;
        let imm10 = (hw1 & 0x03FF) as u32;
        let imm11 = (hw2 & 0x07FF) as u32;
        let imm25 = (s << 24) | (i1 << 23) | (i2 << 22) | (imm10 << 12) | (imm11 << 1);
        let offset = sign_extend(imm25, 25);
        let target = current_pc.wrapping_add(4).wrapping_add_signed(offset);
        return Ok(make_branch_instruction(ArmInsn::ARM_INS_BL, current_pc, 4, ArmCC::ARM_CC_AL, target, None));
    }

    if hw1 == 0xF8DF {
        let rt = ((hw2 >> 12) & 0xF) as u32;
        let imm12 = (hw2 & 0x0FFF) as u32;
        return Ok(make_load_instruction(ArmInsn::ARM_INS_LDR, current_pc, 4, rt, 15, imm12 as i32, false, false, 0, rt == 15));
    }

    if (hw1 & 0xFFF0) == 0xF890 {
        let rn = (hw1 & 0xF) as u32;
        let rt = ((hw2 >> 12) & 0xF) as u32;
        let imm12 = (hw2 & 0x0FFF) as u32;
        return Ok(make_load_instruction(ArmInsn::ARM_INS_LDRB, current_pc, 4, rt, rn, imm12 as i32, false, false, 0, false));
    }

    if (hw1 & 0xFFF0) == 0xF8B0 {
        let rn = (hw1 & 0xF) as u32;
        let rt = ((hw2 >> 12) & 0xF) as u32;
        let imm12 = (hw2 & 0x0FFF) as u32;
        return Ok(make_load_instruction(ArmInsn::ARM_INS_LDRH, current_pc, 4, rt, rn, imm12 as i32, false, false, 0, false));
    }

    if (hw1 & 0xFFF0) == 0xF8D0 {
        let rn = (hw1 & 0xF) as u32;
        let rt = ((hw2 >> 12) & 0xF) as u32;
        let imm12 = (hw2 & 0x0FFF) as u32;
        return Ok(make_load_instruction(ArmInsn::ARM_INS_LDR, current_pc, 4, rt, rn, imm12 as i32, false, false, 0, rt == 15));
    }

    if (hw1 & 0xFFF0) == 0xF8C0 {
        let rn = (hw1 & 0xF) as u32;
        let rt = ((hw2 >> 12) & 0xF) as u32;
        let imm12 = (hw2 & 0x0FFF) as u32;
        return Ok(make_store_instruction(ArmInsn::ARM_INS_STR, current_pc, 4, rt, rn, imm12 as i32, false));
    }

    if (hw1 & 0xFFF0) == 0xF880 {
        let rn = (hw1 & 0xF) as u32;
        let rt = ((hw2 >> 12) & 0xF) as u32;
        let imm12 = (hw2 & 0x0FFF) as u32;
        return Ok(make_store_instruction(ArmInsn::ARM_INS_STRB, current_pc, 4, rt, rn, imm12 as i32, false));
    }

    if (hw1 & 0xFFF0) == 0xF8A0 {
        let rn = (hw1 & 0xF) as u32;
        let rt = ((hw2 >> 12) & 0xF) as u32;
        let imm12 = (hw2 & 0x0FFF) as u32;
        return Ok(make_store_instruction(ArmInsn::ARM_INS_STRH, current_pc, 4, rt, rn, imm12 as i32, false));
    }

    if (hw1 & 0xFBF0) == 0xF240 && (hw2 & 0x8000) == 0 {
        let rd = ((hw2 >> 8) & 0xF) as u32;
        let imm4 = (hw1 & 0xF) as u32;
        let i = ((hw1 >> 10) & 1) as u32;
        let imm3 = ((hw2 >> 12) & 0x7) as u32;
        let imm8 = (hw2 & 0xFF) as u32;
        let imm16 = (imm4 << 12) | (i << 11) | (imm3 << 8) | imm8;
        return Ok(make_move_imm_instruction(ArmInsn::ARM_INS_MOV, current_pc, 4, rd, imm16, false, rd == 15));
    }

    if (hw1 & 0xFBF0) == 0xF2C0 && (hw2 & 0x8000) == 0 {
        let rd = ((hw2 >> 8) & 0xF) as u32;
        let imm4 = (hw1 & 0xF) as u32;
        let i = ((hw1 >> 10) & 1) as u32;
        let imm3 = ((hw2 >> 12) & 0x7) as u32;
        let imm8 = (hw2 & 0xFF) as u32;
        let imm16 = (imm4 << 12) | (i << 11) | (imm3 << 8) | imm8;
        return Ok(make_move_imm_instruction(ArmInsn::ARM_INS_MOV, current_pc, 4, rd, imm16 << 16, false, rd == 15));
    }

    if (hw1 & 0xFFF0) == 0xFBB0 && (hw2 & 0x00F0) == 0x00F0 {
        let rn = (hw1 & 0xF) as u32;
        let rd = ((hw2 >> 8) & 0xF) as u32;
        let rm = (hw2 & 0xF) as u32;
        return Ok(make_three_reg_instruction(ArmInsn::ARM_INS_UDIV, current_pc, 4, "udiv", rd, rn, rm, rd == 15));
    }

    if (hw1 & 0xFFF0) == 0xFB00 && (hw2 & 0x00F0) == 0x0010 {
        let rn = (hw1 & 0xF) as u32;
        let ra = ((hw2 >> 12) & 0xF) as u32;
        let rd = ((hw2 >> 8) & 0xF) as u32;
        let rm = (hw2 & 0xF) as u32;
        return Ok(make_four_reg_instruction(ArmInsn::ARM_INS_MLS, current_pc, 4, "mls", rd, rn, rm, ra, rd == 15));
    }

    if (hw1 & 0xFFF0) == 0xF3C0 {
        let rn = (hw1 & 0xF) as u32;
        let rd = ((hw2 >> 8) & 0xF) as u32;
        let lsb = ((((hw2 >> 12) & 0x7) as u32) << 2) | (((hw2 >> 6) & 0x3) as u32);
        let width = ((hw2 & 0x1F) as u32) + 1;
        return Ok(make_ubfx_instruction(current_pc, 4, rd, rn, lsb, width, rd == 15));
    }

    if (hw1 & 0xFA00) == 0xF000 && (hw2 & 0x8000) == 0 {
        let op = (hw1 >> 5) & 0xF;
        let rn = (hw1 & 0xF) as u32;
        let rd = ((hw2 >> 8) & 0xF) as u32;
        let setflags = ((hw1 >> 4) & 0x1) != 0;
        let imm12 = (((hw1 >> 10) & 1) as u16) << 11 | ((hw2 >> 12) & 0x7) << 8 | (hw2 & 0xFF);
        let imm = thumb_expand_imm12(imm12) as u32;
        return Ok(match op {
            0 => make_imm_binary_instruction(ArmInsn::ARM_INS_AND, current_pc, 4, if setflags { "ands" } else { "and" }, rd, rn, imm, setflags, rd == 15),
            1 => make_imm_binary_instruction(ArmInsn::ARM_INS_BIC, current_pc, 4, if setflags { "bics" } else { "bic" }, rd, rn, imm, setflags, rd == 15),
            2 if rn == 15 => make_move_imm_instruction(if setflags { ArmInsn::ARM_INS_MOVS } else { ArmInsn::ARM_INS_MOV }, current_pc, 4, rd, imm, setflags, rd == 15),
            2 => make_imm_binary_instruction(ArmInsn::ARM_INS_ORR, current_pc, 4, if setflags { "orrs" } else { "orr" }, rd, rn, imm, setflags, rd == 15),
            8 => make_imm_binary_instruction(ArmInsn::ARM_INS_ADD, current_pc, 4, if setflags { "adds" } else { "add" }, rd, rn, imm, setflags, rd == 15),
            13 => make_imm_binary_instruction(ArmInsn::ARM_INS_CMP, current_pc, 4, "cmp", 0, rn, imm, true, false),
            _ => make_unknown_instruction(current_pc, 4),
        });
    }

    if (hw1 & 0xFE00) == 0xEA00 || (hw1 & 0xFE00) == 0xEB00 {
        let op = (hw1 >> 5) & 0xF;
        let rn = (hw1 & 0xF) as u32;
        let rd = ((hw2 >> 8) & 0xF) as u32;
        let rm = (hw2 & 0xF) as u32;
        let imm3 = ((hw2 >> 12) & 0x7) as u32;
        let imm2 = ((hw2 >> 6) & 0x3) as u32;
        let shift_type = ((hw2 >> 4) & 0x3) as u32;
        let shift_amount = (imm3 << 2) | imm2;
        let shift = match shift_type {
            0 => DecodedShift::Lsl(shift_amount),
            1 => DecodedShift::Lsr(shift_amount),
            2 => DecodedShift::Asr(shift_amount),
            3 => DecodedShift::Ror(shift_amount),
            _ => DecodedShift::Invalid,
        };
        return Ok(match op {
            0 => make_reg_binary_shift_instruction(ArmInsn::ARM_INS_AND, current_pc, 4, "and", rd, rn, rm, shift, false, rd == 15),
            2 => make_reg_binary_shift_instruction(ArmInsn::ARM_INS_ORR, current_pc, 4, "orr", rd, rn, rm, shift, false, rd == 15),
            8 => make_reg_binary_shift_instruction(ArmInsn::ARM_INS_ADD, current_pc, 4, "add", rd, rn, rm, shift, false, rd == 15),
            _ => make_unknown_instruction(current_pc, 4),
        });
    }

    if (hw1 & 0xFFF0) == 0xFA00 {
        let rm = (hw1 & 0xF) as u32;
        let rd = ((hw2 >> 8) & 0xF) as u32;
        let rs = (hw2 & 0xF) as u32;
        return Ok(make_shift_reg_instruction(ArmInsn::ARM_INS_LSL, current_pc, 4, rd, rm, rs, false));
    }

    if hw1 == 0xE92D {
        let regs = full_reg_list(multi_reg_mask_push(hw2));
        return Ok(make_multi_reg_instruction(ArmInsn::ARM_INS_PUSH, current_pc, 4, regs, false));
    }

    if hw1 == 0xE8BD {
        let regs = full_reg_list(multi_reg_mask_pop(hw2));
        return Ok(make_multi_reg_instruction(ArmInsn::ARM_INS_POP, current_pc, 4, regs, false));
    }

    if (hw1 & 0xFFD0) == 0xE890 {
        let rn = (hw1 & 0xF) as u32;
        let writeback = ((hw1 >> 5) & 1) != 0;
        let mut regs = vec![rn];
        regs.extend(full_reg_list(hw2));
        return Ok(make_multi_reg_instruction(ArmInsn::ARM_INS_LDM, current_pc, 4, regs, writeback));
    }

    Ok(make_unknown_instruction(current_pc, 4))
}

fn make_data_instruction(
    insn: ArmInsn,
    address: u32,
    size: u32,
    mnemonic: &str,
    operands: Vec<DecodedOperand>,
    writeback: bool,
    update_flags: bool,
    it_mask: u8,
    writes_pc: bool,
    transed_operands: Vec<u32>,
    arm_operands: DecodedArmOperands,
) -> JitInstruction {
    let data = DecodedInstruction::from_parts(
        address,
        size,
        mnemonic,
        String::new(),
        operands,
        writeback,
        update_flags,
        it_mask,
        writes_pc,
        transed_operands,
        arm_operands,
    );
    let execute_cycles = jit_execute_cycles(insn as u32, &data).unwrap_or(1);
    let mut instr = JitInstruction::new(insn as u32, data, execute_cycles);
    instr.def = instructions::find_def(insn as u32).filter(|def| def.supports(&instr));
    instr
}

fn make_unknown_instruction(address: u32, size: u32) -> JitInstruction {
    let data = DecodedInstruction::from_parts(
        address,
        size,
        "unknown",
        String::new(),
        Vec::new(),
        false,
        false,
        0,
        false,
        Vec::new(),
        empty_arm_operands(),
    );
    JitInstruction::new(ArmInsn::ARM_INS_INVALID as u32, data, 1)
}

fn make_misc_instruction(insn: ArmInsn, address: u32, size: u32, mnemonic: &str, it_mask: u8, writes_pc: bool) -> JitInstruction {
    make_data_instruction(insn, address, size, mnemonic, Vec::new(), false, false, it_mask, writes_pc, Vec::new(), empty_arm_operands())
}

fn make_branch_instruction(
    insn: ArmInsn,
    address: u32,
    size: u32,
    condition: ArmCC,
    target: u32,
    reg_target: Option<u32>,
) -> JitInstruction {
    let op2 = match reg_target {
        Some(reg) => reg_op(reg),
        None => imm_op(target as i64),
    };
    let arm_operands = DecodedArmOperands {
        condition,
        rd: 0,
        rn: 0,
        op2: Some(op2.clone()),
        mem_disp: 0,
        mem_has_index: false,
        mem_writeback: false,
        mem_post_index: false,
        mem_post_imm: 0,
    };
    make_data_instruction(insn, address, size, match insn {
        ArmInsn::ARM_INS_BL => "bl",
        ArmInsn::ARM_INS_BLX => "blx",
        ArmInsn::ARM_INS_BX => "bx",
        _ => "b",
    }, vec![op2], false, false, 0, true, Vec::new(), arm_operands)
}

fn make_compare_branch_instruction(insn: ArmInsn, address: u32, size: u32, rn: u32, target: u32) -> JitInstruction {
    let op2 = imm_op(target as i64);
    make_data_instruction(
        insn,
        address,
        size,
        if insn == ArmInsn::ARM_INS_CBNZ { "cbnz" } else { "cbz" },
        vec![reg_op(rn), op2.clone()],
        false,
        false,
        0,
        false,
        Vec::new(),
        DecodedArmOperands {
            condition: ArmCC::ARM_CC_AL,
            rd: 0,
            rn,
            op2: Some(op2),
            mem_disp: 0,
            mem_has_index: false,
            mem_writeback: false,
            mem_post_index: false,
            mem_post_imm: 0,
        },
    )
}

fn make_load_instruction(
    insn: ArmInsn,
    address: u32,
    size: u32,
    rt: u32,
    base: u32,
    disp: i32,
    has_index: bool,
    writeback: bool,
    post_imm: i32,
    writes_pc: bool,
) -> JitInstruction {
    let mem = mem_op(base, None, disp);
    make_data_instruction(
        insn,
        address,
        size,
        match insn {
            ArmInsn::ARM_INS_LDRB => "ldrb",
            ArmInsn::ARM_INS_LDRH => "ldrh",
            ArmInsn::ARM_INS_LDRSB => "ldrsb",
            ArmInsn::ARM_INS_LDRSH => "ldrsh",
            _ => "ldr",
        },
        vec![reg_op(rt), mem.clone()],
        writeback,
        false,
        0,
        writes_pc,
        Vec::new(),
        DecodedArmOperands {
            condition: ArmCC::ARM_CC_AL,
            rd: rt,
            rn: base,
            op2: None,
            mem_disp: disp,
            mem_has_index: has_index,
            mem_writeback: writeback,
            mem_post_index: false,
            mem_post_imm: post_imm,
        },
    )
}

fn make_store_instruction(insn: ArmInsn, address: u32, size: u32, rt: u32, base: u32, disp: i32, writeback: bool) -> JitInstruction {
    let mem = mem_op(base, None, disp);
    make_data_instruction(
        insn,
        address,
        size,
        match insn {
            ArmInsn::ARM_INS_STRB => "strb",
            ArmInsn::ARM_INS_STRH => "strh",
            _ => "str",
        },
        vec![reg_op(rt), mem.clone()],
        writeback,
        false,
        0,
        false,
        Vec::new(),
        DecodedArmOperands {
            condition: ArmCC::ARM_CC_AL,
            rd: rt,
            rn: base,
            op2: None,
            mem_disp: disp,
            mem_has_index: false,
            mem_writeback: writeback,
            mem_post_index: false,
            mem_post_imm: 0,
        },
    )
}

fn make_load_reg_offset_instruction(insn: ArmInsn, address: u32, size: u32, rt: u32, base: u32, index: u32) -> JitInstruction {
    let mem = mem_op(base, Some(index), 0);
    make_data_instruction(
        insn,
        address,
        size,
        match insn {
            ArmInsn::ARM_INS_LDRB => "ldrb",
            ArmInsn::ARM_INS_LDRH => "ldrh",
            ArmInsn::ARM_INS_LDRSB => "ldrsb",
            ArmInsn::ARM_INS_LDRSH => "ldrsh",
            _ => "ldr",
        },
        vec![reg_op(rt), mem.clone()],
        false,
        false,
        0,
        rt == 15 && insn == ArmInsn::ARM_INS_LDR,
        Vec::new(),
        DecodedArmOperands {
            condition: ArmCC::ARM_CC_AL,
            rd: rt,
            rn: base,
            op2: None,
            mem_disp: 0,
            mem_has_index: true,
            mem_writeback: false,
            mem_post_index: false,
            mem_post_imm: 0,
        },
    )
}

fn make_store_reg_offset_instruction(insn: ArmInsn, address: u32, size: u32, rt: u32, base: u32, index: u32) -> JitInstruction {
    let mem = mem_op(base, Some(index), 0);
    make_data_instruction(
        insn,
        address,
        size,
        match insn {
            ArmInsn::ARM_INS_STRB => "strb",
            ArmInsn::ARM_INS_STRH => "strh",
            _ => "str",
        },
        vec![reg_op(rt), mem.clone()],
        false,
        false,
        0,
        false,
        Vec::new(),
        DecodedArmOperands {
            condition: ArmCC::ARM_CC_AL,
            rd: rt,
            rn: base,
            op2: None,
            mem_disp: 0,
            mem_has_index: true,
            mem_writeback: false,
            mem_post_index: false,
            mem_post_imm: 0,
        },
    )
}

fn make_imm_binary_instruction(insn: ArmInsn, address: u32, size: u32, mnemonic: &str, rd: u32, rn: u32, imm: u32, update_flags: bool, writes_pc: bool) -> JitInstruction {
    let op2 = imm_op(imm as i64);
    let mut operands = Vec::new();
    if rd != 0 || !matches!(insn, ArmInsn::ARM_INS_CMP | ArmInsn::ARM_INS_CMN | ArmInsn::ARM_INS_TST | ArmInsn::ARM_INS_TEQ) {
        operands.push(reg_op(rd));
    }
    if rn != 0 || matches!(insn, ArmInsn::ARM_INS_CMP | ArmInsn::ARM_INS_CMN | ArmInsn::ARM_INS_TST | ArmInsn::ARM_INS_TEQ | ArmInsn::ARM_INS_ADD | ArmInsn::ARM_INS_SUB | ArmInsn::ARM_INS_AND | ArmInsn::ARM_INS_BIC | ArmInsn::ARM_INS_ORR) {
        operands.push(reg_op(rn));
    }
    operands.push(op2.clone());
    make_data_instruction(
        insn,
        address,
        size,
        mnemonic,
        operands,
        false,
        update_flags,
        0,
        writes_pc,
        Vec::new(),
        DecodedArmOperands {
            condition: ArmCC::ARM_CC_AL,
            rd,
            rn,
            op2: Some(op2),
            mem_disp: 0,
            mem_has_index: false,
            mem_writeback: false,
            mem_post_index: false,
            mem_post_imm: 0,
        },
    )
}

fn make_reg_binary_instruction(insn: ArmInsn, address: u32, size: u32, mnemonic: &str, rd: u32, rn: u32, rm: u32, update_flags: bool, writes_pc: bool) -> JitInstruction {
    make_reg_binary_shift_instruction(insn, address, size, mnemonic, rd, rn, rm, DecodedShift::Invalid, update_flags, writes_pc)
}

fn make_reg_binary_shift_instruction(insn: ArmInsn, address: u32, size: u32, mnemonic: &str, rd: u32, rn: u32, rm: u32, shift: DecodedShift, update_flags: bool, writes_pc: bool) -> JitInstruction {
    let op2 = DecodedOperand {
        op_type: DecodedOperandKind::Reg(rm),
        shift,
    };
    let mut operands = Vec::new();
    if rd != 0 || !matches!(insn, ArmInsn::ARM_INS_CMP | ArmInsn::ARM_INS_CMN | ArmInsn::ARM_INS_TST | ArmInsn::ARM_INS_TEQ) {
        operands.push(reg_op(rd));
    }
    if rn != 0 || matches!(insn, ArmInsn::ARM_INS_CMP | ArmInsn::ARM_INS_CMN | ArmInsn::ARM_INS_TST | ArmInsn::ARM_INS_TEQ | ArmInsn::ARM_INS_ADD | ArmInsn::ARM_INS_ADC | ArmInsn::ARM_INS_SUB | ArmInsn::ARM_INS_SBC | ArmInsn::ARM_INS_RSB | ArmInsn::ARM_INS_AND | ArmInsn::ARM_INS_ORR | ArmInsn::ARM_INS_EOR | ArmInsn::ARM_INS_BIC | ArmInsn::ARM_INS_ORN | ArmInsn::ARM_INS_MUL) {
        operands.push(reg_op(rn));
    }
    operands.push(op2.clone());
    make_data_instruction(
        insn,
        address,
        size,
        mnemonic,
        operands,
        false,
        update_flags,
        0,
        writes_pc,
        Vec::new(),
        DecodedArmOperands {
            condition: ArmCC::ARM_CC_AL,
            rd,
            rn,
            op2: Some(op2),
            mem_disp: 0,
            mem_has_index: false,
            mem_writeback: false,
            mem_post_index: false,
            mem_post_imm: 0,
        },
    )
}

fn make_move_reg_instruction(insn: ArmInsn, address: u32, size: u32, rd: u32, rm: u32, update_flags: bool, writes_pc: bool) -> JitInstruction {
    make_data_instruction(
        insn,
        address,
        size,
        if insn == ArmInsn::ARM_INS_MVN { "mvn" } else { "mov" },
        vec![reg_op(rd), reg_op(rm)],
        false,
        update_flags,
        0,
        writes_pc,
        Vec::new(),
        DecodedArmOperands {
            condition: ArmCC::ARM_CC_AL,
            rd,
            rn: 0,
            op2: Some(reg_op(rm)),
            mem_disp: 0,
            mem_has_index: false,
            mem_writeback: false,
            mem_post_index: false,
            mem_post_imm: 0,
        },
    )
}

fn make_move_imm_instruction(insn: ArmInsn, address: u32, size: u32, rd: u32, imm: u32, update_flags: bool, writes_pc: bool) -> JitInstruction {
    let op2 = imm_op(imm as i64);
    make_data_instruction(
        insn,
        address,
        size,
        if insn == ArmInsn::ARM_INS_MOVS { "movs" } else { "mov" },
        vec![reg_op(rd), op2.clone()],
        false,
        update_flags,
        0,
        writes_pc,
        Vec::new(),
        DecodedArmOperands {
            condition: ArmCC::ARM_CC_AL,
            rd,
            rn: 0,
            op2: Some(op2),
            mem_disp: 0,
            mem_has_index: false,
            mem_writeback: false,
            mem_post_index: false,
            mem_post_imm: 0,
        },
    )
}

fn make_shift_imm_instruction(insn: ArmInsn, address: u32, size: u32, rd: u32, rm: u32, amount: u32, update_flags: bool) -> JitInstruction {
    let shift = match insn {
        ArmInsn::ARM_INS_LSL => DecodedShift::Lsl(amount),
        ArmInsn::ARM_INS_LSR => DecodedShift::Lsr(amount),
        ArmInsn::ARM_INS_ASR => DecodedShift::Asr(amount),
        ArmInsn::ARM_INS_ROR => DecodedShift::Ror(amount),
        _ => DecodedShift::Invalid,
    };
    make_reg_binary_shift_instruction(insn, address, size, match insn {
        ArmInsn::ARM_INS_LSL => "lsl",
        ArmInsn::ARM_INS_LSR => "lsr",
        ArmInsn::ARM_INS_ASR => "asr",
        ArmInsn::ARM_INS_ROR => "ror",
        _ => "shift",
    }, rd, 0, rm, shift, update_flags, rd == 15)
}

fn make_shift_reg_instruction(insn: ArmInsn, address: u32, size: u32, rd: u32, rm: u32, rs: u32, update_flags: bool) -> JitInstruction {
    let op2 = reg_op(rs);
    make_data_instruction(
        insn,
        address,
        size,
        match insn {
            ArmInsn::ARM_INS_LSL => "lsl",
            ArmInsn::ARM_INS_LSR => "lsr",
            ArmInsn::ARM_INS_ASR => "asr",
            ArmInsn::ARM_INS_ROR => "ror",
            _ => "shift",
        },
        vec![reg_op(rd), reg_op(rm), op2.clone()],
        false,
        update_flags,
        0,
        rd == 15,
        Vec::new(),
        DecodedArmOperands {
            condition: ArmCC::ARM_CC_AL,
            rd,
            rn: rm,
            op2: Some(op2),
            mem_disp: 0,
            mem_has_index: false,
            mem_writeback: false,
            mem_post_index: false,
            mem_post_imm: 0,
        },
    )
}

fn make_adr_instruction(address: u32, size: u32, rd: u32, target: u32) -> JitInstruction {
    let op2 = imm_op(target as i64);
    make_data_instruction(
        ArmInsn::ARM_INS_ADR,
        address,
        size,
        "adr",
        vec![reg_op(rd), op2.clone()],
        false,
        false,
        0,
        rd == 15,
        Vec::new(),
        DecodedArmOperands {
            condition: ArmCC::ARM_CC_AL,
            rd,
            rn: 15,
            op2: Some(op2),
            mem_disp: 0,
            mem_has_index: false,
            mem_writeback: false,
            mem_post_index: false,
            mem_post_imm: 0,
        },
    )
}

fn make_three_reg_instruction(insn: ArmInsn, address: u32, size: u32, mnemonic: &str, rd: u32, rn: u32, rm: u32, writes_pc: bool) -> JitInstruction {
    make_data_instruction(
        insn,
        address,
        size,
        mnemonic,
        vec![reg_op(rd), reg_op(rn), reg_op(rm)],
        false,
        false,
        0,
        writes_pc,
        Vec::new(),
        DecodedArmOperands {
            condition: ArmCC::ARM_CC_AL,
            rd,
            rn,
            op2: Some(reg_op(rm)),
            mem_disp: 0,
            mem_has_index: false,
            mem_writeback: false,
            mem_post_index: false,
            mem_post_imm: 0,
        },
    )
}

fn make_four_reg_instruction(insn: ArmInsn, address: u32, size: u32, mnemonic: &str, rd: u32, rn: u32, rm: u32, ra: u32, writes_pc: bool) -> JitInstruction {
    make_data_instruction(
        insn,
        address,
        size,
        mnemonic,
        vec![reg_op(rd), reg_op(rn), reg_op(rm), reg_op(ra)],
        false,
        false,
        0,
        writes_pc,
        Vec::new(),
        DecodedArmOperands {
            condition: ArmCC::ARM_CC_AL,
            rd,
            rn,
            op2: Some(reg_op(rm)),
            mem_disp: 0,
            mem_has_index: false,
            mem_writeback: false,
            mem_post_index: false,
            mem_post_imm: 0,
        },
    )
}

fn make_ubfx_instruction(address: u32, size: u32, rd: u32, rn: u32, lsb: u32, width: u32, writes_pc: bool) -> JitInstruction {
    make_data_instruction(
        ArmInsn::ARM_INS_UBFX,
        address,
        size,
        "ubfx",
        vec![reg_op(rd), reg_op(rn), imm_op(lsb as i64), imm_op(width as i64)],
        false,
        false,
        0,
        writes_pc,
        Vec::new(),
        DecodedArmOperands {
            condition: ArmCC::ARM_CC_AL,
            rd,
            rn,
            op2: None,
            mem_disp: 0,
            mem_has_index: false,
            mem_writeback: false,
            mem_post_index: false,
            mem_post_imm: 0,
        },
    )
}

fn make_multi_reg_instruction(insn: ArmInsn, address: u32, size: u32, regs: Vec<u32>, writeback: bool) -> JitInstruction {
    let base_reg = if matches!(insn, ArmInsn::ARM_INS_LDM | ArmInsn::ARM_INS_STM) {
        regs.first().copied().unwrap_or(0)
    } else {
        13
    };
    let transed_operands = regs.clone();
    let operands = regs.iter().copied().map(reg_op).collect::<Vec<_>>();
    make_data_instruction(
        insn,
        address,
        size,
        match insn {
            ArmInsn::ARM_INS_PUSH => "push",
            ArmInsn::ARM_INS_POP => "pop",
            ArmInsn::ARM_INS_STM => "stm",
            _ => "ldm",
        },
        operands,
        writeback,
        false,
        0,
        regs.contains(&15),
        transed_operands,
        DecodedArmOperands {
            condition: ArmCC::ARM_CC_AL,
            rd: 0,
            rn: base_reg,
            op2: None,
            mem_disp: 0,
            mem_has_index: false,
            mem_writeback: writeback,
            mem_post_index: false,
            mem_post_imm: 0,
        },
    )
}

fn reg_op(reg: u32) -> DecodedOperand {
    DecodedOperand {
        op_type: DecodedOperandKind::Reg(reg),
        shift: DecodedShift::Invalid,
    }
}

fn imm_op(imm: i64) -> DecodedOperand {
    DecodedOperand {
        op_type: DecodedOperandKind::Imm(imm),
        shift: DecodedShift::Invalid,
    }
}

fn mem_op(base: u32, index: Option<u32>, disp: i32) -> DecodedOperand {
    DecodedOperand {
        op_type: DecodedOperandKind::Mem(DecodedMemOperand { base, index, disp }),
        shift: DecodedShift::Invalid,
    }
}

fn empty_arm_operands() -> DecodedArmOperands {
    DecodedArmOperands {
        condition: ArmCC::ARM_CC_AL,
        rd: 0,
        rn: 0,
        op2: None,
        mem_disp: 0,
        mem_has_index: false,
        mem_writeback: false,
        mem_post_index: false,
        mem_post_imm: 0,
    }
}

fn fetch16(cpu: &Cpu, addr: u32) -> u16 {
    let bytes = cpu.fetch_thumb_bytes(addr);
    u16::from_le_bytes([bytes[0], bytes[1]])
}

fn is_thumb32(hw1: u16) -> bool {
    matches!(hw1 >> 11, 0b11101 | 0b11110 | 0b11111)
}

fn sign_extend(value: u32, bits: u32) -> i32 {
    let shift = 32 - bits;
    ((value << shift) as i32) >> shift
}

fn low_reg_list(mask: u16) -> Vec<u32> {
    full_reg_list(mask)
}

fn full_reg_list(mask: u16) -> Vec<u32> {
    (0..16).filter(|bit| (mask & (1 << bit)) != 0).map(|bit| bit as u32).collect()
}

fn multi_reg_mask_push(hw2: u16) -> u16 {
    let mut mask = hw2 & 0x1FFF;
    if (hw2 & 0x4000) != 0 {
        mask |= 1 << 14;
    }
    mask
}

fn multi_reg_mask_pop(hw2: u16) -> u16 {
    let mut mask = hw2 & 0x1FFF;
    if (hw2 & 0x8000) != 0 {
        mask |= 1 << 15;
    }
    mask
}

fn thumb_expand_imm12(imm12: u16) -> u32 {
    let imm8 = (imm12 & 0x00FF) as u32;
    let imm3 = ((imm12 >> 8) & 0x7) as u32;
    let i = ((imm12 >> 11) & 0x1) as u32;
    let top = (imm12 >> 10) & 0x3;
    match top {
        0 => match imm3 {
            0 => imm8,
            1 => (imm8 << 16) | imm8,
            2 => (imm8 << 24) | (imm8 << 8),
            3 => (imm8 << 24) | (imm8 << 16) | (imm8 << 8) | imm8,
            _ => unreachable!(),
        },
        _ => {
            let unrotated = (1u32 << 7) | imm8;
            let rot = (i << 4) | (imm3 << 1);
            unrotated.rotate_right(rot)
        }
    }
}

fn rebuild_block_metadata(table: &mut JitBlockTable) {
    let blocks = JitBlockBuilder::build(table);
    let block_starts = blocks
        .iter()
        .enumerate()
        .map(|(index, block)| (block.start_pc, index))
        .collect();
    let mut block_membership = FxHashMap::default();
    for (index, block) in blocks.iter().enumerate() {
        let mut current_pc = block.start_pc;
        loop {
            block_membership.insert(current_pc, index);
            if current_pc == block.end_pc {
                break;
            }
            let Some(ins) = table.get(current_pc) else {
                break;
            };
            current_pc = current_pc.wrapping_add(ins.data.size());
        }
    }
    table.blocks = blocks;
    table.block_starts = block_starts;
    table.block_membership = block_membership;
}

#[cfg(test)]
mod tests {
    use super::*;
    use capstone::arch;
    use capstone::prelude::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicU32;

    use crate::cpu::Cpu;
    use crate::peripheral::bus::Bus;

    fn build_thumb_capstone() -> Capstone {
        Capstone::new()
            .arm()
            .mode(arch::arm::ArchMode::Thumb)
            .extra_mode([arch::arm::ArchExtraMode::MClass].iter().copied())
            .detail(true)
            .build()
            .expect("failed to create capstone")
    }

    fn build_cpu() -> Cpu {
        Cpu::new(Arc::new(AtomicU32::new(8_000_000)), 1, Bus::new(), Bus::new())
    }

    #[test]
    fn jit_instruction_table_builder_builds_entries() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x08, 0x68, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");

        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
        .expect("failed to build jit instruction table");

        assert_eq!(table.len(), 2);
        assert!(table.get(0x0800_0000).is_some());
        assert!(table.get(0x0800_0002).is_some());
        assert_eq!(table.blocks().len(), 1);
        assert_eq!(table.blocks()[0].start_pc, 0x0800_0000);
        assert_eq!(table.blocks()[0].end_pc, 0x0800_0002);
        assert_eq!(table.blocks()[0].terminator, JitBlockTerminator::EndOfTable);
        assert!(table.block_starting_at(0x0800_0000).is_some());
    }

    #[test]
    fn jit_instruction_table_keeps_fallback_data_only_for_unsupported_entries() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let arm_opcode = ArmOpcode::new(&cs, insns.iter().next().expect("missing instruction"))
            .expect("failed to decode arm opcode");
        let decoded = DecodedInstruction::from_arm_opcode(&arm_opcode);

        let mut builder = JitBlockTableBuilder::new();
        builder.add_instruction(JitInstruction::new(u32::MAX, decoded, 1));
        let table = builder.build();

        let instr = table.get(0x0800_0000).expect("missing instruction");
        assert!(instr.def.is_none());
    }

    #[test]
    fn jit_instruction_table_builder_extends_from_runtime_pc() {
        let mut cpu = build_cpu();
        cpu.load_code_bytes(0x0800_0000, &[0x08, 0x68, 0x00, 0xBF]);

        let mut builder = JitBlockTableBuilder::new();
        let added = builder
            .extend_from_pc(&cpu, 0x0800_0000, 0x0800_0004, 16)
            .expect("failed to extend jit table from runtime pc");
        let table = builder.build_snapshot();

        assert_eq!(added, 2);
        assert!(table.get(0x0800_0000).is_some());
        assert!(table.get(0x0800_0002).is_some());
    }

    #[test]
    fn jit_instruction_addresses_stay_stable_across_snapshots() {
        let mut cpu = build_cpu();
        cpu.load_code_bytes(0x0800_0000, &[0x08, 0x68, 0x00, 0xBF, 0x01, 0x20, 0x00, 0xBF]);

        let mut builder = JitBlockTableBuilder::new();
        builder
            .extend_from_pc(&cpu, 0x0800_0000, 0x0800_0004, 16)
            .expect("failed to build initial snapshot");
        let first = builder.build_snapshot();
        let before = first
            .get(0x0800_0000)
            .expect("missing first instruction") as *const JitInstruction;

        builder
            .extend_from_pc(&cpu, 0x0800_0004, 0x0800_0008, 16)
            .expect("failed to extend snapshot");
        let second = builder.build_snapshot();
        let after = second
            .get(0x0800_0000)
            .expect("missing first instruction after extend") as *const JitInstruction;

        assert_eq!(before, after);
    }

    #[test]
    fn jit_block_builder_starts_new_block_at_branch_target() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x00, 0xE0, 0x00, 0xBF, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
        .expect("failed to build jit instruction table");

        assert_eq!(table.blocks().len(), 3);
        assert_eq!(table.blocks()[0].start_pc, 0x0800_0000);
        assert_eq!(table.blocks()[0].end_pc, 0x0800_0000);
        assert_eq!(table.blocks()[0].terminator, JitBlockTerminator::Branch);
        assert_eq!(table.blocks()[1].start_pc, 0x0800_0002);
        assert_eq!(table.blocks()[1].end_pc, 0x0800_0002);
        assert_eq!(table.blocks()[1].terminator, JitBlockTerminator::BranchTarget);
        assert_eq!(table.blocks()[2].start_pc, 0x0800_0004);
        assert_eq!(table.blocks()[2].end_pc, 0x0800_0004);
        assert_eq!(table.blocks()[2].terminator, JitBlockTerminator::EndOfTable);
        assert!(table.block_starting_at(0x0800_0004).is_some());
    }

    #[test]
    fn jit_block_builder_splits_on_branch_instruction() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0xFE, 0xE7, 0x00, 0xBF, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
        .expect("failed to build jit instruction table");

        assert_eq!(table.blocks().len(), 2);
        assert_eq!(table.blocks()[0].start_pc, 0x0800_0000);
        assert_eq!(table.blocks()[0].end_pc, 0x0800_0000);
        assert_eq!(table.blocks()[0].instruction_count, 1);
        assert_eq!(table.blocks()[0].terminator, JitBlockTerminator::Branch);
        assert_eq!(table.blocks()[1].start_pc, 0x0800_0002);
        assert_eq!(table.blocks()[1].end_pc, 0x0800_0004);
        assert_eq!(table.blocks()[1].instruction_count, 2);
        assert_eq!(table.blocks()[1].terminator, JitBlockTerminator::EndOfTable);
    }

    #[test]
    fn jit_block_builder_splits_on_pc_write_instruction() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x87, 0x46, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
        .expect("failed to build jit instruction table");

        assert_eq!(table.blocks().len(), 2);
        assert_eq!(table.blocks()[0].start_pc, 0x0800_0000);
        assert_eq!(table.blocks()[0].end_pc, 0x0800_0000);
        assert_eq!(table.blocks()[0].terminator, JitBlockTerminator::PcWrite);
        assert_eq!(table.blocks()[1].start_pc, 0x0800_0002);
        assert_eq!(table.blocks()[1].end_pc, 0x0800_0002);
        assert_eq!(table.blocks()[1].terminator, JitBlockTerminator::EndOfTable);
    }

    #[test]
    fn jit_block_builder_splits_at_it_block_end() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x18, 0xBF, 0xFB, 0x1A, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
        .expect("failed to build jit instruction table");

        assert_eq!(table.blocks().len(), 2);
        assert_eq!(table.blocks()[0].start_pc, 0x0800_0000);
        assert_eq!(table.blocks()[0].end_pc, 0x0800_0002);
        assert_eq!(table.blocks()[0].instruction_count, 2);
        assert_eq!(table.blocks()[0].terminator, JitBlockTerminator::ItBlockEnd);
        assert_eq!(table.blocks()[1].start_pc, 0x0800_0004);
        assert_eq!(table.blocks()[1].end_pc, 0x0800_0004);
        assert_eq!(table.blocks()[1].terminator, JitBlockTerminator::EndOfTable);
    }

    #[test]
    fn jit_block_builder_splits_on_exception_return_path() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x10, 0xBD, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
        .expect("failed to build jit instruction table");

        assert_eq!(table.blocks().len(), 2);
        assert_eq!(table.blocks()[0].start_pc, 0x0800_0000);
        assert_eq!(table.blocks()[0].end_pc, 0x0800_0000);
        assert_eq!(table.blocks()[0].terminator, JitBlockTerminator::ExceptionReturn);
        assert_eq!(table.blocks()[1].start_pc, 0x0800_0002);
        assert_eq!(table.blocks()[1].end_pc, 0x0800_0002);
        assert_eq!(table.blocks()[1].terminator, JitBlockTerminator::EndOfTable);
    }
}

fn optimize_entries(mut entries: FxHashMap<u32, Arc<JitInstruction>>) -> JitBlockTable {
    if entries.is_empty() {
        return JitBlockTable {
            entries,
            fast_table: Vec::new(),
            fast_base: 0,
            blocks: Vec::new(),
            block_starts: FxHashMap::default(),
            block_membership: FxHashMap::default(),
        };
    }

    let mut min_addr = u32::MAX;
    let mut max_addr = 0;
    for addr in entries.keys() {
        if *addr < min_addr {
            min_addr = *addr;
        }
        if *addr > max_addr {
            max_addr = *addr;
        }
    }

    let range = max_addr - min_addr;
    if range > 16 * 1024 * 1024 {
        return JitBlockTable {
            entries,
            fast_table: Vec::new(),
            fast_base: 0,
            blocks: Vec::new(),
            block_starts: FxHashMap::default(),
            block_membership: FxHashMap::default(),
        };
    }

    let size = (range / 2) as usize + 2;
    let mut fast_table = Vec::with_capacity(size);
    for _ in 0..size {
        fast_table.push(None);
    }

    let fast_base = min_addr;
    let mut keys: Vec<u32> = entries.keys().copied().collect();
    keys.sort_unstable();

    for addr in keys {
        if let Some(instr) = entries.remove(&addr) {
            let offset = ((addr - fast_base) >> 1) as usize;
            if offset < fast_table.len() {
                fast_table[offset] = Some(instr);
            } else {
                entries.insert(addr, instr);
            }
        }
    }

    JitBlockTable {
        entries,
        fast_table,
        fast_base,
        blocks: Vec::new(),
        block_starts: FxHashMap::default(),
        block_membership: FxHashMap::default(),
    }
}