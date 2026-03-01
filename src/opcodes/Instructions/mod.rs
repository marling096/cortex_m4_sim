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

	fn assert_defs_valid(defs: &[crate::opcodes::opcode::Opcode], label: &str) {
		assert!(!defs.is_empty(), "{label} definitions should not be empty");
		for op in defs {
			assert!(op.insnid != 0, "{label} has invalid insnid");
			assert!(!op.name.trim().is_empty(), "{label} has empty mnemonic");
			assert!(op.length > 0, "{label} has invalid length");
		}
	}

	#[test]
	fn all_instruction_definitions_are_buildable_and_valid() {
        let groups: [(&str, Vec<crate::opcodes::opcode::Opcode>); 21] = [
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
		];

		for (name, defs) in &groups {
			assert_defs_valid(defs, name);
		}
	}
}