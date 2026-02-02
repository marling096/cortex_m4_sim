use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, CycleInfo, Executable, MatchFn, Opcode, Operand2_resolver, UpdateApsr_C,
    UpdateApsr_N, UpdateApsr_Z, check_condition, op2_imm_match, op2_reg_match,
};
use capstone::arch::arm::{ArmInsn, ArmOperandType};

pub struct Op_It;
impl Executable for Op_It {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        it(cpu, data);
        data.size()
    }
}
fn it(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
    if !check_condition(cpu, data.condition()) {
        return;
    }
}

pub struct It_builder;
impl InstrBuilder for It_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_it_def()
    }
}
pub fn add_it_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![crate::opcodes::opcode::Opcode {
        insnid: capstone::arch::arm::ArmInsn::ARM_INS_IT as u32,
        name: "IT".to_string(),
        length: 16,
        cycles: crate::opcodes::opcode::CycleInfo {
            fetch_cycles: 1,
            decode_cycles: 0,
            execute_cycles: 1,
        },
        exec: &Op_It,
        adjust_cycles: None,
    }]
}
