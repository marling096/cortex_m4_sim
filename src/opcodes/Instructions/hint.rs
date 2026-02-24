use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{ArmOpcode, Executable, OperandResolver, check_condition};

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
        operand_resolver: &OpHintResolver,
        adjust_cycles: None,
    }]
}

// Hint{cond}
pub struct Op_Hint;
impl Executable for Op_Hint {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        // hint is treated as a no-op
        data.size()
    }
}

pub struct OpHintResolver;
impl OperandResolver for OpHintResolver {
    fn resolve(&self, _data: &mut ArmOpcode) -> u32 {
        // Hint has no operands; nothing to push.
        0
    }
}
