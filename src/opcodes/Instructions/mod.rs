pub mod mov;
pub mod bit_field;
pub mod extend;
pub mod branch;
pub mod shift;
pub mod cmp;
pub mod bitop;
pub mod nop;
pub mod stack;
pub mod ldr;
pub mod str;
pub mod calculate;
pub mod compare_branch;
pub mod breakpoint;
pub mod adr;
pub mod movs;
pub mod hint;
pub mod tst;
pub mod ldm;
pub mod stm;
pub mod it;

#[cfg(test)]
mod tests {
	use super::*;
	use crate::context::CpuContext;
	use crate::cpu::Cpu;
	use crate::disassembler::disassemble_from_reset_handler;
	use crate::opcodes::opcode::{ArmOpcode, Opcode};
	use crate::peripheral::bus::Bus;
	use capstone::arch;
	use capstone::prelude::*;
	use std::collections::HashMap;
	use std::path::Path;
	use std::sync::OnceLock;
	use std::sync::Arc;
	use std::sync::atomic::AtomicU32;

	#[derive(Clone)]
	struct SampleInsn {
		bytes: Vec<u8>,
		address: u64,
	}

	static EXEC_SAMPLE_CACHE: OnceLock<HashMap<u32, SampleInsn>> = OnceLock::new();

	fn all_definition_groups() -> Vec<(&'static str, Vec<Opcode>)> {
		vec![
			("adr", adr::add_adr_def()),
			("bit_field", bit_field::add_bit_field_def()),
			("bitop", bitop::add_bitop_def()),
			("branch", branch::add_branch_def()),
			("breakpoint", breakpoint::add_breakpoint_def()),
			("calculate", calculate::add_calculate_def()),
			("cmp", cmp::add_cmp_def()),
			("compare_branch", compare_branch::add_compare_branch_def()),
			("extend", extend::add_extend_def()),
			("hint", hint::add_Hint_def()),
			("it", it::add_it_def()),
			("ldm", ldm::add_ldm_def()),
			("ldr", ldr::add_ldr_def()),
			("mov", mov::add_mov_def()),
			("movs", movs::add_movs_def()),
			("nop", nop::add_nop_def()),
			("shift", shift::addd_shift_def()),
			("stack", stack::add_stack_def()),
			("stm", stm::add_stm_def()),
			("str", str::add_str_def()),
			("tst", tst::add_tst_def()),
		]
	}

	fn assert_defs_valid(defs: &[Opcode], label: &str) {
		assert!(!defs.is_empty(), "{label} definitions should not be empty");
		for op in defs {
			assert!(op.insnid != 0, "{label} has invalid insnid");
			assert!(!op.name.trim().is_empty(), "{label} has empty mnemonic");
			assert!(op.length > 0, "{label} has invalid length");
		}
	}

	fn make_cpu() -> Cpu {
		Cpu::new(Arc::new(AtomicU32::new(8_000_000)), 1, Bus::new(), Bus::new())
	}

	fn seed_cpu_state(cpu: &mut Cpu) {
		for reg in 0..13u32 {
			cpu.write_reg(reg, 0x0000_0100u32 + reg * 4);
		}
		cpu.write_sp(0x2000_1A00);
		cpu.write_lr(0x0800_0101);
		cpu.write_pc(0x0800_0100);
		cpu.write_apsr(1u32 << 29);

		for i in 0..256u32 {
			cpu.write_mem(0x0000_0000 + i * 4, 0x0101_0000u32 ^ i);
		}

		for i in 0..256u32 {
			cpu.write_mem(0x2000_0800 + i * 4, 0x1111_0000u32 ^ i);
		}

		for i in 0..128u32 {
			cpu.write_mem(0x0800_0000 + i * 4, 0x0800_0001u32 + i * 2);
		}
	}

	fn make_thumb_capstone() -> Capstone {
		Capstone::new()
			.arm()
			.mode(arch::arm::ArchMode::Thumb)
			.extra_mode([arch::arm::ArchExtraMode::MClass].iter().copied())
			.detail(true)
			.build()
			.expect("failed to create capstone for execute test")
	}

	fn collect_execute_samples_from_axf() -> HashMap<u32, SampleInsn> {
		let axf_candidates = ["uart_loop.axf", "uart_helloworld.axf", "timer.axf", "io_toggle.axf"];
		let mut samples = HashMap::new();

		for axf in axf_candidates {
			if !Path::new(axf).exists() {
				continue;
			}

			let output = format!("target/exec_sample_{}.asm", axf.replace('.', "_"));
			let Ok((_result, cs, code_segments, _dcw_data, _sp, _rh, _rha)) =
				disassemble_from_reset_handler(axf, &output)
			else {
				continue;
			};

			for (addr, bytes) in &code_segments {
				let Ok(insns) = cs.disasm_all(bytes, *addr) else {
					continue;
				};

				for insn in insns.iter() {
					samples.entry(insn.id().0).or_insert_with(|| SampleInsn {
						bytes: insn.bytes().to_vec(),
						address: insn.address(),
					});
				}
			}
		}

		samples
	}

