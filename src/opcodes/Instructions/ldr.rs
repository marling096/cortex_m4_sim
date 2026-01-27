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
            length: 32,
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
            length: 32,
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
            length: 32,
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
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rt, addr) = Operand_resolver_multi(cpu, data);
        // data.op_writer();
        let val = cpu.read_mem(addr);
        cpu.write_reg(rt, val);
    }
}

pub struct Op_Ldrb;
impl Executable for Op_Ldrb {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rt, addr) = Operand_resolver_multi(cpu, data);
        let val = read_u8(cpu, addr);
        cpu.write_reg(rt, val);
    }
}

pub struct Op_Ldrsb;
impl Executable for Op_Ldrsb {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rt, addr) = Operand_resolver_multi(cpu, data);
        let val = read_u8(cpu, addr);
        let signed_val = (val as i8) as i32 as u32;
        cpu.write_reg(rt, signed_val);
    }
}

pub struct Op_Ldrh;
impl Executable for Op_Ldrh {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rt, addr) = Operand_resolver_multi(cpu, data);
        let val = read_u16(cpu, addr);
        cpu.write_reg(rt, val);
    }
}

pub struct Op_Ldrsh;
impl Executable for Op_Ldrsh {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rt, addr) = Operand_resolver_multi(cpu, data);
        let val = read_u16(cpu, addr);
        let signed_val = (val as i16) as i32 as u32;
        cpu.write_reg(rt, signed_val);
    }
}

// --- LDRD ---

pub struct Op_Ldrd;
impl Executable for Op_Ldrd {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        // let (rt, rt2, addr) = Operand_resolver_multi(cpu, data);
        // let val1 = cpu.read_mem(addr);
        // let val2 = cpu.read_mem(addr.wrapping_add(4));
        // cpu.write_reg(rt, val1);
        // cpu.write_reg(rt2, val2);
    }
}

// fn resolve_addr_imm(
//     data: &ArmOpcode,
//     ops: &Vec<capstone::arch::arm::ArmOperand>,
//     mem_idx: usize,
//     cpu: &dyn CpuContext,
// ) -> Option<(u32, Option<(u32, u32)>)> {
//     if let ArmOperandType::Mem(mem) = ops[mem_idx].op_type {
//         let base = data.resolve_reg(mem.base());
//         let base_val = cpu.read_reg(base);
//         // "Offset are all immediates": disp is the immediate offset for [Rn, #imm]
//         let disp = mem.disp() as i32;

//         if data.post_index() {
//             // Post-index: Address = Base. Writeback = Base + offset
//             let addr = base_val;
//             // For post-index, the immediate offset is typically the next operand
//             let next_op_idx = mem_idx + 1;
//             let offset = if ops.len() > next_op_idx {
//                 if let ArmOperandType::Imm(val) = ops[next_op_idx].op_type {
//                     val as i32
//                 } else {
//                     disp // Fallback to disp if no separate Imm operand found (unlikely for valid post-index)
//                 }
//             } else {
//                 disp
//             };
//             Some((addr, Some((base, base_val.wrapping_add(offset as u32)))))
//         } else if data.writeback() {
//             // Pre-index: Address = Base + Disp. Writeback = Address
//             // disp is the immediate offset
//             let addr = base_val.wrapping_add(disp as u32);
//             Some((addr, Some((base, addr))))
//         } else {
//             // The offset value is added to or subtracted from the address obtained from the
//             // register Rn. The result is used as the address for the memory access. The register
//             // Rn is unaltered. The assembly language syntax for this mode is:
//             // [Rn, #offset]
//             Some((base_val.wrapping_add(disp as u32), None))
//         }
//     } else {
//         None
//     }
// }

// fn resolve_addr_reg(
//     data: &ArmOpcode,
//     ops: &Vec<capstone::arch::arm::ArmOperand>,
//     mem_idx: usize,
//     cpu: &dyn CpuContext,
// ) -> Option<(u32, Option<(u32, u32)>)> {
//     if let ArmOperandType::Mem(mem) = ops[mem_idx].op_type {
//         // Should have an index register. If 0 (Invalid), it's likely an Imm offset or malformed.
//         if mem.index().0 == 0 {
//             return None;
//         }

//         let base = data.resolve_reg(mem.base());
//         let index = data.resolve_reg(mem.index());

//         // Offset is implemented as addition/subtraction, shift does not exist
//         let index_val = cpu.read_reg(index);

//         let base_val = cpu.read_reg(base);

