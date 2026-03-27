use std::fmt;

use crate::arch::ArmInsn;
use capstone::{Capstone, Insn};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::jit_engine::clif::instructions::{self, InsDef};
use crate::opcodes::decoded::{
    DecodedInstruction, DecodedOperandKind, jit_execute_cycles,
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
    entries: FxHashMap<u32, JitInstruction>,
    fast_table: Vec<Option<JitInstruction>>,
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
                    return Some(instr);
                }
            }
        }

        if self.entries.is_empty() {
            None
        } else {
            self.entries.get(&pc)
        }
    }

    pub fn iter_entries(&self) -> impl Iterator<Item = (u32, &JitInstruction)> + '_ {
        let fast_base = self.fast_base;
        let fast_iter = self.fast_table.iter().enumerate().filter_map(move |(index, entry)| {
            entry
                .as_ref()
                .map(|instr| (fast_base.wrapping_add((index as u32) << 1), instr))
        });

        fast_iter.chain(self.entries.iter().map(|(pc, ins)| (*pc, ins)))
    }
}

#[derive(Default)]
pub struct JitBlockTableBuilder {
    entries: FxHashMap<u32, JitInstruction>,
}

impl JitBlockTableBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_instruction(&mut self, mut instr: JitInstruction) {
        instr.def = instructions::find_def(instr.insn_id).filter(|def| def.supports(&instr));
        self.entries.insert(instr.data.address(), instr);
    }

    pub fn add_disassembled_instruction(
        &mut self,
        cs: &Capstone,
        insn: &Insn<'_>,
    ) -> Result<(), JitTableBuildError> {
        let insn_id = ArmInsn::from_raw(insn.id().0)
            .map(|id| id as u32)
            .ok_or(JitTableBuildError::MissingOpcodeDefinition { insn_id: insn.id().0 })?;
        let arm_opcode = ArmOpcode::new(cs, insn).ok_or(JitTableBuildError::DecodeFailed {
            address: insn.address(),
        })?;

        if let Some(instr) = build_decoded_only_instruction(insn_id, &arm_opcode) {
            self.add_instruction(instr);
            return Ok(());
        }

        let instr = build_fallback_instruction(insn_id, &arm_opcode);
        self.add_instruction(instr);
        Ok(())
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
        let blocks = JitBlockBuilder::build(&table);
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
        table
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

#[cfg(test)]
mod tests {
    use super::*;
    use capstone::arch;
    use capstone::prelude::*;

    fn build_thumb_capstone() -> Capstone {
        Capstone::new()
            .arm()
            .mode(arch::arm::ArchMode::Thumb)
            .extra_mode([arch::arm::ArchExtraMode::MClass].iter().copied())
            .detail(true)
            .build()
            .expect("failed to create capstone")
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

fn optimize_entries(mut entries: FxHashMap<u32, JitInstruction>) -> JitBlockTable {
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