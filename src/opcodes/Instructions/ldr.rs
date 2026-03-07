use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, OperandResolver, check_condition, operand_resolver_multi_runtime,
};
use capstone::arch::arm::ArmOperandType;
use capstone::arch::DetailsArchInsn;

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
                execute_cycles: 2,
            },
            exec: Op_Ldr::execute,
            operand_resolver: &OpLdrResolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_LDRB as u32,
            name: "LDRB".to_string(),
            length: 16,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 2,
            },
            exec: Op_Ldrb::execute,
            operand_resolver: &OpLdrResolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_LDRSB as u32,
            name: "LDRSB".to_string(),
            length: 16,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 2,
            },
            exec: Op_Ldrsb::execute,
            operand_resolver: &OpLdrResolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_LDRH as u32,
            name: "LDRH".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 2,
            },
            exec: Op_Ldrh::execute,
            operand_resolver: &OpLdrResolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_LDRSH as u32,
            name: "LDRSH".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 2,
            },
            exec: Op_Ldrsh::execute,
            operand_resolver: &OpLdrResolver,
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
#[inline(always)]
fn read_u8(cpu: &mut crate::cpu::Cpu, addr: u32) -> u32 {
    let word = cpu.read_mem(addr & !3);
    let shift = (addr & 3) * 8;
    (word >> shift) & 0xFF
}

#[inline(always)]
fn read_u16(cpu: &mut crate::cpu::Cpu, addr: u32) -> u32 {
    let word = cpu.read_mem(addr & !3);
    let shift = (addr & 2) * 8;
    (word >> shift) & 0xFFFF
}

