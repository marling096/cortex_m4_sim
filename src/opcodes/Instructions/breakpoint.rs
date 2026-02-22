use crate::context::CpuContext;
use crate::opcodes::opcode::{ArmOpcode, Executable, Operand_resolver, OperandResolver};
use crate::opcodes::instruction::{InstrBuilder};

pub struct Breakpoint_builder;
impl InstrBuilder for Breakpoint_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_breakpoint_def()
    }
}

pub fn add_breakpoint_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_BKPT as u32,
            name: "BKPT".to_string(),
            length: 16,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Bkpt,
            operand_resolver: &OpBkptResolver,
            adjust_cycles: None,
        },
    ]
}

// BKPT #imm
// The BKPT Opcode causes the processor to enter Debug state. Debug tools can use this to investigate system state when the Opcode at a particular address is reached.

pub struct Op_Bkpt;
impl Executable for Op_Bkpt {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        // BKPT may be unconditional but follow common pattern
        let imm = data.transed_operands.get(0).copied().unwrap_or_else(|| Operand_resolver(cpu, data));
        breakpoint_imm(cpu, imm);
        0
    }
}
fn breakpoint_imm(_cpu: &mut dyn CpuContext, imm: u32) {
    println!("BKPT #{}", imm);
}

pub struct OpBkptResolver;
impl OperandResolver for OpBkptResolver {
    fn resolve(&self, cpu: &mut dyn CpuContext, data: &mut ArmOpcode) -> u32 {
        let imm = Operand_resolver(cpu, data);
        data.transed_operands.reserve(1);
        data.transed_operands.push(imm);
        imm
    }
}
