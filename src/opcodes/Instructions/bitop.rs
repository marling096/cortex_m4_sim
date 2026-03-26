use crate::arch::ArmInsn;
use crate::context::CpuContext;
use crate::opcodes::decoded::{DecodedInstructionBuilder, DecodedOperandKind};
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, CycleInfo, Executable, Opcode, OperandResolver, UpdateApsr_C, UpdateApsr_N,
    UpdateApsr_Z, check_condition, resolve_op2_runtime,
};

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
    fn resolve(&self, raw: &ArmOpcode, decoded: &mut DecodedInstructionBuilder) -> u32 {
        let mut rd = 0;
        let mut rn = 0;

        if decoded.get_operand(2).is_some() {
            if let Some(op) = decoded.get_operand(0) {
                if let DecodedOperandKind::Reg(reg) = op.op_type {
                    rd = reg;
                }
            }
            if let Some(op) = decoded.get_operand(1) {
                if let DecodedOperandKind::Reg(reg) = op.op_type {
                    rn = reg;
                }
            }
        } else if decoded.get_operand(1).is_some() {
            if let Some(op) = decoded.get_operand(0) {
                if let DecodedOperandKind::Reg(reg) = op.op_type {
                    rd = reg;
                    rn = rd;
                }
            }
        }

        decoded.arm_operands.condition = raw.condition();
        decoded.arm_operands.rd = rd;
        decoded.arm_operands.rn = rn;
        decoded.arm_operands.op2 = if decoded.get_operand(2).is_some() {
            decoded.get_operand(2).cloned()
        } else {
            decoded.get_operand(1).cloned()
        };

        rd
    }
}

fn resolve_op2_and_carry(cpu: &mut dyn CpuContext, data: &ArmOpcode) -> (u32, u8) {
    resolve_op2_runtime(cpu, data)
}