	fn execute_sample_cache() -> &'static HashMap<u32, SampleInsn> {
		EXEC_SAMPLE_CACHE.get_or_init(collect_execute_samples_from_axf)
	}

	fn fallback_sample_for_definition(name: &str) -> Option<SampleInsn> {
		let bytes: &'static [u8] = match name {
			"ADD" => &[0x01, 0x30],
			"ADC" => &[0x48, 0x41],
			"ADR" => &[0x00, 0xA0],
			"AND" => &[0x08, 0x40],
			"ASR" => &[0x08, 0x41],
			"BIC" => &[0x88, 0x43],
			"BKPT" => &[0x00, 0xBE],
			"B" => &[0xFE, 0xE7],
			"BL" => &[0x00, 0xF0, 0x18, 0xF8],
			"BLX" => &[0x80, 0x47],
			"BX" => &[0x00, 0x47],
			"CBNZ" => &[0x00, 0xB9],
			"CBZ" => &[0x00, 0xB1],
			"CMP" => &[0x88, 0x42],
			"CMN" => &[0xC8, 0x42],
			"EOR" => &[0x48, 0x40],
			"Hint" => &[0x00, 0xBF],
			"IT" => &[0x08, 0xBF],
			"LDM" => &[0x01, 0xC9],
			"LDR" => &[0x00, 0x48],
			"LDRB" => &[0x88, 0x5C],
			"LDRSB" => &[0x88, 0x56],
			"LDRH" => &[0x88, 0x5A],
			"LDRSH" => &[0x88, 0x5E],
			"LSL" => &[0x88, 0x40],
			"LSR" => &[0xC8, 0x40],
			"MOV" => &[0x01, 0x20],
			"MOVS" => &[0x01, 0x20],
			"MUL" => &[0x48, 0x43],
			"MVN" => &[0xC8, 0x43],
			"NOP" => &[0x00, 0xBF],
			"ORN" => &[0x48, 0x40],
			"ORR" => &[0x08, 0x43],
			"POP" => &[0x01, 0xBC],
			"PUSH" => &[0x01, 0xB4],
			"ROR" => &[0xC8, 0x41],
			"RSB" => &[0x48, 0x42],
			"RRX" => &[0xC8, 0x41],
			"SBC" => &[0x88, 0x41],
			"STM" => &[0x01, 0xC1],
			"STR" => &[0x88, 0x50],
			"STRB" => &[0x88, 0x54],
			"STRH" => &[0x88, 0x52],
			"SUB" => &[0x01, 0x38],
			"TEQ" => &[0x48, 0x40],
			"TST" => &[0x08, 0x42],
			"UXTH" => &[0x88, 0xB2],
			"UXTB" => &[0xC8, 0xB2],
			_ => return None,
		};

		Some(SampleInsn {
			bytes: bytes.to_vec(),
			address: 0x0800_0100,
		})
	}

	fn sample_for_definition(def: &Opcode) -> SampleInsn {
		if let Some(sample) = execute_sample_cache().get(&def.insnid) {
			return sample.clone();
		}

		if let Some(sample) = fallback_sample_for_definition(def.name.as_str()) {
			return sample;
		}

		panic!(
			"missing executable sample for definition {} (insnid={})",
			def.name, def.insnid
		);
	}

	fn execute_definition_once(group: &str, def: &Opcode) {
		let sample = sample_for_definition(def);

		let cs = make_thumb_capstone();
		let insns = cs
			.disasm_all(&sample.bytes, sample.address)
			.unwrap_or_else(|e| panic!("failed to disasm sample for {group}/{}: {e}", def.name));
		let insn = insns
			.iter()
			.next()
			.unwrap_or_else(|| panic!("sample bytes produce no instruction for {group}/{}", def.name));

		let mut data = ArmOpcode::new(&cs, &insn)
			.unwrap_or_else(|| panic!("failed to build ArmOpcode for {group}/{}", def.name));
		let mut cpu = make_cpu();
		seed_cpu_state(&mut cpu);

		def.operand_resolver.resolve(&mut data);
		let ret = (def.exec)(&mut cpu, &data);

		assert!(
			ret == 0 || ret == data.size(),
			"unexpected return from {group}/{} execute: ret={ret}, size={} mnemonic={} opstr={}",
			def.name,
			data.size(),
			data.mnemonic(),
			data.op_str()
		);
	}

	#[test]
	fn all_instruction_definitions_are_buildable_and_valid() {
		let groups = all_definition_groups();
		for (name, defs) in &groups {
			assert_defs_valid(defs, name);
		}
	}

	#[test]
	fn every_execute_entrypoint_runs_once() {
		let groups = all_definition_groups();
		for (group, defs) in groups {
			for def in defs {
				execute_definition_once(group, &def);
			}
		}
	}
}