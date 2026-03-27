use crate::arch::{ArmCC, ArmInsn};
use crate::context::CpuContext;
use crate::opcodes::opcode::ArmOpcode;
use capstone::arch::arm::{ArmOperand, ArmOperandType, ArmReg, ArmShift};
use capstone::RegId;

#[derive(Clone, Debug)]
pub enum DecodedOperandKind {
    Imm(i64),
    Reg(u32),
    Mem(DecodedMemOperand),
    Invalid,
}

#[derive(Clone, Copy, Debug)]
pub enum DecodedShift {
    Invalid,
    Lsl(u32),
    Lsr(u32),
    Asr(u32),
    Ror(u32),
    Rrx(u32),
}

#[derive(Clone, Debug)]
pub struct DecodedOperand {
    pub op_type: DecodedOperandKind,
    pub shift: DecodedShift,
}

#[derive(Clone, Debug)]
pub struct DecodedMemOperand {
    pub base: u32,
    pub index: Option<u32>,
    pub disp: i32,
}

#[derive(Clone, Debug)]
pub struct DecodedArmOperands {
    pub condition: ArmCC,
    pub rd: u32,
    pub rn: u32,
    pub op2: Option<DecodedOperand>,
    pub mem_disp: i32,
    pub mem_has_index: bool,
    pub mem_writeback: bool,
    pub mem_post_index: bool,
    pub mem_post_imm: i32,
}

#[derive(Clone, Debug)]
pub struct DecodedInstruction {
    address: u32,
    size: u32,
    mnemonic: String,
    op_str: String,
    operands: Vec<DecodedOperand>,
    writeback: bool,
    update_flags: bool,
    it_mask: u8,
    writes_pc: bool,
    pub transed_operands: Vec<u32>,
    pub arm_operands: DecodedArmOperands,
}

#[derive(Clone, Debug)]
pub struct DecodedInstructionBuilder {
    address: u32,
    size: u32,
    mnemonic: String,
    op_str: String,
    operands: Vec<DecodedOperand>,
    writeback: bool,
    update_flags: bool,
    it_mask: u8,
    writes_pc: bool,
    pub transed_operands: Vec<u32>,
    pub arm_operands: DecodedArmOperands,
}

impl DecodedInstruction {
    pub fn from_parts(
        address: u32,
        size: u32,
        mnemonic: impl Into<String>,
        op_str: impl Into<String>,
        operands: Vec<DecodedOperand>,
        writeback: bool,
        update_flags: bool,
        it_mask: u8,
        writes_pc: bool,
        transed_operands: Vec<u32>,
        arm_operands: DecodedArmOperands,
    ) -> Self {
        Self {
            address,
            size,
            mnemonic: mnemonic.into(),
            op_str: op_str.into(),
            operands,
            writeback,
            update_flags,
            it_mask,
            writes_pc,
            transed_operands,
            arm_operands,
        }
    }

    pub fn from_arm_opcode(opcode: &ArmOpcode<'_>) -> Self {
        let operands: Vec<_> = opcode
            .operands()
            .map(|operand| decode_operand(opcode, &operand))
            .collect();

        Self {
            address: opcode.address(),
            size: opcode.size(),
            mnemonic: opcode.mnemonic().to_string(),
            op_str: opcode.op_str().to_string(),
            operands,
            writeback: opcode.writeback(),
            update_flags: opcode.update_flags(),
            it_mask: opcode.it_mask(),
            writes_pc: opcode.writes_pc(),
            transed_operands: opcode.transed_operands.clone(),
            arm_operands: DecodedArmOperands {
                condition: opcode.arm_operands.condition,
                rd: opcode.arm_operands.rd,
                rn: opcode.arm_operands.rn,
                op2: opcode.arm_operands.op2.as_ref().cloned(),
                mem_disp: opcode.arm_operands.mem_disp,
                mem_has_index: opcode.arm_operands.mem_has_index,
                mem_writeback: opcode.arm_operands.mem_writeback,
                mem_post_index: opcode.arm_operands.mem_post_index,
                mem_post_imm: opcode.arm_operands.mem_post_imm,
            },
        }
    }

