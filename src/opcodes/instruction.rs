use crate::opcodes::Instructions::adr::AdrBuilder;
use crate::opcodes::Instructions::bit_field::Bit_field_builder;
use crate::opcodes::Instructions::bitop::Bitop_builder;
use crate::opcodes::Instructions::branch::Branch_builder;
use crate::opcodes::Instructions::breakpoint::Breakpoint_builder;
use crate::opcodes::Instructions::calculate::Calculate_builder;
use crate::opcodes::Instructions::cmp::Cmp_builder;
use crate::opcodes::Instructions::compare_branch::Compare_branch_builder;
use crate::opcodes::Instructions::extend::Extend_builder;
use crate::opcodes::Instructions::hint::Hint_builder;
use crate::opcodes::Instructions::it::It_builder;
use crate::opcodes::Instructions::ldm::Ldm_builder;
use crate::opcodes::Instructions::ldr::Ldr_builder;
use crate::opcodes::Instructions::mov::Mov_builder;
use crate::opcodes::Instructions::movs::Movs_builder;
use crate::opcodes::Instructions::nop::Nop_builder;
use crate::opcodes::Instructions::shift::Shiift_builder;
use crate::opcodes::Instructions::stack::Stack_builder;
use crate::opcodes::Instructions::stm::Stm_builder;
use crate::opcodes::Instructions::str::Str_builder;
use crate::opcodes::Instructions::tst::Tst_builder;
use crate::opcodes::opcode::{ArmOpcode, Opcode};

use std::collections::HashMap;
use rustc_hash::FxHashMap;

pub struct Cpu_Instruction<'a> {
    pub op: Opcode,
    pub data: ArmOpcode<'a>,
}
impl<'a> Cpu_Instruction<'a> {
    pub fn new(op: Opcode, data: ArmOpcode<'a>) -> Cpu_Instruction<'a> {
        Cpu_Instruction { op, data }
    }
}

pub struct Cpu_InstrTable<'a> {
    pub table: FxHashMap<u32, Cpu_Instruction<'a>>,
    fast_table: Vec<Option<Cpu_Instruction<'a>>>,
    fast_base: u32,
}

impl<'a> Cpu_InstrTable<'a> {
    pub fn new() -> Cpu_InstrTable<'a> {
        Cpu_InstrTable {
            table: FxHashMap::default(),
            fast_table: Vec::new(),
            fast_base: 0,
        }
    }
    
    pub fn add_instruction(&mut self, mut instr: Cpu_Instruction<'a>) {
        instr.op.operand_resolver.resolve(&mut instr.data);
        if let Some(adjust_cycles) = instr.op.adjust_cycles {
            let operands: Vec<_> = instr.data.operands().collect();
            adjust_cycles(&mut instr.op.cycles, &operands);
        }
        self.table.insert(instr.data.address(), instr);
    }

    #[inline(always)]
    pub fn get(&self, addr: u32) -> Option<&Cpu_Instruction<'a>> {
        // 利用无符号数溢出特性合并判断：如果 addr < fast_base，减法结果会是一个巨大的 u32，
        // 肯定大于 fast_table.len()，从而一次检查涵盖上下界。
        let offset = (addr.wrapping_sub(self.fast_base)) >> 1;
        if (offset as usize) < self.fast_table.len() {
             // Safety: 已经在 if 中检查了边界，使用 get_unchecked 移除切片内部的二次检查
             // 对于高性能取指非常关键
             unsafe {
                 if let Some(ins) = self.fast_table.get_unchecked(offset as usize) {
                     return Some(ins);
                 }
             }
        }
        // Fallback to slow path (HashMap) - 仅在未命中 fast_table 时触发
        if self.table.is_empty() { None } else { self.table.get(&addr) }
    }

    pub fn optimize(&mut self) {
        if self.table.is_empty() {
            return;
        }
        
        // Find address range
        let mut min_addr = u32::MAX;
        let mut max_addr = 0;
        for k in self.table.keys() {
            if *k < min_addr { min_addr = *k; }
            if *k > max_addr { max_addr = *k; }
        }

        // Check if optimization is feasible (limit to 16MB range)
        let range = max_addr - min_addr;
        if range > 16 * 1024 * 1024 {
            println!("Address range too large for fast lookup: {} bytes. Optimization skipped.", range);
            return;
        }
        
        // Determine vector size. +2 to cover the last instruction fully
        let size = (range / 2) as usize + 2; 
        
        let mut fast_table = Vec::with_capacity(size);
        for _ in 0..size {
            fast_table.push(None);
        }
        
        self.fast_base = min_addr;

        // Move instructions from HashMap to Vec
        // We iterate over execution order (keys) to be deterministic, though not strictly needed
        let mut keys: Vec<u32> = self.table.keys().cloned().collect();
        keys.sort();

        for addr in keys {
            if let Some(instr) = self.table.remove(&addr) {
                let offset = ((addr - self.fast_base) >> 1) as usize;
                if offset < fast_table.len() {
                    fast_table[offset] = Some(instr);
                } else {
                    // Should not happen, put back
                    self.table.insert(addr, instr);
                }
            }
        }

        self.fast_table = fast_table;
        println!("Instruction table optimized: Base=0x{:08X}, Size={}, Entries moved to fast lookup.", self.fast_base, size);
    }
}


pub struct OpcodeTable {
    pub table: HashMap<u16, Vec<Opcode>>,
}

impl OpcodeTable {
    pub fn new() -> OpcodeTable {
        let mut table = HashMap::new();
        OpcodeTable::Instructions_builder(&mut table);
        OpcodeTable { table }
    }

    pub fn get_table(&self) -> &HashMap<u16, Vec<Opcode>> {
        &self.table
    }

    fn Instructions_builder(table: &mut HashMap<u16, Vec<Opcode>>) {
        let builders: Vec<Box<dyn InstrBuilder>> = vec![
            Box::new(Bit_field_builder),
            Box::new(Bitop_builder),
            Box::new(Branch_builder),
            Box::new(Mov_builder),
            Box::new(Shiift_builder),
            Box::new(Cmp_builder),
            Box::new(Compare_branch_builder),
            Box::new(Extend_builder),
            Box::new(Nop_builder),
            Box::new(Stack_builder),
            Box::new(Ldr_builder),
            Box::new(Str_builder),
            Box::new(Calculate_builder),
            Box::new(Breakpoint_builder),
            Box::new(AdrBuilder),
            Box::new(Movs_builder),
            Box::new(Hint_builder),
            Box::new(Ldm_builder),
            Box::new(Stm_builder),
            Box::new(It_builder),
            Box::new(Tst_builder),
        ];

        for b in builders.iter() {
            for ins in b.build().into_iter() {
                let key = ins.insnid as u16;
                table.entry(key).or_insert_with(Vec::new).push(ins);
            }
        }
    }
}

pub trait InstrBuilder {
    fn build(&self) -> Vec<Opcode>;
}
