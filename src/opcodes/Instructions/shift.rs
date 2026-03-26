use crate::arch::ArmInsn;
use crate::context::CpuContext;
use crate::opcodes::decoded::{DecodedInstructionBuilder, DecodedOperandKind};
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, OperandResolver, UpdateApsr_C, UpdateApsr_N,
    UpdateApsr_Z, check_condition,
};

pub struct Shiift_builder;
impl InstrBuilder for Shiift_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        addd_shift_def()
    }
}

pub fn addd_shift_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![
        crate::opcodes::opcode::Opcode {
            insnid: ArmInsn::ARM_INS_ASR as u32,
            name: "ASR".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Asr::execute,
            operand_resolver: &OpShiftResolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: ArmInsn::ARM_INS_LSL as u32,
            name: "LSL".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Lsl::execute,
            operand_resolver: &OpShiftResolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: ArmInsn::ARM_INS_LSR as u32,
            name: "LSR".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Lsr::execute,
            operand_resolver: &OpShiftResolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: ArmInsn::ARM_INS_ROR as u32,
            name: "ROR".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Ror::execute,
            operand_resolver: &OpShiftResolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: ArmInsn::ARM_INS_RRX as u32,
            name: "RRX".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Rrx::execute,
            operand_resolver: &OpShiftResolver,
            adjust_cycles: None,
        },
    ]
}

pub struct OpShiftResolver;
impl OperandResolver for OpShiftResolver {
    fn resolve(&self, raw: &ArmOpcode, decoded: &mut DecodedInstructionBuilder) -> u32 {
        let rd = match decoded.get_operand(0) {
            Some(op) => match op.op_type {
                DecodedOperandKind::Reg(reg) => reg,
                _ => 0,
            },
            None => 0,
        };
        let rm = match decoded.get_operand(1) {
            Some(op) => match op.op_type {
                DecodedOperandKind::Reg(reg) => reg,
                _ => 0,
            },
            None => 0,
        };

        decoded.arm_operands.condition = raw.condition();
        decoded.arm_operands.rd = rd;

        let op2 = decoded.get_operand(2).cloned();
        if op2.is_some() {
            decoded.arm_operands.rn = rm;
            decoded.arm_operands.op2 = op2;
        } else {
            decoded.arm_operands.rn = rd;
            decoded.arm_operands.op2 = decoded.get_operand(1).cloned();
        }
        rd
    }
}

// ASR, LSL, LSR, ROR, and RRX
// op{S}{cond} Rd, Rm, Rs
// op{S}{cond} Rd, Rm, #n
// RRX{S}{cond} Rd, Rm

pub struct Op_Asr;
impl Executable for Op_Asr {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        let rd = data.arm_operands.rd;
        let rm = data.arm_operands.rn;
        let rs_val = resolve_shift_amount(cpu, data);
        let rm_val = cpu.read_reg(rm);

        let result = if rs_val == 0 {
            rm_val
        } else if rs_val >= 32 {
            if (rm_val & 0x80000000) != 0 {
                0xFFFFFFFF
            } else {
                0
            }
        } else {
            ((rm_val as i32) >> rs_val) as u32
        };

        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
            if rs_val > 0 {
                let carry = if rs_val >= 32 {
                    (rm_val >> 31) & 1
                } else {
                    (rm_val >> (rs_val - 1)) & 1
                };
                UpdateApsr_C(cpu, carry as u8);
            }
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Lsl;
impl Executable for Op_Lsl {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        let rd = data.arm_operands.rd;
        let rm = data.arm_operands.rn;
        let rs_val = resolve_shift_amount(cpu, data);
        let rm_val = cpu.read_reg(rm);

        let result = if rs_val >= 32 { 0 } else { rm_val.wrapping_shl(rs_val) };
        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
            if rs_val > 0 {
                let carry = if rs_val > 32 {
                    0
                } else if rs_val == 32 {
                    rm_val & 1
                } else {
                    (rm_val >> (32 - rs_val)) & 1
                };
                UpdateApsr_C(cpu, carry as u8);
            }
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Lsr;
impl Executable for Op_Lsr {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        let rd = data.arm_operands.rd;
        let rm = data.arm_operands.rn;
        let rs_val = resolve_shift_amount(cpu, data);
        let rm_val = cpu.read_reg(rm);

        let result = if rs_val >= 32 { 0 } else { rm_val >> rs_val };

        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
            if rs_val > 0 {
                let carry = if rs_val > 32 {
                    0
                } else if rs_val == 32 {
                    (rm_val >> 31) & 1
                } else {
                    (rm_val >> (rs_val - 1)) & 1
                };
                UpdateApsr_C(cpu, carry as u8);
            }
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Ror;
impl Executable for Op_Ror {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        let rd = data.arm_operands.rd;
        let rm = data.arm_operands.rn;
        let rs_val = resolve_shift_amount(cpu, data);
        let rm_val = cpu.read_reg(rm);

        let shift = rs_val & 31;
        let result = if rs_val == 0 { rm_val } else { rm_val.rotate_right(shift) };

        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
            if rs_val > 0 {
                let carry = (result >> 31) & 1;
                UpdateApsr_C(cpu, carry as u8);
            }
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Rrx;
impl Executable for Op_Rrx {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        let rd = data.arm_operands.rd;
        let rm = data.arm_operands.rn;
        let rm_val = cpu.read_reg(rm);
        let carry_in = (cpu.read_apsr() >> 29) & 1;

        let result = (carry_in << 31) | (rm_val >> 1);
        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
            let carry = rm_val & 1;
            UpdateApsr_C(cpu, carry as u8);
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

fn resolve_shift_amount(cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
    match &data.arm_operands.op2 {
        Some(op) => match op.op_type {
            DecodedOperandKind::Imm(imm) => (imm as u32) & 0xFF,
            DecodedOperandKind::Reg(reg) => cpu.read_reg(reg) & 0xFF,
            _ => 0,
        },
        None => 0,
    }
}
