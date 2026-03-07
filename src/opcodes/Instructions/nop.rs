use crate::context::CpuContext;
use crate::opcodes::opcode::{ArmOpcode, Executable, OperandResolver, check_condition};
use crate::opcodes::instruction::{InstrBuilder};

pub struct Nop_builder;
impl InstrBuilder for Nop_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_nop_def()
    }
}

pub fn add_nop_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_NOP as u32,
            name: "NOP".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Nop::execute,
            operand_resolver: &OpNopResolver,
            adjust_cycles: None,
        },
    ]
}

// NOP{cond}
pub struct Op_Nop;
impl Executable for Op_Nop {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        // no-op
        data.size()
    }
}

pub struct OpNopResolver;
impl OperandResolver for OpNopResolver {
    fn resolve(&self, _data: &mut ArmOpcode) -> u32 {
        // NOP has no operands
        0
    }
}
