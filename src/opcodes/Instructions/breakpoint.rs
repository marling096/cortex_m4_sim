use crate::context::CpuContext;
use crate::opcodes::decoded::{DecodedInstructionBuilder, DecodedOperandKind};
use crate::opcodes::opcode::{ArmOpcode, Executable, OperandResolver};
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
            insnid: crate::arch::ArmInsn::ARM_INS_BKPT as u32,
            name: "BKPT".to_string(),
            length: 16,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Bkpt::execute,
            operand_resolver: &OpBkptResolver,
            adjust_cycles: None,
        },
    ]
}

// BKPT #imm
// The BKPT Opcode causes the processor to enter Debug state. Debug tools can use this to investigate system state when the Opcode at a particular address is reached.

pub struct Op_Bkpt;
impl Executable for Op_Bkpt {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        // BKPT may be unconditional but follow common pattern
        let imm = resolve_bkpt_imm(cpu, data);
        breakpoint_imm(cpu, imm);
        0
    }
}
fn breakpoint_imm(_cpu: &mut dyn CpuContext, imm: u32) {
    println!("BKPT #{}", imm);
}

pub struct OpBkptResolver;
impl OperandResolver for OpBkptResolver {
    fn resolve(&self, raw: &ArmOpcode, decoded: &mut DecodedInstructionBuilder) -> u32 {
        decoded.arm_operands.condition = raw.condition();
        decoded.arm_operands.op2 = decoded.get_operand(0).cloned();
        0
    }
}

fn resolve_bkpt_imm(cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
    match &data.arm_operands.op2 {
        Some(op) => match op.op_type {
            DecodedOperandKind::Imm(imm) => imm as u32,
            DecodedOperandKind::Reg(reg) => cpu.read_reg(reg),
            _ => 0,
        },
        None => 0,
    }
}
