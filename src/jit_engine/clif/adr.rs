use crate::arch::ArmInsn;
use cranelift::prelude::*;

use crate::jit_engine::clif::instructions::{
	InsDef, emit_pc_update_for_rd, emit_read_reg, emit_write_reg, with_cc_pure,
};
use crate::jit_engine::engine::LoweringContext;
use crate::jit_engine::table::JitInstruction;
use crate::opcodes::decoded::{DecodedOperandKind, is_pc_reg};

pub(crate) static ADR_DEF: AdrDef = AdrDef;

pub(crate) fn find_def(insn_id: u32) -> Option<&'static dyn InsDef> {
	match insn_id {
		x if x == ArmInsn::ARM_INS_ADR as u32 => Some(&ADR_DEF),
		_ => None,
	}
}

pub(crate) struct AdrDef;

impl InsDef for AdrDef {
	fn insn_id(&self) -> u32 {
		ArmInsn::ARM_INS_ADR as u32
	}

	fn mnemonic(&self) -> &'static str {
		"ADR"
	}

	fn supports(&self, insn: &JitInstruction) -> bool {
		match insn.data.arm_operands.op2.as_ref().map(|op| &op.op_type) {
			Some(DecodedOperandKind::Imm(_))
			| Some(DecodedOperandKind::Mem(_))
			| Some(DecodedOperandKind::Reg(_)) => true,
			_ => false,
		}
	}

	fn execute(&self, lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction) {
		emit_adr(lowering, insn)
	}
}

fn emit_adr(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction) {
	with_cc_pure(lowering, insn, |lowering| {
		let rd = insn.data.arm_operands.rd;
		let target = emit_adr_target(lowering, insn);
		emit_write_reg(lowering, rd, target);
		emit_pc_update_for_rd(lowering, insn, rd);
	})
}

fn emit_adr_target(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction) -> Value {
	let pc_aligned = lowering.iconst_u32(insn.data.address().wrapping_add(4) & !0x3);

	match insn.data.arm_operands.op2.as_ref().map(|op| &op.op_type) {
		Some(DecodedOperandKind::Imm(imm)) => emit_add_signed(lowering, pc_aligned, i64::from(*imm)),
		Some(DecodedOperandKind::Mem(mem)) => {
			let base = if is_pc_reg(mem.base) {
				pc_aligned
			} else {
				emit_read_reg(lowering, mem.base)
			};
			emit_add_signed(lowering, base, i64::from(mem.disp))
		}
		Some(DecodedOperandKind::Reg(reg)) => {
			emit_read_reg(lowering, *reg)
		}
		_ => lowering.iconst_u32(0),
	}
}

fn emit_add_signed(lowering: &mut LoweringContext<'_, '_>, base: Value, offset: i64) -> Value {
	if offset == 0 {
		return base;
	}

	let offset_value = lowering.builder.ins().iconst(types::I32, offset);
	lowering.builder.ins().iadd(base, offset_value)
}
