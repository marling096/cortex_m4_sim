use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{ArmOpcode, Executable, check_condition};

pub struct Hint_builder;
impl InstrBuilder for Hint_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_Hint_def()
    }
}

pub fn add_Hint_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![crate::opcodes::opcode::Opcode {
        insnid: capstone::arch::arm::ArmInsn::ARM_INS_HINT as u32,
        name: "Hint".to_string(),
        length: 32,
        cycles: crate::opcodes::opcode::CycleInfo {
            fetch_cycles: 1,
            decode_cycles: 0,
            execute_cycles: 1,
        },
        exec: &Op_Hint,
        adjust_cycles: None,
    }]
}

// Hint{cond}
pub struct Op_Hint;
impl Executable for Op_Hint {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        Hint(cpu, data);
        data.size()
    }
}

fn Hint(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
    if !check_condition(cpu, data.condition()) {
        return;
    }
    // no-op
}
