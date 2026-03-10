use capstone::arch::arm::{ArmInsn, ArmOperandType, ArmReg};
use cranelift::prelude::*;

use crate::jit_engine::clif::instructions::{InsDef, check_cc};
use crate::jit_engine::engine::LoweringContext;
use crate::jit_engine::table::JitInstruction;

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

	fn supports(&self, insn: &JitInstruction<'_>) -> bool {
		match insn.data.arm_operands.op2.as_ref().map(|op| &op.op_type) {
			Some(ArmOperandType::Imm(_))
			| Some(ArmOperandType::Mem(_))
			| Some(ArmOperandType::Reg(_)) => true,
			_ => false,
		}
	}

	fn execute(&self, lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
		emit_adr(lowering, insn);
	}
}

fn emit_adr(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
	check_cc(lowering, insn);

	let rd = lowering.iconst_u32(insn.data.arm_operands.rd);
	let target = emit_adr_target(lowering, insn);
	lowering.call_void(lowering.helpers.write_reg, &[lowering.cpu_ptr, rd, target]);

	let pc_update = if insn.data.arm_operands.rd == 15 {
		lowering.iconst_u32(0)
	} else {
		lowering.iconst_u32(insn.data.size())
	};
	lowering.builder.ins().return_(&[pc_update]);
}

fn emit_adr_target(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) -> Value {
	let pc_aligned = lowering.iconst_u32(insn.data.address().wrapping_add(4) & !0x3);

	match insn.data.arm_operands.op2.as_ref().map(|op| &op.op_type) {
		Some(ArmOperandType::Imm(imm)) => emit_add_signed(lowering, pc_aligned, i64::from(*imm)),
		Some(ArmOperandType::Mem(mem)) => {
			let base = if mem.base().0 == ArmReg::ARM_REG_PC as u16 {
				pc_aligned
			} else {
				let reg = lowering.iconst_u32(insn.data.resolve_reg(mem.base()));
				lowering.call_value(lowering.helpers.read_reg, &[lowering.cpu_ptr, reg])
			};
			emit_add_signed(lowering, base, i64::from(mem.disp()))
		}
		Some(ArmOperandType::Reg(reg)) => {
			let reg = lowering.iconst_u32(insn.data.resolve_reg(*reg));
			lowering.call_value(lowering.helpers.read_reg, &[lowering.cpu_ptr, reg])
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
