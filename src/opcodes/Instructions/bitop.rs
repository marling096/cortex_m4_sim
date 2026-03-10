use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, CycleInfo, Executable, Opcode, OperandResolver, UpdateApsr_C, UpdateApsr_N,
    UpdateApsr_Z, check_condition,
};
use capstone::arch::arm::{ArmInsn, ArmOperandType, ArmShift};
use capstone::arch::DetailsArchInsn;

pub struct Bitop_builder;
impl InstrBuilder for Bitop_builder {
    fn build(&self) -> Vec<Opcode> {
        add_bitop_def()
    }
}

pub fn add_bitop_def() -> Vec<Opcode> {
    vec![
        Opcode {
            insnid: ArmInsn::ARM_INS_AND as u32,
            name: "AND".to_string(),
            length: 16,
            cycles: CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_And::execute,
            operand_resolver: &OpBitResolver,
            adjust_cycles: None,
        },
        Opcode {
            insnid: ArmInsn::ARM_INS_ORR as u32,
            name: "ORR".to_string(),
            length: 32,
            cycles: CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Orr::execute,
            operand_resolver: &OpBitResolver,
            adjust_cycles: None,
        },
        Opcode {
            insnid: ArmInsn::ARM_INS_EOR as u32,
            name: "EOR".to_string(),
            length: 32,
            cycles: CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Eor::execute,
            operand_resolver: &OpBitResolver,
            adjust_cycles: None,
        },
        Opcode {
            insnid: ArmInsn::ARM_INS_BIC as u32,
            name: "BIC".to_string(),
            length: 32,
            cycles: CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Bic::execute,
            operand_resolver: &OpBitResolver,
            adjust_cycles: None,
        },
        Opcode {
            insnid: ArmInsn::ARM_INS_ORN as u32,
            name: "ORN".to_string(),
            length: 32,
            cycles: CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Orn::execute,
            operand_resolver: &OpBitResolver,
            adjust_cycles: None,
        },
    ]
}

// AND, ORR, EOR, BIC, and ORN
// op{S}{cond} {Rd,} Rn, Operand2
// Operand2 can be a:
// 鈥?Constant
// 鈥?Register with optional shift

pub struct Op_And;
impl Executable for Op_And {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        let rd = data.arm_operands.rd;
        let rn = data.arm_operands.rn;
        let (op2, carry) = resolve_op2_and_carry(cpu, data);

        let result = cpu.read_reg(rn) & op2;

        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_C(cpu, carry);
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Orr;
impl Executable for Op_Orr {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        let rd = data.arm_operands.rd;
        let rn = data.arm_operands.rn;
        let (op2, carry) = resolve_op2_and_carry(cpu, data);
        let result = cpu.read_reg(rn) | op2;
        cpu.write_reg(rd, result);
        if data.update_flags() {
            UpdateApsr_C(cpu, carry);
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Bic;
impl Executable for Op_Bic {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        let rd = data.arm_operands.rd;
        let rn = data.arm_operands.rn;
        let (op2, carry) = resolve_op2_and_carry(cpu, data);
        let result = cpu.read_reg(rn) & !op2;
        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_C(cpu, carry);
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Orn;
impl Executable for Op_Orn {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        let rd = data.arm_operands.rd;
        let rn = data.arm_operands.rn;
        let (op2, carry) = resolve_op2_and_carry(cpu, data);
        let result = cpu.read_reg(rn) | !op2;
        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_C(cpu, carry);
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Eor;
impl Executable for Op_Eor {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        let rd = data.arm_operands.rd;
        let rn = data.arm_operands.rn;
        let (op2, carry) = resolve_op2_and_carry(cpu, data);
        let result = cpu.read_reg(rn) ^ op2;
        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_C(cpu, carry);
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct OpBitResolver;
impl OperandResolver for OpBitResolver {
    fn resolve(&self, data: &mut ArmOpcode) -> u32 {
        let arch_detail = if let capstone::arch::ArchDetail::ArmDetail(arm) = data.detail.arch_detail() {
            arm
        } else {
            panic!("ArmOpcode has invalid detail");
        };
        let ops: Vec<_> = arch_detail.operands().collect();

        let mut rd = 0;
        let mut rn = 0;

        if ops.len() == 3 {
            if let ArmOperandType::Reg(r) = ops[0].op_type {
                rd = data.resolve_reg(r);
            }
            if let ArmOperandType::Reg(r) = ops[1].op_type {
                rn = data.resolve_reg(r);
            }
        } else if ops.len() == 2 {
            if let ArmOperandType::Reg(r) = ops[0].op_type {
                rd = data.resolve_reg(r);
                rn = rd;
            }
        }

        data.arm_operands.condition = data.condition();
        data.arm_operands.rd = rd;
        data.arm_operands.rn = rn;
        data.arm_operands.op2 = ops.last().cloned();

        rd
    }
}

fn resolve_op2_and_carry(cpu: &mut dyn CpuContext, data: &ArmOpcode) -> (u32, u8) {
    let current_c = ((cpu.read_apsr() >> 29) & 1) as u8;
    let Some(op2) = &data.arm_operands.op2 else {
        return (0, current_c);
    };

    match op2.op_type {
        ArmOperandType::Reg(reg) => {
            let value = cpu.read_reg(data.resolve_reg(reg));
            op_shift_match(op2.shift, value, current_c)
        }
        ArmOperandType::Imm(imm) => (imm as u32, current_c),
        _ => (0, current_c),
    }
}

fn op_shift_match(shift: ArmShift, value: u32, current_c: u8) -> (u32, u8) {
    match shift {
        ArmShift::Lsl(amount) => match amount {
            0 => (value, current_c),
            1..=31 => {
                let carry = ((value >> (32 - amount)) & 1) as u8;
                (value.wrapping_shl(amount), carry)
            }
            32 => (0, (value & 1) as u8),
            _ => (0, 0),
        },
        ArmShift::Lsr(amount) => match amount {
            0 => (value, current_c),
            1..=31 => {
                let carry = ((value >> (amount - 1)) & 1) as u8;
                (value >> amount, carry)
            }
            32 => (0, (value >> 31) as u8),
            _ => (0, 0),
        },
        ArmShift::Asr(amount) => match amount {
            0 => (value, current_c),
            1..=31 => {
                let carry = ((value >> (amount - 1)) & 1) as u8;
                (((value as i32) >> amount) as u32, carry)
            }
            _ => {
                let carry = ((value >> 31) & 1) as u8;
                let result = if (value as i32) < 0 { 0xFFFF_FFFF } else { 0 };
                (result, carry)
            }
        },
        ArmShift::Ror(amount) => {
            if amount == 0 {
                (value, current_c)
            } else {
                let shift_mod = amount % 32;
                if shift_mod == 0 {
                    (value, (value >> 31) as u8)
                } else {
                    let result = value.rotate_right(shift_mod);
                    (result, ((result >> 31) & 1) as u8)
                }
            }
        }
        ArmShift::Rrx(_) => {
            let carry = (value & 1) as u8;
            let result = (value >> 1) | ((current_c as u32) << 31);
            (result, carry)
        }
        _ => (value, current_c),
    }
}

