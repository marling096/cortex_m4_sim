use crate::context::CpuContext;
use crate::opcodes::decoded::DecodedInstructionBuilder;
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
        insnid: crate::arch::ArmInsn::ARM_INS_HINT as u32,
        name: "Hint".to_string(),
        length: 32,
        cycles: crate::opcodes::opcode::CycleInfo {
            fetch_cycles: 1,
            decode_cycles: 0,
            execute_cycles: 1,
        },
        exec: Op_Hint::execute,
        operand_resolver: &OpHintResolver,
        adjust_cycles: None,
    }]
}

// Hint{cond}
pub struct Op_Hint;
impl Executable for Op_Hint {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        // hint is treated as a no-op
        data.size()
    }
}

pub struct OpHintResolver;
impl OperandResolver for OpHintResolver {
    fn resolve(&self, raw: &ArmOpcode, decoded: &mut DecodedInstructionBuilder) -> u32 {
        decoded.arm_operands.condition = raw.condition();
        0
    }
}
