use crate::context::CpuContext;
use crate::opcodes::decoded::DecodedInstructionBuilder;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{ArmOpcode, Executable, OperandResolver, check_condition};

pub struct Op_It;
impl Executable for Op_It {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        // IT sets the following instruction's condition in Thumb; emulator treats as no-op
        data.size()
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
        insnid: crate::arch::ArmInsn::ARM_INS_IT as u32,
        name: "IT".to_string(),
        length: 16,
        cycles: crate::opcodes::opcode::CycleInfo {
            fetch_cycles: 1,
            decode_cycles: 0,
            execute_cycles: 1,
        },
        exec: Op_It::execute,
        operand_resolver: &OpItResolver,
        adjust_cycles: None,
    }]
}

pub struct OpItResolver;
impl OperandResolver for OpItResolver {
    fn resolve(&self, raw: &ArmOpcode, decoded: &mut DecodedInstructionBuilder) -> u32 {
        decoded.arm_operands.condition = raw.condition();
        0
    }
}
