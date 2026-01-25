use crate::context::CpuContext;
use crate::opcodes::opcode::{ArmInstruction, Executable, Operand_resolver};
use crate::opcodes::instruction::{InstrBuilder};

pub struct Breakpoint_builder;
impl InstrBuilder for Breakpoint_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Instruction> {
        add_breakpoint_def()
    }
}

pub fn add_breakpoint_def() -> Vec<crate::opcodes::opcode::Instruction> {
    vec![
        crate::opcodes::opcode::Instruction {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_BKPT as u32,
            name: "BKPT".to_string(),
            length: 16,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Bkpt,
            adjust_cycles: None,
        },
    ]
}

// BKPT #imm
// The BKPT instruction causes the processor to enter Debug state. Debug tools can use this to investigate system state when the instruction at a particular address is reached.

pub struct Op_Bkpt;
impl Executable for Op_Bkpt {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
        breakpoint_imm(cpu, data);
    }
}

fn breakpoint_imm(cpu: &mut dyn CpuContext, data: &ArmInstruction) {
    let imm = Operand_resolver(cpu, data);
    println!("BKPT #{}", imm);
}