//         // Handle Scale (Add/Sub)
//         let scale = mem.scale();
//         let addr = if scale == -1 {
//             base_val.wrapping_sub(index_val)
//         } else {
//             base_val.wrapping_add(index_val)
//         };

//         if data.post_index() {
//             Some((base_val, Some((base, addr))))
//         } else if data.writeback() {
//             Some((addr, Some((base, addr))))
//         } else {
//             Some((addr, None))
//         }
//     } else {
//         None
//     }
// }

// fn resolve_addr_lit(
//     data: &ArmOpcode,
//     ops: &Vec<capstone::arch::arm::ArmOperand>,
//     mem_idx: usize,
//     cpu: &dyn CpuContext,
// ) -> Option<u32> {
//     if let ArmOperandType::Mem(mem) = ops[mem_idx].op_type {
//         // Typically Base=PC.
//         // If Base is not PC, it is strange for "Literal" load, but we can compute standard Mem
//         // LDR (literal) encoding uses PC as base.
//         let base = data.resolve_reg(mem.base());
//         let base_val = if base == 15 {
//             let pc = cpu.read_pc();
//             pc & 0xFFFFFFFC
//         } else {
//             cpu.read_reg(base)
//         };
//         Some(base_val.wrapping_add(mem.disp() as u32))
//     } else if let ArmOperandType::Imm(val) = ops[mem_idx].op_type {
//         Some(val as u32)
//     } else {
//         None
//     }
// }

// // --- Common Logic Implementations ---

// fn load_imm(cpu: &mut dyn CpuContext, data: &ArmOpcode, size: u32, signed: bool) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let ops: Vec<_> = data.operands().collect();
//     let rt = if let ArmOperandType::Reg(r) = ops[0].op_type {
//         data.resolve_reg(r)
//     } else {
//         return;
//     };

//     if let Some((addr, wb)) = resolve_addr_imm(data, &ops, 1, cpu) {
//         let val = match size {
//             1 => {
//                 let v = read_u8(cpu, addr);
//                 if signed { (v as i8) as i32 as u32 } else { v }
//             }
//             2 => {
//                 let v = read_u16(cpu, addr);
//                 if signed { (v as i16) as i32 as u32 } else { v }
//             }
//             _ => read_u32(cpu, addr),
//         };
//         cpu.write_reg(rt, val);
//         if let Some((wb_reg, wb_val)) = wb {
//             cpu.write_reg(wb_reg, wb_val);
//         }
//     }
// }

// fn load_dual_imm(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let ops: Vec<_> = data.operands().collect();
//     let rt = if let ArmOperandType::Reg(r) = ops[0].op_type {
//         data.resolve_reg(r)
//     } else {
//         return;
//     };
//     let rt2 = if let ArmOperandType::Reg(r) = ops[1].op_type {
//         data.resolve_reg(r)
//     } else {
//         return;
//     };

//     if let Some((addr, wb)) = resolve_addr_imm(data, &ops, 2, cpu) {
//         let val1 = read_u32(cpu, addr);
//         let val2 = read_u32(cpu, addr.wrapping_add(4));
//         cpu.write_reg(rt, val1);
//         cpu.write_reg(rt2, val2);
//         if let Some((wb_reg, wb_val)) = wb {
//             cpu.write_reg(wb_reg, wb_val);
//         }
//     }
// }

// fn store_imm(cpu: &mut dyn CpuContext, data: &ArmOpcode, size: u32) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let ops: Vec<_> = data.operands().collect();
//     let rt = if let ArmOperandType::Reg(r) = ops[0].op_type {
//         data.resolve_reg(r)
//     } else {
//         return;
//     };
//     let val = cpu.read_reg(rt);

//     if let Some((addr, wb)) = resolve_addr_imm(data, &ops, 1, cpu) {
//         match size {
//             1 => write_u8(cpu, addr, val),
//             2 => write_u16(cpu, addr, val),
//             _ => write_u32(cpu, addr, val),
//         }
//         if let Some((wb_reg, wb_val)) = wb {
//             cpu.write_reg(wb_reg, wb_val);
//         }
//     }
// }

// fn store_dual_imm(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let ops: Vec<_> = data.operands().collect();
//     let rt = if let ArmOperandType::Reg(r) = ops[0].op_type {
//         data.resolve_reg(r)
//     } else {
//         return;
//     };
//     let rt2 = if let ArmOperandType::Reg(r) = ops[1].op_type {
//         data.resolve_reg(r)
//     } else {
//         return;
//     };

