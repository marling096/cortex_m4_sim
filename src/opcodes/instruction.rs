use crate::context::CpuContext;
use crate::opcodes::Instructions::bitop::Bitop_builder;
use crate::opcodes::Instructions::branch::Branch_builder;
use crate::opcodes::Instructions::mov::Mov_builder;
use crate::opcodes::Instructions::shift::Shiift_builder;
use crate::opcodes::Instructions::cmp::Cmp_builder;
use crate::opcodes::Instructions::compare_branch::Compare_branch_builder;
use crate::opcodes::Instructions::nop::Nop_builder;
use crate::opcodes::Instructions::stack::Stack_builder;
use crate::opcodes::Instructions::ldr::Ldr_builder;
use crate::opcodes::Instructions::str::Str_builder;
use crate::opcodes::Instructions::calculate::Calculate_builder;
use crate::opcodes::Instructions::breakpoint::Breakpoint_builder;
use crate::opcodes::opcode::{ArmInstruction, CycleInfo, Executable, Instruction, check_condition};

use capstone::arch::arm::{ArmInsn, ArmOperandType, ArmShift};
use std::collections::HashMap;
pub struct InstrTable {
    pub table: HashMap<u16, Vec<Instruction>>,
}

impl InstrTable {
    pub fn new() -> Self {
        let mut table = HashMap::new();
        InstrTable::instructions_builder(&mut table);
        InstrTable { table }
    }

    pub fn instructions_builder(table: &mut HashMap<u16, Vec<Instruction>>) {
        let builders: Vec<Box<dyn InstrBuilder>> = vec![
            Box::new(Bitop_builder),
            Box::new(Branch_builder),
            Box::new(Mov_builder),
            Box::new(Shiift_builder),
            Box::new(Cmp_builder),
            Box::new(Compare_branch_builder),
            Box::new(Nop_builder),
            Box::new(Stack_builder),
            Box::new(Ldr_builder),
            Box::new(Str_builder),
            Box::new(Calculate_builder),
            Box::new(Breakpoint_builder),
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
    fn build(&self) -> Vec<Instruction>;
}
