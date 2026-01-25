use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{ArmInstruction, Executable, Operand_resolver_multi, check_condition};
use capstone::arch::arm::ArmOperandType;

pub struct Str_builder;
impl InstrBuilder for Str_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Instruction> {
        add_str_def()
    }
}

pub fn add_str_def() -> Vec<crate::opcodes::opcode::Instruction> {
    vec![
        crate::opcodes::opcode::Instruction {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_STR as u32,
            name: "STR".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Str,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Instruction {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_STRB as u32,
            name: "STRB".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Strb,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Instruction {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_STRH as u32,
            name: "STRH".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Strh,
            adjust_cycles: None,
        },
    ]
}

// op{type}{cond} Rt, [Rn {, #offset}]
// op{type}{cond} Rt, [Rn, #offset]!
// op{type}{cond} Rt, [Rn], #offset
// opD{cond} Rt, Rt2, [Rn {, #offset}]
// opD{cond} Rt, Rt2, [Rn, #offset]!
// opD{cond} Rt, Rt2, [Rn], #offset

// Helpers for Memory Access
fn read_u32(cpu: &mut dyn CpuContext, addr: u32) -> u32 {
    cpu.read_mem(addr)
}

fn read_u8(cpu: &mut dyn CpuContext, addr: u32) -> u32 {
    let word = cpu.read_mem((addr & !3));
    let shift = (addr & 3) * 8;
    (word >> shift) & 0xFF
}

fn read_u16(cpu: &mut dyn CpuContext, addr: u32) -> u32 {
    let word = cpu.read_mem(addr & !3);
    let shift = (addr & 2) * 8;
    (word >> shift) & 0xFFFF
}

fn write_u32(cpu: &mut dyn CpuContext, addr: u32, val: u32) {
    cpu.write_mem(addr, val);
}

fn write_u8(cpu: &mut dyn CpuContext, addr: u32, val: u32) {
    let aligned_addr = addr & !3;
    let word = cpu.read_mem(aligned_addr);
    let shift = (addr & 3) * 8;
    let mask = !(0xFF << shift);
    let new_word = (word & mask) | ((val & 0xFF) << shift);
    cpu.write_mem(aligned_addr, new_word);
}

fn write_u16(cpu: &mut dyn CpuContext, addr: u32, val: u32) {
    let aligned_addr = addr & !3;
    let word = cpu.read_mem(aligned_addr);
    let shift = (addr & 2) * 8;
    let mask = !(0xFFFF << shift);
    let new_word = (word & mask) | ((val & 0xFFFF) << shift);
    cpu.write_mem(aligned_addr, new_word);
}

// --- Address Resolution Helpers ---

// --- STR ---
pub struct Op_Str;
impl Executable for Op_Str {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rt, addr) = Operand_resolver_multi(cpu, data);
        let val = cpu.read_reg(rt);
        cpu.write_mem(addr, val);
    }
}

pub struct Op_Strb;
impl Executable for Op_Strb {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rt, addr) = Operand_resolver_multi(cpu, data);
        let val = cpu.read_reg(rt) & 0xFF;
        write_u8(cpu, addr, val);
    }
}

pub struct Op_Strsb;
impl Executable for Op_Strsb {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rt, addr) = Operand_resolver_multi(cpu, data);
        let val = cpu.read_reg(rt) & 0xFF;
        write_u8(cpu, addr, val);
    }
}

pub struct Op_Strh;
impl Executable for Op_Strh {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rt, addr) = Operand_resolver_multi(cpu, data);
        let val = cpu.read_reg(rt) & 0xFFFF;
        write_u16(cpu, addr, val);
    }
}

pub struct Op_Strsh;
impl Executable for Op_Strsh {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rt, addr) = Operand_resolver_multi(cpu, data);
        let val = cpu.read_reg(rt) & 0xFFFF;
        write_u16(cpu, addr, val);
    }
}
