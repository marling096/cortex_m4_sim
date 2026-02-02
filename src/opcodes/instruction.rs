use crate::context::CpuContext;
use crate::opcodes::Instructions::adr::Adr_builder;
use crate::opcodes::Instructions::bitop::Bitop_builder;
use crate::opcodes::Instructions::branch::Branch_builder;
use crate::opcodes::Instructions::breakpoint::Breakpoint_builder;
use crate::opcodes::Instructions::calculate::Calculate_builder;
use crate::opcodes::Instructions::cmp::Cmp_builder;
use crate::opcodes::Instructions::compare_branch::Compare_branch_builder;
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
use crate::opcodes::opcode::{ArmOpcode, CycleInfo, Executable, Opcode, check_condition};

use capstone::arch::arm::{ArmInsn, ArmOperandType, ArmShift};
use std::collections::{BTreeMap, HashMap};

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
    pub table: BTreeMap<u32, Cpu_Instruction<'a>>,
}
impl<'a> Cpu_InstrTable<'a> {
    pub fn new() -> Cpu_InstrTable<'a> {
        Cpu_InstrTable { table: BTreeMap::new() }
    }
    pub fn add_instruction(&mut self, instr: Cpu_Instruction<'a>) {
        self.table.insert(instr.data.address(), instr);
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
            Box::new(Adr_builder),
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
