use crate::arch::ArmCC;
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
