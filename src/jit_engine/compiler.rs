use crate::jit_engine::engine::{JitBlockFn, JitEngine, JitError};
use crate::jit_engine::table::JitBlockTable;

pub struct JitInstructionCompiler {
    engine: JitEngine,
}

impl JitInstructionCompiler {
    pub fn new() -> Result<Self, JitError> {
        Ok(Self {
            engine: JitEngine::new()?,
        })
    }

    pub fn compile_table<'a>(
        &mut self,
        table: &JitBlockTable<'a>,
    ) -> Result<Vec<(u32, JitBlockFn)>, JitError> {
        self.engine.compile_table(table)
    }

    pub fn compiled_entry(&self, pc: u32) -> Option<JitBlockFn> {
        self.engine.compiled_entry(pc)
    }

    pub fn compiled_entries(&self) -> Vec<(u32, JitBlockFn)> {
        self.engine.compiled_entries()
    }
}

#[cfg(test)]
mod tests {
    use capstone::arch;
    use capstone::prelude::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicU32;

    use super::*;
    use crate::context::CpuContext;
    use crate::cpu::Cpu;
    use crate::jit_engine::table::JitBlockTableBuilder;
    use crate::opcodes::instruction::OpcodeTable;
    use crate::peripheral::bus::Bus;
    use crate::peripheral::nvic::Nvic;
    use crate::peripheral::scb::Scb;
    use crate::peripheral::systick::SysTick;

    fn build_thumb_capstone() -> Capstone {
        Capstone::new()
            .arm()
            .mode(arch::arm::ArchMode::Thumb)
            .extra_mode([arch::arm::ArchExtraMode::MClass].iter().copied())
            .detail(true)
            .build()
            .expect("failed to create capstone")
    }

    fn build_cpu() -> Cpu {
        let mut ppb = Bus::new();
        ppb.register_peripheral(Box::new(SysTick::new(0xE000_E010, 0xE000_E01F)));
        ppb.register_peripheral(Box::new(Nvic::new(0xE000_E100, 0xE000_E4EF)));
        ppb.register_peripheral(Box::new(Scb::new(0xE000_ED00, 0xE000_ED3C)));

        Cpu::new(Arc::new(AtomicU32::new(8_000_000)), 1, Bus::new(), ppb)
    }

    #[test]
    fn jit_instruction_compiler_compiles_new_table() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x08, 0x68, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let opcode_table = OpcodeTable::new();
        let table = JitBlockTableBuilder::build_from_disassembly(
            &opcode_table,
            &cs,
            insns.iter(),
        )
        .expect("failed to build jit instruction table");

        let mut compiler = JitInstructionCompiler::new().expect("failed to create jit compiler");
        let compiled = compiler
            .compile_table(&table)
            .expect("failed to compile table");

        assert_eq!(compiled.len(), 1);
        assert!(compiler.compiled_entry(0x0800_0000).is_some());
        assert!(compiler.compiled_entry(0x0800_0002).is_none());

        let mut cpu = build_cpu();
        cpu.write_reg(1, 0x2000_0000);
        cpu.write_mem(0x2000_0000, 0x1122_3344);
        let entry = compiler
            .compiled_entry(0x0800_0000)
            .expect("missing compiled entry");
        let cycles = unsafe {
            entry(
                &mut cpu as *mut Cpu,
                table
                    .get(0x0800_0000)
                    .expect("missing table entry") as *const _ as *const (),
            )
        };

        assert_eq!(cycles, 3);
        assert_eq!(cpu.next_pc, 0x0800_0004);
        assert_eq!(cpu.read_reg(0), 0x1122_3344);
    }

    #[test]
    fn jit_instruction_compiler_executes_adr() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x00, 0xA0], 0x0800_0000)
            .expect("failed to disassemble");
        let opcode_table = OpcodeTable::new();
        let table = JitBlockTableBuilder::build_from_disassembly(
            &opcode_table,
            &cs,
            insns.iter(),
        )
        .expect("failed to build jit instruction table");

        let mut compiler = JitInstructionCompiler::new().expect("failed to create jit compiler");
        compiler
            .compile_table(&table)
            .expect("failed to compile table");

        let mut cpu = build_cpu();
        let entry = compiler
            .compiled_entry(0x0800_0000)
            .expect("missing compiled entry");
        let cycles = unsafe {
            entry(
                &mut cpu as *mut Cpu,
                table
                    .get(0x0800_0000)
                    .expect("missing table entry") as *const _ as *const (),
            )
        };

        assert_eq!(cycles, 1);
        assert_eq!(cpu.next_pc, 0x0800_0002);
        assert_eq!(cpu.read_reg(0), 0x0800_0004);
    }
}