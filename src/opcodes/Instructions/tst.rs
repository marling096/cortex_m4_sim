use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, CycleInfo, Executable, Opcode, OperandResolver, Operand2_resolver,
    UpdateApsr_C, UpdateApsr_N, UpdateApsr_Z, check_condition,
};
use capstone::arch::arm::{ArmInsn, ArmOperandType};

// TST{cond} Rn, Operand2
// TEQ{cond} Rn, Operand2

pub struct Op_Tst;
impl Executable for Op_Tst {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        let rn = data.transed_operands.get(0).copied().unwrap_or(0);
        let op2 = data.transed_operands.get(1).copied().unwrap_or(0);
        let rn_data = cpu.read_reg(rn);
        let result = rn_data & op2;

        UpdateApsr_N(cpu, result);
        UpdateApsr_Z(cpu, result);
        UpdateApsr_C(cpu, data.update_carry);

        data.size()
    }
}

pub struct Op_Teq;
impl Executable for Op_Teq {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        let rn = data.transed_operands.get(0).copied().unwrap_or(0);
        let op2 = data.transed_operands.get(1).copied().unwrap_or(0);
        let rn_data = cpu.read_reg(rn);
        let result = rn_data ^ op2;

        UpdateApsr_N(cpu, result);
        UpdateApsr_Z(cpu, result);
        UpdateApsr_C(cpu, data.update_carry);

        data.size()
    }
}

pub struct OpTst_resolver;
impl OperandResolver for OpTst_resolver {
    fn resolve(&self, cpu: &mut dyn crate::context::CpuContext, data: &mut ArmOpcode) -> u32 {
        let (_rd, rn, op2) = Operand2_resolver(cpu, data);
        data.transed_operands.reserve(2);
        data.transed_operands.push(rn);
        data.transed_operands.push(op2);
        op2
    }
}

pub struct Tst_builder;
impl InstrBuilder for Tst_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_tst_def()
    }
}
pub fn add_tst_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_TST as u32,
            name: "TST".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Tst,
            operand_resolver: &OpTst_resolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_TEQ as u32,
            name: "TEQ".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Teq,
            operand_resolver: &OpTst_resolver,
            adjust_cycles: None,
        },
    ]
}
