use capstone::arch::arm::ArmInsn;
use cranelift::codegen::ir::condcodes::IntCC;
use cranelift::prelude::*;

use crate::jit_engine::clif::instructions::{
    InsDef, check_cc, emit_read_reg, emit_resolve_simple_op2, emit_size_value, emit_write_reg,
};
use crate::jit_engine::engine::LoweringContext;
use crate::jit_engine::table::JitInstruction;

macro_rules! define_def {
    ($struct_name:ident, $static_name:ident, $insn:ident, $mnemonic:literal, $emit:ident) => {
        pub(crate) static $static_name: $struct_name = $struct_name;

        pub(crate) struct $struct_name;

        impl InsDef for $struct_name {
            fn insn_id(&self) -> u32 {
                ArmInsn::$insn as u32
            }

            fn mnemonic(&self) -> &'static str {
                $mnemonic
            }

            fn execute(&self, lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
                $emit(lowering, insn);
            }
        }
    };
}

define_def!(BDef, B_DEF, ARM_INS_B, "B", emit_b);
define_def!(BlDef, BL_DEF, ARM_INS_BL, "BL", emit_bl);
define_def!(BxDef, BX_DEF, ARM_INS_BX, "BX", emit_bx);
define_def!(BlxDef, BLX_DEF, ARM_INS_BLX, "BLX", emit_blx);
define_def!(CbzDef, CBZ_DEF, ARM_INS_CBZ, "CBZ", emit_cbz);
define_def!(CbnzDef, CBNZ_DEF, ARM_INS_CBNZ, "CBNZ", emit_cbnz);

pub(crate) fn find_def(insn_id: u32) -> Option<&'static dyn InsDef> {
    match insn_id {
        x if x == ArmInsn::ARM_INS_B as u32 => Some(&B_DEF),
        x if x == ArmInsn::ARM_INS_BL as u32 => Some(&BL_DEF),
        x if x == ArmInsn::ARM_INS_BX as u32 => Some(&BX_DEF),
        x if x == ArmInsn::ARM_INS_BLX as u32 => Some(&BLX_DEF),
        x if x == ArmInsn::ARM_INS_CBZ as u32 => Some(&CBZ_DEF),
        x if x == ArmInsn::ARM_INS_CBNZ as u32 => Some(&CBNZ_DEF),
        _ => None,
    }
}

fn emit_b(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    check_cc(lowering, insn);
    let target = emit_resolve_simple_op2(lowering);
    emit_write_reg(lowering, 15, target);
    let zero = lowering.iconst_u32(0);
    lowering.builder.ins().return_(&[zero]);
}

fn emit_bl(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    check_cc(lowering, insn);
    let target = emit_resolve_simple_op2(lowering);
    let pc = emit_read_reg(lowering, 15);
    let one = lowering.iconst_u32(1);
    let lr = lowering.builder.ins().bor(pc, one);
    emit_write_reg(lowering, 14, lr);
    emit_write_reg(lowering, 15, target);
    let zero = lowering.iconst_u32(0);
    lowering.builder.ins().return_(&[zero]);
}

fn emit_bx(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    check_cc(lowering, insn);
    let target = emit_resolve_simple_op2(lowering);
    let handled = lowering.call_value(
        lowering.helpers.try_exception_return,
        &[lowering.cpu_ptr, target],
    );

    let handled_block = lowering.builder.create_block();
    let continue_block = lowering.builder.create_block();
    lowering
        .builder
        .ins()
        .brif(handled, handled_block, &[], continue_block, &[]);

    lowering.builder.switch_to_block(handled_block);
    lowering.builder.seal_block(handled_block);
    let zero = lowering.iconst_u32(0);
    lowering.builder.ins().return_(&[zero]);

    lowering.builder.switch_to_block(continue_block);
    lowering.builder.seal_block(continue_block);
    let mask = lowering.iconst_u32(!1u32);
    let aligned = lowering.builder.ins().band(target, mask);
    emit_write_reg(lowering, 15, aligned);
    let zero = lowering.iconst_u32(0);
    lowering.builder.ins().return_(&[zero]);
}

fn emit_blx(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    check_cc(lowering, insn);
    let target = emit_resolve_simple_op2(lowering);
    let pc = emit_read_reg(lowering, 15);
    let delta = lowering.iconst_i32(insn.data.size() as i32 - 4);
    let return_addr = lowering.builder.ins().iadd(pc, delta);
    let one = lowering.iconst_u32(1);
    let lr = lowering.builder.ins().bor(return_addr, one);
    let mask = lowering.iconst_u32(!1u32);
    let aligned = lowering.builder.ins().band(target, mask);
    emit_write_reg(lowering, 14, lr);
    emit_write_reg(lowering, 15, aligned);
    let zero = lowering.iconst_u32(0);
    lowering.builder.ins().return_(&[zero]);
}

fn emit_cbz(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_compare_branch(lowering, insn, true);
}

fn emit_cbnz(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_compare_branch(lowering, insn, false);
}

fn emit_compare_branch(
    lowering: &mut LoweringContext<'_, '_>,
    insn: &JitInstruction<'_>,
    branch_on_zero: bool,
) {
    check_cc(lowering, insn);

    let value = emit_read_reg(lowering, insn.data.arm_operands.rn);
    let target = emit_resolve_simple_op2(lowering);
    let cond = if branch_on_zero {
        lowering.builder.ins().icmp_imm(IntCC::Equal, value, 0)
    } else {
        lowering.builder.ins().icmp_imm(IntCC::NotEqual, value, 0)
    };

    let taken_block = lowering.builder.create_block();
    let fallthrough_block = lowering.builder.create_block();
    lowering
        .builder
        .ins()
        .brif(cond, taken_block, &[], fallthrough_block, &[]);

    lowering.builder.switch_to_block(taken_block);
    lowering.builder.seal_block(taken_block);
    emit_write_reg(lowering, 15, target);
    let zero = lowering.iconst_u32(0);
    lowering.builder.ins().return_(&[zero]);

    lowering.builder.switch_to_block(fallthrough_block);
    lowering.builder.seal_block(fallthrough_block);
    let size = emit_size_value(lowering, insn);
    lowering.builder.ins().return_(&[size]);
}