//     if let Some((addr, wb)) = resolve_addr_imm(data, &ops, 2, cpu) {
//         let val1 = cpu.read_reg(rt);
//         let val2 = cpu.read_reg(rt2);
//         write_u32(cpu, addr, val1);
//         write_u32(cpu, addr.wrapping_add(4), val2);
//         if let Some((wb_reg, wb_val)) = wb {
//             cpu.write_reg(wb_reg, wb_val);
//         }
//     }
// }

// fn load_reg(cpu: &mut dyn CpuContext, data: &ArmOpcode, size: u32, signed: bool) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let ops: Vec<_> = data.operands().collect();
//     let rt = if let ArmOperandType::Reg(r) = ops[0].op_type {
//         data.resolve_reg(r)
//     } else {
//         return;
//     };

//     if let Some((addr, wb)) = resolve_addr_reg(data, &ops, 1, cpu) {
//         let val = match size {
//             1 => {
//                 let v = read_u8(cpu, addr);
//                 if signed { (v as i8) as i32 as u32 } else { v }
//             }
//             2 => {
//                 let v = read_u16(cpu, addr);
//                 if signed { (v as i16) as i32 as u32 } else { v }
//             }
//             _ => read_u32(cpu, addr),
//         };
//         cpu.write_reg(rt, val);
//         if let Some((wb_reg, wb_val)) = wb {
//             cpu.write_reg(wb_reg, wb_val);
//         }
//     }
// }

// fn load_dual_reg(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let ops: Vec<_> = data.operands().collect();
//     let rt = if let ArmOperandType::Reg(r) = ops[0].op_type {
//         data.resolve_reg(r)
//     } else {
//         return;
//     };
//     let rt2 = if let ArmOperandType::Reg(r) = ops[1].op_type {
//         data.resolve_reg(r)
//     } else {
//         return;
//     };

//     if let Some((addr, wb)) = resolve_addr_reg(data, &ops, 2, cpu) {
//         let val1 = read_u32(cpu, addr);
//         let val2 = read_u32(cpu, addr.wrapping_add(4));
//         cpu.write_reg(rt, val1);
//         cpu.write_reg(rt2, val2);
//         if let Some((wb_reg, wb_val)) = wb {
//             cpu.write_reg(wb_reg, wb_val);
//         }
//     }
// }

// fn store_reg(cpu: &mut dyn CpuContext, data: &ArmOpcode, size: u32) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let ops: Vec<_> = data.operands().collect();
//     let rt = if let ArmOperandType::Reg(r) = ops[0].op_type {
//         data.resolve_reg(r)
//     } else {
//         return;
//     };
//     let val = cpu.read_reg(rt);

//     if let Some((addr, wb)) = resolve_addr_reg(data, &ops, 1, cpu) {
//         match size {
//             1 => write_u8(cpu, addr, val),
//             2 => write_u16(cpu, addr, val),
//             _ => write_u32(cpu, addr, val),
//         }
//         if let Some((wb_reg, wb_val)) = wb {
//             cpu.write_reg(wb_reg, wb_val);
//         }
//     }
// }

// fn store_dual_reg(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let ops: Vec<_> = data.operands().collect();
//     let rt = if let ArmOperandType::Reg(r) = ops[0].op_type {
//         data.resolve_reg(r)
//     } else {
//         return;
//     };
//     let rt2 = if let ArmOperandType::Reg(r) = ops[1].op_type {
//         data.resolve_reg(r)
//     } else {
//         return;
//     };

//     if let Some((addr, wb)) = resolve_addr_reg(data, &ops, 2, cpu) {
//         let val1 = cpu.read_reg(rt);
//         let val2 = cpu.read_reg(rt2);
//         write_u32(cpu, addr, val1);
//         write_u32(cpu, addr.wrapping_add(4), val2);
//         if let Some((wb_reg, wb_val)) = wb {
//             cpu.write_reg(wb_reg, wb_val);
//         }
//     }
// }

// fn load_lit(cpu: &mut dyn CpuContext, data: &ArmOpcode, size: u32, signed: bool) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let ops: Vec<_> = data.operands().collect();
//     let rt = if let ArmOperandType::Reg(r) = ops[0].op_type {
//         data.resolve_reg(r)
//     } else {
//         return;
//     };

//     if let Some(addr) = resolve_addr_lit(data, &ops, 1, cpu) {
//         let val = match size {
//             1 => {
//                 let v = read_u8(cpu, addr);
//                 if signed { (v as i8) as i32 as u32 } else { v }
//             }
//             2 => {
//                 let v = read_u16(cpu, addr);
//                 if signed { (v as i16) as i32 as u32 } else { v }
//             }
//             _ => read_u32(cpu, addr),
//         };
//         cpu.write_reg(rt, val);
//     }
// }