    pub fn address(&self) -> u32 {
        self.address
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn get_operand(&self, index: usize) -> Option<&DecodedOperand> {
        self.operands.get(index)
    }

    pub fn mnemonic(&self) -> &str {
        &self.mnemonic
    }

    pub fn op_str(&self) -> &str {
        &self.op_str
    }

    pub fn writeback(&self) -> bool {
        self.writeback
    }

    pub fn update_flags(&self) -> bool {
        self.update_flags
    }

    pub fn it_following_count(&self) -> u8 {
        if self.it_mask == 0 {
            return 0;
        }

        (4u32.saturating_sub(self.it_mask.trailing_zeros())) as u8
    }

    pub fn writes_pc(&self) -> bool {
        self.writes_pc
    }
}

impl DecodedInstructionBuilder {
    pub fn from_arm_opcode(opcode: &ArmOpcode<'_>) -> Self {
        let operands: Vec<_> = opcode
            .operands()
            .map(|operand| decode_operand(opcode, &operand))
            .collect();

        Self {
            address: opcode.address(),
            size: opcode.size(),
            mnemonic: opcode.mnemonic().to_string(),
            op_str: opcode.op_str().to_string(),
            operands,
            writeback: opcode.writeback(),
            update_flags: opcode.update_flags(),
            it_mask: opcode.it_mask(),
            writes_pc: opcode.writes_pc(),
            transed_operands: opcode.transed_operands.clone(),
            arm_operands: DecodedArmOperands {
                condition: opcode.condition(),
                rd: 0,
                rn: 0,
                op2: None,
                mem_disp: 0,
                mem_has_index: false,
                mem_writeback: opcode.writeback(),
                mem_post_index: opcode.post_index(),
                mem_post_imm: 0,
            },
        }
    }

    pub fn get_operand(&self, index: usize) -> Option<&DecodedOperand> {
        self.operands.get(index)
    }

