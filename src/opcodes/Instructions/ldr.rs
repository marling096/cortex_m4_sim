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
                execute_cycles: 1,
            },
            exec: &Op_Ldr,
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
                execute_cycles: 1,
            },
            exec: &Op_Ldrb,
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
                execute_cycles: 1,
            },
            exec: &Op_Ldrsb,
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
                execute_cycles: 1,
            },
            exec: &Op_Ldrh,
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
                execute_cycles: 1,
            },
            exec: &Op_Ldrsh,
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
fn read_u8(cpu: &mut dyn CpuContext, addr: u32) -> u32 {
    let word = cpu.read_mem(addr & !3);
    let shift = (addr & 3) * 8;
    (word >> shift) & 0xFF
}

fn read_u16(cpu: &mut dyn CpuContext, addr: u32) -> u32 {
    let word = cpu.read_mem(addr & !3);
    let shift = (addr & 2) * 8;
    (word >> shift) & 0xFFFF
}

// --- Address Resolution Helpers ---
fn operand_resolver_multi_cached(cpu: &mut dyn CpuContext, data: &ArmOpcode) -> (u32, u32) {
    if data.arm_operands.mem_has_index || data.arm_operands.mem_writeback {
        return operand_resolver_multi_runtime(cpu, data);
    }

    let rt = data.arm_operands.rd;
    let base = cpu.read_reg(data.arm_operands.rn);
    let addr = base.wrapping_add_signed(data.arm_operands.mem_disp);
    (rt, addr)
}

// --- LDR ---
pub struct Op_Ldr;
impl Executable for Op_Ldr {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
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
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
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
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
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
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
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
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
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
