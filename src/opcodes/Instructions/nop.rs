use crate::context::CpuContext;
use crate::opcodes::opcode::{ArmInstruction, Executable, check_condition};
use crate::opcodes::instruction::{InstrBuilder};

pub struct Nop_builder;
impl InstrBuilder for Nop_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Instruction> {
        add_nop_def()
    }
}

pub fn add_nop_def() -> Vec<crate::opcodes::opcode::Instruction> {
    vec![
        crate::opcodes::opcode::Instruction {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_NOP as u32,
            name: "NOP".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Nop,
            adjust_cycles: None,
        },
    ]
}

// NOP{cond}
pub struct Op_Nop;
impl Executable for Op_Nop {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
        nop(cpu, data);
    }
}

fn nop(cpu: &mut dyn CpuContext, data: &ArmInstruction) {
    if !check_condition(cpu, data.condition()) {
        return;
    }
    // no-op
}
