use crate::context::CpuContext;
use crate::opcodes::opcode::{ArmOpcode, Executable, check_condition};
use crate::opcodes::instruction::{InstrBuilder};

pub struct Stack_builder;
impl InstrBuilder for Stack_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_stack_def()
    }
}

pub fn add_stack_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_PUSH as u32,
            name: "PUSH".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Push,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_POP as u32,
            name: "POP".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Pop,
            adjust_cycles: None,
        },
    ]
}

// PUSH{cond} reglist
// POP{cond} reglist
pub struct Op_Push;
impl Executable for Op_Push {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        stack_push(cpu, data);
    }
}

pub struct Op_Pop;
impl Executable for Op_Pop {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        stack_pop(cpu, data);
    }
}

fn stack_push(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
    if !check_condition(cpu, data.condition()) {
        return;
    }

    let mut sp = cpu.read_reg(13);
    // PUSH: full-descending (pre-decrement)
    for &r in data.operands.iter().rev() {
        sp = sp.wrapping_sub(4);
        let val = cpu.read_reg(r);
        cpu.write_mem(sp, val);
    }
    cpu.write_reg(13, sp);
}

fn stack_pop(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
    if !check_condition(cpu, data.condition()) {
        return;
    }

    let mut sp = cpu.read_reg(13);
    // POP: full-descending stack, so pop is post-increment
    for &r in data.operands.iter() {
        let val = cpu.read_mem(sp);
        cpu.write_reg(r, val);
        sp = sp.wrapping_add(4);
    }
    cpu.write_reg(13, sp);
}

// pub struct Stack_Push;

// impl Executable for Stack_Push {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         stack_push(cpu, data);
//     }
// }
// pub struct Stack_Pop;

// impl Executable for Stack_Pop {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         stack_pop(cpu, data);
//     }
// }
