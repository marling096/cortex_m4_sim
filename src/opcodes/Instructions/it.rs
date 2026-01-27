use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, CycleInfo, Executable, MatchFn, Opcode, Operand2_resolver, UpdateApsr_C,
    UpdateApsr_N, UpdateApsr_Z, check_condition, op2_imm_match, op2_reg_match,
};
use capstone::arch::arm::{ArmInsn, ArmOperandType};

pub struct Op_It;
impl Executable for Op_It {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        it(cpu, data);
    }
}
fn it(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
    if !check_condition(cpu, data.condition()) {
        let pc = cpu.read_pc_counter();
        cpu.write_pc_counter(pc + 1); //无状态，通过外部pc计数器跳过后续指令

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