// fn load_dual_lit(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let ops: Vec<_> = data.operands().collect();
//     let rt = if let ArmOperandType::Reg(r) = ops[0].op_type {
//         data.resolve_reg(r)
//     } else {
//         return;
//     };
//     let rt2 = if let ArmOperandType::Reg(r) = ops[1].op_type {
//         data.resolve_reg(r)
//     } else {
//         return;
//     };

//     if let Some(addr) = resolve_addr_lit(data, &ops, 2, cpu) {
//         let val1 = read_u32(cpu, addr);
//         let val2 = read_u32(cpu, addr.wrapping_add(4));
//         cpu.write_reg(rt, val1);
//         cpu.write_reg(rt2, val2);
//     }
// }

// // --- Struct Definitions ---

// // Immediate Load
// pub struct LoadImm;
// impl Executable for LoadImm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         load_imm(cpu, data, 4, false);
//     }
// }
// pub struct LoadByteImm;
// impl Executable for LoadByteImm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         load_imm(cpu, data, 1, false);
//     }
// }
// pub struct LoadHalfImm;
// impl Executable for LoadHalfImm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         load_imm(cpu, data, 2, false);
//     }
// }
// pub struct LoadSignedByteImm;
// impl Executable for LoadSignedByteImm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         load_imm(cpu, data, 1, true);
//     }
// }
// pub struct LoadSignedHalfImm;
// impl Executable for LoadSignedHalfImm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         load_imm(cpu, data, 2, true);
//     }
// }
// pub struct LoadDoubleImm;
// impl Executable for LoadDoubleImm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         load_dual_imm(cpu, data);
//     }
// }

// // Immediate Store
// pub struct StoreImm;
// impl Executable for StoreImm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         store_imm(cpu, data, 4);
//     }
// }
// pub struct StoreByteImm;
// impl Executable for StoreByteImm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         store_imm(cpu, data, 1);
//     }
// }
// pub struct StoreHalfImm;
// impl Executable for StoreHalfImm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         store_imm(cpu, data, 2);
//     }
// }
// pub struct StoreDoubleImm;
// impl Executable for StoreDoubleImm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         store_dual_imm(cpu, data);
//     }
// }

// // Register Load
// pub struct LoadReg;
// impl Executable for LoadReg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         load_reg(cpu, data, 4, false);
//     }
// }
// pub struct LoadByteReg;
// impl Executable for LoadByteReg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         load_reg(cpu, data, 1, false);
//     }
// }
// pub struct LoadHalfReg;
// impl Executable for LoadHalfReg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         load_reg(cpu, data, 2, false);
//     }
// }
// pub struct LoadSignedByteReg;
// impl Executable for LoadSignedByteReg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         load_reg(cpu, data, 1, true);
//     }
// }
// pub struct LoadSignedHalfReg;
// impl Executable for LoadSignedHalfReg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         load_reg(cpu, data, 2, true);
//     }
// }
// pub struct LoadDoubleReg;
// impl Executable for LoadDoubleReg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         load_dual_reg(cpu, data);
//     }
// }

// // Register Store
// pub struct StoreReg;
// impl Executable for StoreReg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         store_reg(cpu, data, 4);
//     }
// }
// pub struct StoreByteReg;
// impl Executable for StoreByteReg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         store_reg(cpu, data, 1);
//     }
// }
// pub struct StoreHalfReg;
// impl Executable for StoreHalfReg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         store_reg(cpu, data, 2);
//     }
// }
// pub struct StoreDoubleReg;
// impl Executable for StoreDoubleReg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         store_dual_reg(cpu, data);
//     }
// }

// // Literal Load
// pub struct LoadLiteral;
// impl Executable for LoadLiteral {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         load_lit(cpu, data, 4, false);
//     }
// }
// pub struct LoadByteLiteral;
// impl Executable for LoadByteLiteral {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         load_lit(cpu, data, 1, false);
//     }
// }
// pub struct LoadHalfLiteral;
// impl Executable for LoadHalfLiteral {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         load_lit(cpu, data, 2, false);
//     }
// }
// pub struct LoadSignedByteLiteral;
// impl Executable for LoadSignedByteLiteral {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         load_lit(cpu, data, 1, true);
//     }
// }
// pub struct LoadSignedHalfLiteral;
// impl Executable for LoadSignedHalfLiteral {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         load_lit(cpu, data, 2, true);
//     }
// }
// pub struct LoadDoubleLiteral;
// impl Executable for LoadDoubleLiteral {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         load_dual_lit(cpu, data);
//     }
// }