// --- Address Resolution Helpers ---
#[inline(always)]
fn operand_resolver_multi_cached(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> (u32, u32) {
    let ops = &data.arm_operands;

    if ops.mem_has_index {
        return operand_resolver_multi_runtime(cpu, data);
    }

    operand_resolver_multi_cached_no_index(cpu, ops)
}

#[inline(always)]
fn operand_resolver_multi_cached_no_index(
    cpu: &mut crate::cpu::Cpu,
    ops: &crate::opcodes::opcode::ArmOperands,
) -> (u32, u32) {
    let rt = ops.rd;

    if !ops.mem_writeback {
        let base = cpu.read_reg(ops.rn);
        let addr = base.wrapping_add_signed(ops.mem_disp);
        return (rt, addr);
    }

    let base = cpu.read_reg(ops.rn);
    if ops.mem_post_index {
        // Post-indexed addressing uses the old base for access, then updates base.
        let new_base = base.wrapping_add_signed(ops.mem_post_imm);
        cpu.write_reg(ops.rn, new_base);
        (rt, base)
    } else {
        let addr = base.wrapping_add_signed(ops.mem_disp);
        cpu.write_reg(ops.rn, addr);
        (rt, addr)
    }
}

// --- LDR ---
pub struct Op_Ldr;
impl Executable for Op_Ldr {
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        let (rt, mut addr) = operand_resolver_multi_cached(cpu, data);
        // data.op_writer();
        addr = addr & !3; // Align address to word boundary
        let val = cpu.read_mem(addr);
        // print!("LDR from address 0x{:08X}: 0x{:08X}\n", addr, val);
        cpu.write_reg(rt, val);
        if rt == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Ldrb;
impl Executable for Op_Ldrb {
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        let (rt, addr) = operand_resolver_multi_cached(cpu, data);
        let val = read_u8(cpu, addr);
        cpu.write_reg(rt, val);
        if rt == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Ldrsb;
impl Executable for Op_Ldrsb {
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        let (rt, addr) = operand_resolver_multi_cached(cpu, data);
        let val = read_u8(cpu, addr);
        let signed_val = (val as i8) as i32 as u32;
        cpu.write_reg(rt, signed_val);
        if rt == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Ldrh;
impl Executable for Op_Ldrh {
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        let (rt, addr) = operand_resolver_multi_cached(cpu, data);
        let val = read_u16(cpu, addr);
        cpu.write_reg(rt, val);
        if rt == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Ldrsh;
impl Executable for Op_Ldrsh {
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        let (rt, addr) = operand_resolver_multi_cached(cpu, data);
        let val = read_u16(cpu, addr);
        let signed_val = (val as i16) as i32 as u32;
        cpu.write_reg(rt, signed_val);
        if rt == 15 { 0 } else { data.size() }
    }
}

pub struct OpLdrResolver;
impl OperandResolver for OpLdrResolver {
    fn resolve(&self, data: &mut ArmOpcode) -> u32 {
        let arch_detail = if let capstone::arch::ArchDetail::ArmDetail(arm) = data.detail.arch_detail() {
            arm
        } else {
            panic!("ArmOpcode has invalid detail");
        };

        let mut operands = arch_detail.operands();
        let op_rt = operands.next().expect("missing rt operand");
        let op_mem = operands.next().expect("missing mem operand");
        let op3 = operands.next();

        data.arm_operands.rd = match op_rt.op_type {
            ArmOperandType::Reg(r) => data.resolve_reg(r),
            _ => panic!("first operand is not a register"),
        };
        data.arm_operands.op2 = Some(op_mem.clone());

        data.arm_operands.mem_has_index = false;
        data.arm_operands.mem_writeback = data.writeback();
        data.arm_operands.mem_post_index = op3.is_some();
        data.arm_operands.mem_post_imm = 0;
        data.arm_operands.mem_disp = 0;

        match op_mem.op_type {
            ArmOperandType::Mem(mem) => {
                data.arm_operands.rn = data.resolve_reg(mem.base());
                data.arm_operands.mem_disp = mem.disp();
                data.arm_operands.mem_has_index = mem.index() != capstone::RegId::INVALID_REG;
            }
            _ => panic!("operand2 is not a memory operand"),
        }

        if let Some(op3) = op3 {
            data.arm_operands.mem_post_imm = match op3.op_type {
                ArmOperandType::Imm(imm) => imm,
                _ => 0,
            };
        }

        data.arm_operands.rd
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::CpuContext;
    use crate::cpu::Cpu;
    use crate::peripheral::bus::Bus;
    use std::hint::black_box;
    use std::sync::Arc;
    use std::sync::atomic::AtomicU32;
    use std::time::Instant;

    fn make_cpu() -> Cpu {
        Cpu::new(Arc::new(AtomicU32::new(8_000_000)), 1, Bus::new(), Bus::new())
    }

    #[test]
    fn operand_resolver_cached_non_writeback_uses_base_register_value() {
        let mut cpu = make_cpu();
        cpu.write_reg(2, 0x2000_0100);

        let mut ops = crate::opcodes::opcode::ArmOperands::new();
        ops.rd = 1;
        ops.rn = 2;
        ops.mem_has_index = false;
        ops.mem_writeback = false;
        ops.mem_post_index = false;
        ops.mem_disp = 0x20;

        let (rt, addr) = operand_resolver_multi_cached_no_index(&mut cpu, &ops);
        assert_eq!(rt, 1);
        assert_eq!(addr, 0x2000_0120);
    }

    #[test]
    fn operand_resolver_cached_writeback_pre_index_uses_mem_disp() {
        let mut cpu = make_cpu();
        cpu.write_reg(4, 0x2000_0200);

        let mut ops = crate::opcodes::opcode::ArmOperands::new();
        ops.rd = 3;
        ops.rn = 4;
        ops.mem_has_index = false;
        ops.mem_writeback = true;
        ops.mem_post_index = false;
        ops.mem_disp = -0x10;

        let (rt, addr) = operand_resolver_multi_cached_no_index(&mut cpu, &ops);
        assert_eq!(rt, 3);
        assert_eq!(addr, 0x2000_01F0);
        assert_eq!(cpu.read_reg(4), 0x2000_01F0);
    }

    #[test]
    fn operand_resolver_cached_writeback_post_index_uses_post_imm() {
        let mut cpu = make_cpu();
        cpu.write_reg(6, 0x2000_0300);

        let mut ops = crate::opcodes::opcode::ArmOperands::new();
        ops.rd = 5;
        ops.rn = 6;
        ops.mem_has_index = false;
        ops.mem_writeback = true;
        ops.mem_post_index = true;
        ops.mem_post_imm = 0x24;
        ops.mem_disp = -0x100;

        let (rt, addr) = operand_resolver_multi_cached_no_index(&mut cpu, &ops);
        assert_eq!(rt, 5);
        assert_eq!(addr, 0x2000_0300);
        assert_eq!(cpu.read_reg(6), 0x2000_0324);
    }

    #[test]
    fn perf_operand_resolver_cached_no_index() {
        let loops = 1_000_000u64;
        let mut cpu = make_cpu();
        cpu.write_reg(2, 0x2000_0100);
        cpu.write_reg(4, 0x2000_0200);
        cpu.write_reg(6, 0x2000_0300);

        let mut ops_non_wb = crate::opcodes::opcode::ArmOperands::new();
        ops_non_wb.rd = 1;
        ops_non_wb.rn = 2;
        ops_non_wb.mem_writeback = false;
        ops_non_wb.mem_disp = 0x20;

        let mut ops_pre = crate::opcodes::opcode::ArmOperands::new();
        ops_pre.rd = 3;
        ops_pre.rn = 4;
        ops_pre.mem_writeback = true;
        ops_pre.mem_post_index = false;
        ops_pre.mem_disp = -0x10;

        let mut ops_post = crate::opcodes::opcode::ArmOperands::new();
        ops_post.rd = 5;
        ops_post.rn = 6;
        ops_post.mem_writeback = true;
        ops_post.mem_post_index = true;
        ops_post.mem_post_imm = 0x24;

        let start = Instant::now();
        let mut checksum = 0u32;
        for _ in 0..loops {
            let (rt, addr) = operand_resolver_multi_cached_no_index(&mut cpu, &ops_non_wb);
            checksum ^= rt ^ addr;
        }
        let elapsed_non_wb = start.elapsed();

        let start = Instant::now();
        for _ in 0..loops {
            let (rt, addr) = operand_resolver_multi_cached_no_index(&mut cpu, &ops_pre);
            checksum ^= rt ^ addr;
        }
        let elapsed_pre = start.elapsed();

        let start = Instant::now();
        for _ in 0..loops {
            let (rt, addr) = operand_resolver_multi_cached_no_index(&mut cpu, &ops_post);
            checksum ^= rt ^ addr;
        }
        let elapsed_post = start.elapsed();

        let to_ns_per_op = |d: std::time::Duration| d.as_nanos() as f64 / loops as f64;
        println!(
            "[perf][ldr] operand_resolver_multi_cached_no_index non_writeback: total={:?}, {:.2} ns/op",
            elapsed_non_wb,
            to_ns_per_op(elapsed_non_wb)
        );
        println!(
            "[perf][ldr] operand_resolver_multi_cached_no_index writeback_pre_index: total={:?}, {:.2} ns/op",
            elapsed_pre,
            to_ns_per_op(elapsed_pre)
        );
        println!(
            "[perf][ldr] operand_resolver_multi_cached_no_index writeback_post_index: total={:?}, {:.2} ns/op",
            elapsed_post,
            to_ns_per_op(elapsed_post)
        );

        black_box(checksum);
        assert!(elapsed_non_wb.as_nanos() > 0);
        assert!(elapsed_pre.as_nanos() > 0);
        assert!(elapsed_post.as_nanos() > 0);
    }
}
