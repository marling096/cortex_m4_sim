use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{ArmOpcode, Executable, Operand_resolver_multi, check_condition};
use capstone::arch::arm::ArmOperandType;

pub struct Ldr_builder;
impl InstrBuilder for Ldr_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_ldr_def()
    }
}

pub fn add_ldr_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_LDR as u32,
            name: "LDR".to_string(),
            length: 16,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Ldr,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_LDRB as u32,
            name: "LDRB".to_string(),
            length: 16,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Ldrb,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_LDRSB as u32,
            name: "LDRSB".to_string(),
            length: 16,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Ldrsb,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_LDRH as u32,
            name: "LDRH".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Ldrh,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_LDRSH as u32,
            name: "LDRSH".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Ldrsh,
            adjust_cycles: None,
        },
        // Additional LDR variants (LDRH, LDRSB, LDRSH, LDRD) would be defined similarly
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

// --- LDR ---
pub struct Op_Ldr;
impl Executable for Op_Ldr {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        let (rt, mut addr) = Operand_resolver_multi(cpu, data);
        // data.op_writer();
        addr = addr & !3; // Align address to word boundary
        let val = cpu.read_mem(addr);
        print!("LDR from address 0x{:08X}: 0x{:08X}\n", addr, val);
        cpu.write_reg(rt, val);
        if rt == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Ldrb;
impl Executable for Op_Ldrb {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        let (rt, addr) = Operand_resolver_multi(cpu, data);
        let val = read_u8(cpu, addr);
        cpu.write_reg(rt, val);
        if rt == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Ldrsb;
impl Executable for Op_Ldrsb {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        let (rt, addr) = Operand_resolver_multi(cpu, data);
        let val = read_u8(cpu, addr);
        let signed_val = (val as i8) as i32 as u32;
        cpu.write_reg(rt, signed_val);
        if rt == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Ldrh;
impl Executable for Op_Ldrh {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        let (rt, addr) = Operand_resolver_multi(cpu, data);
        let val = read_u16(cpu, addr);
        cpu.write_reg(rt, val);
        if rt == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Ldrsh;
impl Executable for Op_Ldrsh {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        let (rt, addr) = Operand_resolver_multi(cpu, data);
        let val = read_u16(cpu, addr);
        let signed_val = (val as i16) as i32 as u32;
        cpu.write_reg(rt, signed_val);
        if rt == 15 { 0 } else { data.size() }
    }
}

// --- LDRD ---

pub struct Op_Ldrd;
impl Executable for Op_Ldrd {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        // let (rt, rt2, addr) = Operand_resolver_multi(cpu, data);
        // let val1 = cpu.read_mem(addr);
        // let val2 = cpu.read_mem(addr.wrapping_add(4));
        // cpu.write_reg(rt, val1);
        // cpu.write_reg(rt2, val2);
        data.size()
    }
}