    pub fn build(self) -> DecodedInstruction {
        DecodedInstruction {
            address: self.address,
            size: self.size,
            mnemonic: self.mnemonic,
            op_str: self.op_str,
            operands: self.operands,
            writeback: self.writeback,
            update_flags: self.update_flags,
            it_mask: self.it_mask,
            writes_pc: self.writes_pc,
            transed_operands: self.transed_operands,
            arm_operands: self.arm_operands,
        }
    }
}

pub fn normalize_for_jit(opcode: &ArmOpcode<'_>) -> Option<DecodedInstruction> {
    let mut decoded = DecodedInstructionBuilder::from_arm_opcode(opcode);
    normalize_jit_builder(opcode, &mut decoded)?;
    Some(decoded.build())
}

pub fn jit_execute_cycles(insn_id: u32, instr: &DecodedInstruction) -> Option<u32> {
    match insn_id {
        x if x == ArmInsn::ARM_INS_LDR as u32
            || x == ArmInsn::ARM_INS_LDRB as u32
            || x == ArmInsn::ARM_INS_LDRSB as u32
            || x == ArmInsn::ARM_INS_LDRH as u32
            || x == ArmInsn::ARM_INS_LDRSH as u32
            || x == ArmInsn::ARM_INS_B as u32
            || x == ArmInsn::ARM_INS_BL as u32
            || x == ArmInsn::ARM_INS_BX as u32
            || x == ArmInsn::ARM_INS_BLX as u32 => Some(2),
        x if x == ArmInsn::ARM_INS_PUSH as u32 => {
            Some(1u32.saturating_add(instr.transed_operands.len() as u32))
        }
        x if x == ArmInsn::ARM_INS_POP as u32 => {
            let mut cycles = 1u32.saturating_add(instr.transed_operands.len() as u32);
            if instr.transed_operands.contains(&15) {
                cycles = cycles.saturating_add(1);
            }
            Some(cycles)
        }
        x if x == ArmInsn::ARM_INS_LDM as u32 => {
            let reg_count = (instr.transed_operands.len() as u32).saturating_sub(1);
            let mut cycles = 1u32.saturating_add(reg_count);
            if instr.transed_operands.iter().skip(1).any(|&reg| reg == 15) {
                cycles = cycles.saturating_add(1);
            }
            Some(cycles)
        }
        x if x == ArmInsn::ARM_INS_STM as u32 => {
            let reg_count = (instr.transed_operands.len() as u32).saturating_sub(1);
            Some(1u32.saturating_add(reg_count))
        }
        x if x == ArmInsn::ARM_INS_UBFX as u32
            || x == ArmInsn::ARM_INS_UXTB as u32
            || x == ArmInsn::ARM_INS_UXTH as u32
            || x == ArmInsn::ARM_INS_CMP as u32
            || x == ArmInsn::ARM_INS_CMN as u32
            || x == ArmInsn::ARM_INS_TST as u32
            || x == ArmInsn::ARM_INS_TEQ as u32
            || x == ArmInsn::ARM_INS_MOV as u32
            || x == ArmInsn::ARM_INS_MVN as u32
            || x == ArmInsn::ARM_INS_MOVS as u32
            || x == ArmInsn::ARM_INS_AND as u32
            || x == ArmInsn::ARM_INS_ORR as u32
            || x == ArmInsn::ARM_INS_EOR as u32
            || x == ArmInsn::ARM_INS_BIC as u32
            || x == ArmInsn::ARM_INS_ORN as u32
            || x == ArmInsn::ARM_INS_ADD as u32
            || x == ArmInsn::ARM_INS_ADC as u32
            || x == ArmInsn::ARM_INS_SUB as u32
            || x == ArmInsn::ARM_INS_SBC as u32
            || x == ArmInsn::ARM_INS_RSB as u32
            || x == ArmInsn::ARM_INS_MUL as u32
            || x == ArmInsn::ARM_INS_UDIV as u32
            || x == ArmInsn::ARM_INS_MLS as u32
            || x == ArmInsn::ARM_INS_ASR as u32
            || x == ArmInsn::ARM_INS_LSL as u32
            || x == ArmInsn::ARM_INS_LSR as u32
            || x == ArmInsn::ARM_INS_ROR as u32
            || x == ArmInsn::ARM_INS_RRX as u32
            || x == ArmInsn::ARM_INS_ADR as u32
            || x == ArmInsn::ARM_INS_CBZ as u32
            || x == ArmInsn::ARM_INS_CBNZ as u32
            || x == ArmInsn::ARM_INS_STR as u32
            || x == ArmInsn::ARM_INS_STRB as u32
            || x == ArmInsn::ARM_INS_STRH as u32
            || x == ArmInsn::ARM_INS_NOP as u32
            || x == ArmInsn::ARM_INS_HINT as u32
            || x == ArmInsn::ARM_INS_IT as u32
            || x == ArmInsn::ARM_INS_BKPT as u32 => Some(1),
        _ => None,
    }
}

fn normalize_jit_builder(opcode: &ArmOpcode<'_>, decoded: &mut DecodedInstructionBuilder) -> Option<()> {
    let insn = ArmInsn::from_raw(opcode.id())?;
    decoded.arm_operands.condition = opcode.condition();

    match insn {
        ArmInsn::ARM_INS_UBFX => normalize_ubfx(decoded),
        ArmInsn::ARM_INS_UXTB | ArmInsn::ARM_INS_UXTH => normalize_extend(decoded),
        ArmInsn::ARM_INS_CMP | ArmInsn::ARM_INS_CMN | ArmInsn::ARM_INS_TST | ArmInsn::ARM_INS_TEQ => {
            normalize_compare(decoded)
        }
        ArmInsn::ARM_INS_MOV | ArmInsn::ARM_INS_MVN | ArmInsn::ARM_INS_MOVS => normalize_move(decoded),
        ArmInsn::ARM_INS_AND
        | ArmInsn::ARM_INS_ORR
        | ArmInsn::ARM_INS_EOR
        | ArmInsn::ARM_INS_BIC
        | ArmInsn::ARM_INS_ORN => normalize_binary(decoded),
        ArmInsn::ARM_INS_ADD
        | ArmInsn::ARM_INS_ADC
        | ArmInsn::ARM_INS_SUB
        | ArmInsn::ARM_INS_SBC
        | ArmInsn::ARM_INS_RSB
        | ArmInsn::ARM_INS_MUL
        | ArmInsn::ARM_INS_UDIV
        | ArmInsn::ARM_INS_MLS => normalize_binary(decoded),
        ArmInsn::ARM_INS_ASR
        | ArmInsn::ARM_INS_LSL
        | ArmInsn::ARM_INS_LSR
        | ArmInsn::ARM_INS_ROR
        | ArmInsn::ARM_INS_RRX => normalize_shift(decoded),
        ArmInsn::ARM_INS_B | ArmInsn::ARM_INS_BL | ArmInsn::ARM_INS_BX | ArmInsn::ARM_INS_BLX => {
            normalize_branch(decoded)
        }
        ArmInsn::ARM_INS_CBZ | ArmInsn::ARM_INS_CBNZ => normalize_compare_branch(decoded),
        ArmInsn::ARM_INS_ADR => normalize_adr(decoded),
        ArmInsn::ARM_INS_LDR
        | ArmInsn::ARM_INS_LDRB
        | ArmInsn::ARM_INS_LDRSB
        | ArmInsn::ARM_INS_LDRH
        | ArmInsn::ARM_INS_LDRSH
        | ArmInsn::ARM_INS_STR
        | ArmInsn::ARM_INS_STRB
        | ArmInsn::ARM_INS_STRH => normalize_memory(decoded, opcode),
        ArmInsn::ARM_INS_LDM | ArmInsn::ARM_INS_STM => normalize_multi_reg(decoded, true),
        ArmInsn::ARM_INS_PUSH | ArmInsn::ARM_INS_POP => normalize_multi_reg(decoded, false),
        ArmInsn::ARM_INS_NOP | ArmInsn::ARM_INS_HINT | ArmInsn::ARM_INS_IT => Some(()),
        ArmInsn::ARM_INS_BKPT => {
            decoded.arm_operands.op2 = decoded.get_operand(0).cloned();
            Some(())
        }
        _ => None,
    }
}

fn normalize_ubfx(decoded: &mut DecodedInstructionBuilder) -> Option<()> {
    decoded.arm_operands.rd = reg_operand(decoded, 0)?;
    decoded.arm_operands.rn = reg_operand(decoded, 1)?;
    Some(())
}

fn normalize_extend(decoded: &mut DecodedInstructionBuilder) -> Option<()> {
    decoded.arm_operands.rd = reg_operand(decoded, 0)?;
    decoded.arm_operands.rn = reg_operand(decoded, 1)?;
    decoded.arm_operands.op2 = decoded.get_operand(1).cloned();
    Some(())
}

fn normalize_compare(decoded: &mut DecodedInstructionBuilder) -> Option<()> {
    decoded.arm_operands.rn = reg_operand(decoded, 0)?;
    decoded.arm_operands.op2 = decoded.get_operand(1).cloned();
    Some(())
}

fn normalize_move(decoded: &mut DecodedInstructionBuilder) -> Option<()> {
    decoded.arm_operands.rd = reg_operand(decoded, 0)?;
    decoded.arm_operands.rn = 0;
    decoded.arm_operands.op2 = decoded.get_operand(1).cloned();
    Some(())
}

fn normalize_binary(decoded: &mut DecodedInstructionBuilder) -> Option<()> {
    let rd = reg_operand(decoded, 0)?;
    decoded.arm_operands.rd = rd;
    if decoded.get_operand(2).is_some() {
        decoded.arm_operands.rn = reg_operand(decoded, 1)?;
        decoded.arm_operands.op2 = decoded.get_operand(2).cloned();
    } else {
        decoded.arm_operands.rn = rd;
        decoded.arm_operands.op2 = decoded.get_operand(1).cloned();
    }
    Some(())
}

fn normalize_shift(decoded: &mut DecodedInstructionBuilder) -> Option<()> {
    let rd = reg_operand(decoded, 0)?;
    let rm = reg_operand(decoded, 1)?;
    decoded.arm_operands.rd = rd;
    if decoded.get_operand(2).is_some() {
        decoded.arm_operands.rn = rm;
        decoded.arm_operands.op2 = decoded.get_operand(2).cloned();
    } else {
        decoded.arm_operands.rn = rd;
        decoded.arm_operands.op2 = decoded.get_operand(1).cloned();
    }
    Some(())
}

fn normalize_branch(decoded: &mut DecodedInstructionBuilder) -> Option<()> {
    decoded.arm_operands.op2 = decoded.get_operand(0).cloned();
    Some(())
}

fn normalize_compare_branch(decoded: &mut DecodedInstructionBuilder) -> Option<()> {
    decoded.arm_operands.rn = reg_operand(decoded, 0)?;
    decoded.arm_operands.op2 = decoded.get_operand(1).cloned();
    Some(())
}

fn normalize_adr(decoded: &mut DecodedInstructionBuilder) -> Option<()> {
    decoded.arm_operands.rd = reg_operand(decoded, 0)?;
    decoded.arm_operands.op2 = decoded.get_operand(1).cloned();
    Some(())
}

fn normalize_memory(decoded: &mut DecodedInstructionBuilder, opcode: &ArmOpcode<'_>) -> Option<()> {
    decoded.arm_operands.rd = reg_operand(decoded, 0)?;
    decoded.arm_operands.op2 = decoded.get_operand(1).cloned();
    decoded.arm_operands.mem_writeback = opcode.writeback();
    decoded.arm_operands.mem_post_index = decoded.get_operand(2).is_some();
    decoded.arm_operands.mem_post_imm = 0;
    decoded.arm_operands.mem_disp = 0;
    decoded.arm_operands.mem_has_index = false;

    let mem = match decoded.get_operand(1)?.op_type.clone() {
        DecodedOperandKind::Mem(mem) => mem,
        _ => return None,
    };

    decoded.arm_operands.rn = mem.base;
    decoded.arm_operands.mem_disp = mem.disp;
    decoded.arm_operands.mem_has_index = mem.index.is_some();

    if let Some(op3) = decoded.get_operand(2) {
        decoded.arm_operands.mem_post_imm = match op3.op_type {
            DecodedOperandKind::Imm(imm) => imm as i32,
            _ => 0,
        };
    }

    Some(())
}

fn normalize_multi_reg(decoded: &mut DecodedInstructionBuilder, with_base_reg: bool) -> Option<()> {
    decoded.transed_operands.clear();

    let mut base_captured = !with_base_reg;
    let mut index = 0usize;
    while let Some(op) = decoded.get_operand(index) {
        if let DecodedOperandKind::Reg(reg) = &op.op_type {
            if with_base_reg && !base_captured {
                decoded.transed_operands.push(*reg);
                base_captured = true;
            } else {
                decoded.transed_operands.push(*reg);
            }
        }
        index += 1;
    }

    if with_base_reg {
        if !base_captured {
            return None;
        }
        if decoded.transed_operands.len() > 1 {
            decoded.transed_operands[1..].sort_unstable();
        }
    } else {
        decoded.transed_operands.sort_unstable();
    }

    Some(())
}

fn reg_operand(decoded: &DecodedInstructionBuilder, index: usize) -> Option<u32> {
    match &decoded.get_operand(index)?.op_type {
        DecodedOperandKind::Reg(reg) => Some(*reg),
        _ => None,
    }
}

fn decode_operand(opcode: &ArmOpcode<'_>, operand: &ArmOperand) -> DecodedOperand {
    let op_type = match operand.op_type {
        ArmOperandType::Imm(imm) => DecodedOperandKind::Imm(i64::from(imm)),
        ArmOperandType::Reg(reg) => DecodedOperandKind::Reg(opcode.resolve_reg(reg)),
        ArmOperandType::Mem(mem) => DecodedOperandKind::Mem(DecodedMemOperand {
            base: opcode.resolve_reg(mem.base()),
            index: if mem.index() != RegId::INVALID_REG {
                Some(opcode.resolve_reg(mem.index()))
            } else {
                None
            },
            disp: mem.disp(),
        }),
        _ => DecodedOperandKind::Invalid,
    };

    DecodedOperand {
        op_type,
        shift: decode_shift(operand.shift),
    }
}

fn decode_shift(shift: ArmShift) -> DecodedShift {
    match shift {
        ArmShift::Invalid => DecodedShift::Invalid,
        ArmShift::Lsl(amount) => DecodedShift::Lsl(amount),
        ArmShift::Lsr(amount) => DecodedShift::Lsr(amount),
        ArmShift::Asr(amount) => DecodedShift::Asr(amount),
        ArmShift::Ror(amount) => DecodedShift::Ror(amount),
        ArmShift::Rrx(amount) => DecodedShift::Rrx(amount),
        ArmShift::AsrReg(_)
        | ArmShift::LslReg(_)
        | ArmShift::LsrReg(_)
        | ArmShift::RorReg(_)
        | ArmShift::RrxReg(_) => DecodedShift::Invalid,
    }
}

pub fn runtime_read_reg(cpu: &dyn CpuContext, data: &DecodedInstruction, reg: u32) -> u32 {
    if reg == 15 {
        data.address().wrapping_add(4)
    } else {
        cpu.read_reg(reg)
    }
}

fn runtime_read_reg_opcode(cpu: &dyn CpuContext, data: &ArmOpcode<'_>, reg: u32) -> u32 {
    if reg == 15 {
        data.address().wrapping_add(4)
    } else {
        cpu.read_reg(reg)
    }
}

pub fn operand_resolver_multi_runtime_opcode(
    cpu: &mut dyn CpuContext,
    data: &ArmOpcode<'_>,
) -> (u32, u32) {
    let rt = data.arm_operands.rd;
    let writeback = data.arm_operands.mem_writeback;
    let post_index = data.arm_operands.mem_post_index;
    let Some(op2) = data.decoded_operands.get(1) else {
        panic!("missing mem operand");
    };

    let (base_reg, base_val, disp, index_offset) = match &op2.op_type {
        DecodedOperandKind::Mem(mem) => {
            let base_reg = mem.base;
            let base_val = runtime_read_reg_opcode(cpu, data, base_reg);
            let disp = mem.disp;
            let index_offset = if let Some(index_reg) = mem.index {
                let val = runtime_read_reg_opcode(cpu, data, index_reg);
                let current_c = (cpu.read_apsr() >> 29) as u8 & 1;
                let (r2_val, _carry) = op_shift_match_by_shift(op2.shift, val, current_c);
                r2_val
            } else {
                0
            };
            (base_reg, base_val, disp, index_offset)
        }
        _ => panic!("operand2 is not a memory operand"),
    };

    let pre_offset = index_offset.wrapping_add_signed(disp);
    if !writeback {
        return (rt, base_val.wrapping_add(pre_offset));
    }

    if post_index {
        let post_offset = match data.decoded_operands.get(2).map(|operand| &operand.op_type) {
            Some(DecodedOperandKind::Imm(imm)) => *imm as u32,
            Some(DecodedOperandKind::Reg(reg)) => runtime_read_reg_opcode(cpu, data, *reg),
            _ => panic!("third operand is not an immediate/register"),
        };
        let addr = base_val;
        let new_base = base_val.wrapping_add(post_offset);
        cpu.write_reg(base_reg, new_base);
        (rt, addr)
    } else {
        let addr = base_val.wrapping_add(pre_offset);
        cpu.write_reg(base_reg, addr);
        (rt, addr)
    }
}

pub fn operand_resolver_multi_runtime(
    cpu: &mut dyn CpuContext,
    data: &DecodedInstruction,
) -> (u32, u32) {
    let rt = data.arm_operands.rd;
    let writeback = data.writeback();
    let post_index = data.arm_operands.mem_post_index;
    let Some(op2) = data.get_operand(1) else {
        panic!("missing mem operand");
    };

    let (base_reg, base_val, disp, index_offset) = match &op2.op_type {
        DecodedOperandKind::Mem(mem) => {
            let base_reg = mem.base;
            let base_val = runtime_read_reg(cpu, data, base_reg);
            let disp = mem.disp;
            let index_offset = if let Some(index_reg) = mem.index {
                let val = runtime_read_reg(cpu, data, index_reg);
                let current_c = (cpu.read_apsr() >> 29) as u8 & 1;
                let (r2_val, _carry) = op_shift_match_by_shift(op2.shift, val, current_c);
                r2_val
            } else {
                0
            };
            (base_reg, base_val, disp, index_offset)
        }
        _ => panic!("operand2 is not a memory operand"),
    };

    let pre_offset = index_offset.wrapping_add_signed(disp);
    if !writeback {
        return (rt, base_val.wrapping_add(pre_offset));
    }

    if post_index {
        let post_offset = match data.get_operand(2).map(|operand| &operand.op_type) {
            Some(DecodedOperandKind::Imm(imm)) => *imm as u32,
            Some(DecodedOperandKind::Reg(reg)) => runtime_read_reg(cpu, data, *reg),
            _ => panic!("third operand is not an immediate/register"),
        };
        let addr = base_val;
        let new_base = base_val.wrapping_add(post_offset);
        cpu.write_reg(base_reg, new_base);
        (rt, addr)
    } else {
        let addr = base_val.wrapping_add(pre_offset);
        cpu.write_reg(base_reg, addr);
        (rt, addr)
    }
}

pub fn resolve_op2_runtime(
    cpu: &mut dyn CpuContext,
    data: &DecodedInstruction,
) -> (u32, u8) {
    let current_c = ((cpu.read_apsr() >> 29) & 1) as u8;
    let Some(op2) = &data.arm_operands.op2 else {
        return (0, current_c);
    };

    match &op2.op_type {
        DecodedOperandKind::Reg(reg) => {
            let value = runtime_read_reg(cpu, data, *reg);
            op_shift_match_by_shift(op2.shift, value, current_c)
        }
        DecodedOperandKind::Imm(imm) => (*imm as u32, current_c),
        _ => (0, current_c),
    }
}

fn op_shift_match_by_shift(shift_kind: DecodedShift, val: u32, current_c: u8) -> (u32, u8) {
    match shift_kind {
        DecodedShift::Lsl(shift) => match shift {
            0 => (val, current_c),
            1..=31 => {
                let carry = (val >> (32 - shift)) & 1;
                (val << shift, carry as u8)
            }
            32 => (0, (val & 1) as u8),
            _ => panic!("Lsl invalid shift amount"),
        },
        DecodedShift::Lsr(shift) => match shift {
            0 => (val, current_c),
            1..=31 => {
                let carry = (val >> (shift - 1)) & 1;
                (val >> shift, carry as u8)
            }
            32 => (0, (val >> 31) as u8),
            _ => panic!("Lsr invalid shift amount"),
        },
        DecodedShift::Asr(shift) => match shift {
            0 => (val, current_c),
            1..=31 => {
                let carry = (val >> (shift - 1)) & 1;
                let res = ((val as i32) >> shift) as u32;
                (res, carry as u8)
            }
            _ => {
                let carry = (val >> 31) & 1;
                let res = if (val as i32) < 0 { u32::MAX } else { 0 };
                (res, carry as u8)
            }
        },
        DecodedShift::Ror(shift) => {
            if shift == 0 {
                (val, current_c)
            } else {
                let shift_mod = shift % 32;
                if shift_mod == 0 {
                    (val, (val >> 31) as u8)
                } else {
                    let res = val.rotate_right(shift_mod);
                    let carry = (res >> 31) & 1;
                    (res, carry as u8)
                }
            }
        }
        DecodedShift::Rrx(_) => {
            let c_out = (val & 1) as u8;
            let res = (val >> 1) | ((current_c as u32) << 31);
            (res, c_out)
        }
        DecodedShift::Invalid => (val, current_c),
    }
}

pub fn is_pc_reg(reg: u32) -> bool {
    reg == 15 || reg == ArmReg::ARM_REG_PC as u32
}

pub fn update_apsr_n(cpu: &mut dyn CpuContext, result: u32) {
    let mut apsr = cpu.read_apsr();
    if (result & (1u32 << 31)) != 0 {
        apsr |= 1u32 << 31;
    } else {
        apsr &= !(1u32 << 31);
    }
    cpu.write_apsr(apsr);
}

pub fn update_apsr_z(cpu: &mut dyn CpuContext, result: u32) {
    let mut apsr = cpu.read_apsr();
    if result == 0 {
        apsr |= 1u32 << 30;
    } else {
        apsr &= !(1u32 << 30);
    }
    cpu.write_apsr(apsr);
}

pub fn update_apsr_c(cpu: &mut dyn CpuContext, flag: u8) {
    let mut apsr = cpu.read_apsr();
    if flag != 0 {
        apsr |= 1u32 << 29;
    } else {
        apsr &= !(1u32 << 29);
    }
    cpu.write_apsr(apsr);
}

pub fn update_apsr_v(cpu: &mut dyn CpuContext, flag: u8) {
    let mut apsr = cpu.read_apsr();
    if flag != 0 {
        apsr |= 1u32 << 28;
    } else {
        apsr &= !(1u32 << 28);
    }
    cpu.write_apsr(apsr);
}

pub fn check_condition(cpu: &dyn CpuContext, cc: ArmCC) -> bool {
    let apsr = cpu.read_apsr();
    let n = (apsr >> 31) & 1;
    let z = (apsr >> 30) & 1;
    let c = (apsr >> 29) & 1;
    let v = (apsr >> 28) & 1;

    match cc {
        ArmCC::ARM_CC_EQ => z == 1,
        ArmCC::ARM_CC_NE => z == 0,
        ArmCC::ARM_CC_HS => c == 1,
        ArmCC::ARM_CC_LO => c == 0,
        ArmCC::ARM_CC_MI => n == 1,
        ArmCC::ARM_CC_PL => n == 0,
        ArmCC::ARM_CC_VS => v == 1,
        ArmCC::ARM_CC_VC => v == 0,
        ArmCC::ARM_CC_HI => c == 1 && z == 0,
        ArmCC::ARM_CC_LS => c == 0 || z == 1,
        ArmCC::ARM_CC_GE => n == v,
        ArmCC::ARM_CC_LT => n != v,
        ArmCC::ARM_CC_GT => z == 0 && n == v,
        ArmCC::ARM_CC_LE => z == 1 || n != v,
        ArmCC::ARM_CC_AL => true,
    }
}
