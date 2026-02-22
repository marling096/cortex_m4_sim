use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, CycleInfo, Executable, MatchFn, Opcode, Operand2_resolver, UpdateApsr_C,
    UpdateApsr_N, UpdateApsr_Z, check_condition, op2_imm_match, op2_reg_match,
};
use capstone::arch::arm::{ArmInsn, ArmOperandType};

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
            exec: &Op_And,
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
            exec: &Op_Orr,
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
            exec: &Op_Eor,
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
            exec: &Op_Bic,
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
            exec: &Op_Orn,
            operand_resolver: &OpBitResolver,
            adjust_cycles: None,
        },
    ]
}

// AND, ORR, EOR, BIC, and ORN
// op{S}{cond} {Rd,} Rn, Operand2
// Operand2 can be a:
// • Constant
// • Register with optional shift

pub struct Op_And;
impl Executable for Op_And {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        let rd = data.transed_operands.get(0).copied().unwrap_or(0);
        let rn = data.transed_operands.get(1).copied().unwrap_or(0);
        let op2 = data.transed_operands.get(2).copied().unwrap_or(0);

        let result = cpu.read_reg(rn) & op2;

        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_C(cpu, data.update_carry);
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Orr;
impl Executable for Op_Orr {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        let rd = data.transed_operands.get(0).copied().unwrap_or(0);
        let rn = data.transed_operands.get(1).copied().unwrap_or(0);
        let op2 = data.transed_operands.get(2).copied().unwrap_or(0);
        let result = cpu.read_reg(rn) | op2;
        cpu.write_reg(rd, result);
        if data.update_flags() {
            UpdateApsr_C(cpu, data.update_carry);
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Bic;
impl Executable for Op_Bic {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        let rd = data.transed_operands.get(0).copied().unwrap_or(0);
        let rn = data.transed_operands.get(1).copied().unwrap_or(0);
        let op2 = data.transed_operands.get(2).copied().unwrap_or(0);
        let result = cpu.read_reg(rn) & !op2;
        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_C(cpu, data.update_carry);
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Orn;
impl Executable for Op_Orn {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        let rd = data.transed_operands.get(0).copied().unwrap_or(0);
        let rn = data.transed_operands.get(1).copied().unwrap_or(0);
        let op2 = data.transed_operands.get(2).copied().unwrap_or(0);
        let result = cpu.read_reg(rn) | !op2;
        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_C(cpu, data.update_carry);
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Eor;
impl Executable for Op_Eor {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        let rd = data.transed_operands.get(0).copied().unwrap_or(0);
        let rn = data.transed_operands.get(1).copied().unwrap_or(0);
        let op2 = data.transed_operands.get(2).copied().unwrap_or(0);
        let result = cpu.read_reg(rn) ^ op2;
        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_C(cpu, data.update_carry);
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct OpBitResolver;
impl crate::opcodes::opcode::OperandResolver for OpBitResolver {
    fn resolve(&self, cpu: &mut dyn crate::context::CpuContext, data: &mut ArmOpcode) -> u32 {
        let (rd, rn, op2) = crate::opcodes::opcode::Operand2_resolver(cpu, data);
        data.transed_operands.reserve(3);
        data.transed_operands.push(rd);
        data.transed_operands.push(rn);
        data.transed_operands.push(op2);
        op2
    }
}

