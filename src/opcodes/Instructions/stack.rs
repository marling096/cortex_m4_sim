use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{ArmOpcode, Executable, check_condition};

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
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        stack_push(cpu, data)
    }
}

pub struct Op_Pop;
impl Executable for Op_Pop {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        stack_pop(cpu, data)
    }
}

fn stack_push(cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
    if !check_condition(cpu, data.condition()) {
        return data.size();
    }
    data.op_writer();
    let mut sp = cpu.read_reg(13);
    print!("SP before PUSH:0x{:08X}\n", sp);
    // PUSH: full-descending (pre-decrement)
    let mut regs: Vec<u32> = Vec::new();
    for op in data.operands() {
        if let capstone::arch::arm::ArmOperandType::Reg(reg_id) = op.op_type {
            regs.push(data.resolve_reg(reg_id));
        }
    }
    regs.sort();
    let count = regs.len() as u32;
    let mut addr = sp.wrapping_sub(4 * count);
    for &r in &regs {
        let val = cpu.read_reg(r);
        print!("PUSH R{}:0x{:08X} to 0x{:08X}\n", r, val, addr);
        cpu.write_mem(addr, val);
        addr = addr.wrapping_add(4);
    }
    sp = sp.wrapping_sub(4 * count);
    print!("SP:0x{:08X}\n", sp);
    cpu.write_reg(13, sp);
    data.size()
}

fn stack_pop(cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
    if !check_condition(cpu, data.condition()) {
        return data.size();
    }
    data.op_writer();
    let lr = cpu.read_lr(0);
    print!("LR before POP:0x{:08X}\n", lr);
    let mut sp = cpu.read_reg(13);
    // POP: full-descending stack, so pop is post-increment
    let mut regs: Vec<u32> = Vec::new();
    for op in data.operands() {
        if let capstone::arch::arm::ArmOperandType::Reg(reg_id) = op.op_type {
            regs.push(data.resolve_reg(reg_id));
        }
    }
    regs.sort();
    for &r in &regs {
        let val = cpu.read_mem(sp);
        cpu.write_reg(r, val);
        sp = sp.wrapping_add(4);
    }
    print!("SP:0x{:08X}\n", sp);
    cpu.write_reg(13, sp);
    if regs.contains(&15) { 0 } else { data.size() }
}
