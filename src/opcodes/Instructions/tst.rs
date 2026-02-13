use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, CycleInfo, Executable, MatchFn, Opcode, Operand2_resolver, UpdateApsr_C,
    UpdateApsr_N, UpdateApsr_Z, check_condition, op2_imm_match, op2_reg_match,
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

        let (rd, rn, op2) = Operand2_resolver(cpu, data);
        let rn_data = cpu.read_reg(rn);
        let result = rn_data & op2;

        UpdateApsr_N(cpu, result);
        UpdateApsr_Z(cpu, result);
        // Note: C flag update logic should be added here based on Operand2 specifics
        data.size()
    }
}

pub struct Op_Teq;
impl Executable for Op_Teq {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        let (rd, rn, op2) = Operand2_resolver(cpu, data);
        let rn_data = cpu.read_reg(rn);
        let result = rn_data ^ op2;

        UpdateApsr_N(cpu, result);
        UpdateApsr_Z(cpu, result);
        // Note: C flag update logic should be added here based on Operand2 specifics
        data.size()
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
            adjust_cycles: None,
        },
    ]
}
