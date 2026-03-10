use capstone::arch::arm::{ArmInsn, ArmOperandType};
use cranelift::codegen::ir::condcodes::IntCC;
use cranelift::prelude::*;

use crate::jit_engine::clif::instructions::{
    InsDef, check_cc, emit_bool_to_u32, emit_compute_shift, emit_current_carry,
    emit_read_reg, emit_resolve_op2, emit_return_for_rd, emit_return_size,
    emit_update_apsr_nz, emit_update_apsr_nzc, emit_update_apsr_nzcv,
    emit_write_reg,
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

define_def!(UbfxDef, UBFX_DEF, ARM_INS_UBFX, "UBFX", emit_ubfx);
define_def!(UxtbDef, UXTB_DEF, ARM_INS_UXTB, "UXTB", emit_uxtb);
define_def!(UxthDef, UXTH_DEF, ARM_INS_UXTH, "UXTH", emit_uxth);
define_def!(CmpDef, CMP_DEF, ARM_INS_CMP, "CMP", emit_cmp);
define_def!(CmnDef, CMN_DEF, ARM_INS_CMN, "CMN", emit_cmn);
define_def!(TstDef, TST_DEF, ARM_INS_TST, "TST", emit_tst);
define_def!(TeqDef, TEQ_DEF, ARM_INS_TEQ, "TEQ", emit_teq);
define_def!(MovDef, MOV_DEF, ARM_INS_MOV, "MOV", emit_mov);
define_def!(MvnDef, MVN_DEF, ARM_INS_MVN, "MVN", emit_mvn);
define_def!(MovsDef, MOVS_DEF, ARM_INS_MOVS, "MOVS", emit_movs);
define_def!(AndDef, AND_DEF, ARM_INS_AND, "AND", emit_and);
define_def!(OrrDef, ORR_DEF, ARM_INS_ORR, "ORR", emit_orr);
define_def!(EorDef, EOR_DEF, ARM_INS_EOR, "EOR", emit_eor);
define_def!(BicDef, BIC_DEF, ARM_INS_BIC, "BIC", emit_bic);
define_def!(OrnDef, ORN_DEF, ARM_INS_ORN, "ORN", emit_orn);
define_def!(AddDef, ADD_DEF, ARM_INS_ADD, "ADD", emit_add);
define_def!(AdcDef, ADC_DEF, ARM_INS_ADC, "ADC", emit_adc);
define_def!(SubDef, SUB_DEF, ARM_INS_SUB, "SUB", emit_sub);
define_def!(SbcDef, SBC_DEF, ARM_INS_SBC, "SBC", emit_sbc);
define_def!(RsbDef, RSB_DEF, ARM_INS_RSB, "RSB", emit_rsb);
define_def!(MulDef, MUL_DEF, ARM_INS_MUL, "MUL", emit_mul);
define_def!(UdivDef, UDIV_DEF, ARM_INS_UDIV, "UDIV", emit_udiv);
define_def!(MlsDef, MLS_DEF, ARM_INS_MLS, "MLS", emit_mls);
define_def!(AsrDef, ASR_DEF, ARM_INS_ASR, "ASR", emit_shift);
define_def!(LslDef, LSL_DEF, ARM_INS_LSL, "LSL", emit_shift);
define_def!(LsrDef, LSR_DEF, ARM_INS_LSR, "LSR", emit_shift);
define_def!(RorDef, ROR_DEF, ARM_INS_ROR, "ROR", emit_shift);
define_def!(RrxDef, RRX_DEF, ARM_INS_RRX, "RRX", emit_shift);

pub(crate) fn find_def(insn_id: u32) -> Option<&'static dyn InsDef> {
    match insn_id {
        x if x == ArmInsn::ARM_INS_UBFX as u32 => Some(&UBFX_DEF),
        x if x == ArmInsn::ARM_INS_UXTB as u32 => Some(&UXTB_DEF),
        x if x == ArmInsn::ARM_INS_UXTH as u32 => Some(&UXTH_DEF),
        x if x == ArmInsn::ARM_INS_CMP as u32 => Some(&CMP_DEF),
        x if x == ArmInsn::ARM_INS_CMN as u32 => Some(&CMN_DEF),
        x if x == ArmInsn::ARM_INS_TST as u32 => Some(&TST_DEF),
        x if x == ArmInsn::ARM_INS_TEQ as u32 => Some(&TEQ_DEF),
        x if x == ArmInsn::ARM_INS_MOV as u32 => Some(&MOV_DEF),
        x if x == ArmInsn::ARM_INS_MVN as u32 => Some(&MVN_DEF),
        x if x == ArmInsn::ARM_INS_MOVS as u32 => Some(&MOVS_DEF),
        x if x == ArmInsn::ARM_INS_AND as u32 => Some(&AND_DEF),
        x if x == ArmInsn::ARM_INS_ORR as u32 => Some(&ORR_DEF),
        x if x == ArmInsn::ARM_INS_EOR as u32 => Some(&EOR_DEF),
        x if x == ArmInsn::ARM_INS_BIC as u32 => Some(&BIC_DEF),
        x if x == ArmInsn::ARM_INS_ORN as u32 => Some(&ORN_DEF),
        x if x == ArmInsn::ARM_INS_ADD as u32 => Some(&ADD_DEF),
        x if x == ArmInsn::ARM_INS_ADC as u32 => Some(&ADC_DEF),
        x if x == ArmInsn::ARM_INS_SUB as u32 => Some(&SUB_DEF),
        x if x == ArmInsn::ARM_INS_SBC as u32 => Some(&SBC_DEF),
        x if x == ArmInsn::ARM_INS_RSB as u32 => Some(&RSB_DEF),
        x if x == ArmInsn::ARM_INS_MUL as u32 => Some(&MUL_DEF),
        x if x == ArmInsn::ARM_INS_UDIV as u32 => Some(&UDIV_DEF),
        x if x == ArmInsn::ARM_INS_MLS as u32 => Some(&MLS_DEF),
        x if x == ArmInsn::ARM_INS_ASR as u32 => Some(&ASR_DEF),
        x if x == ArmInsn::ARM_INS_LSL as u32 => Some(&LSL_DEF),
        x if x == ArmInsn::ARM_INS_LSR as u32 => Some(&LSR_DEF),
        x if x == ArmInsn::ARM_INS_ROR as u32 => Some(&ROR_DEF),
        x if x == ArmInsn::ARM_INS_RRX as u32 => Some(&RRX_DEF),
        _ => None,
    }
}

enum LogicOp {
    And,
    Orr,
    Eor,
    Bic,
    Orn,
}

enum CompareOp {
    Cmp,
    Cmn,
}

enum TestOp {
    Tst,
    Teq,
}

enum CalcOp {
    Add,
    Adc,
    Sub,
    Sbc,
    Rsb,
    Mul,
    Udiv,
    Mls,
}

fn emit_ubfx(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    check_cc(lowering, insn);

    let rd = insn.data.arm_operands.rd;
    let rn = emit_read_reg(lowering, insn.data.arm_operands.rn);
    let lsb = imm_operand(insn, 2);
    let width = imm_operand(insn, 3);
    let result = if width == 0 || lsb >= 32 {
        lowering.iconst_u32(0)
    } else {
        let eff_width = width.min(32 - lsb);
        let shifted = lowering.builder.ins().ushr_imm(rn, i64::from(lsb));
        if eff_width >= 32 {
            shifted
        } else {
            let mask = lowering.iconst_u32((1u32 << eff_width) - 1);
            lowering.builder.ins().band(shifted, mask)
        }
    };

    emit_write_reg(lowering, rd, result);
    emit_return_for_rd(lowering, insn, rd);
}

fn emit_uxtb(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_extend_mask(lowering, insn, 0xFF);
}

fn emit_uxth(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_extend_mask(lowering, insn, 0xFFFF);
}

fn emit_extend_mask(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>, mask: u32) {
    check_cc(lowering, insn);

    let rd = insn.data.arm_operands.rd;
    let (value, _) = emit_resolve_op2(lowering);
    let mask = lowering.iconst_u32(mask);
    let result = lowering.builder.ins().band(value, mask);
    emit_write_reg(lowering, rd, result);
    emit_return_for_rd(lowering, insn, rd);
}

fn emit_cmp(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_compare(lowering, insn, CompareOp::Cmp);
}

fn emit_cmn(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_compare(lowering, insn, CompareOp::Cmn);
}

fn emit_compare(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>, op: CompareOp) {
    check_cc(lowering, insn);

    let rn = emit_read_reg(lowering, insn.data.arm_operands.rn);
    let (op2, _) = emit_resolve_op2(lowering);
    let sign_mask = lowering.iconst_u32(0x8000_0000);

    let (result, carry, overflow) = match op {
        CompareOp::Cmp => {
            let result = lowering.builder.ins().isub(rn, op2);
            let carry_cond = lowering
                .builder
                .ins()
                .icmp(IntCC::UnsignedGreaterThanOrEqual, rn, op2);
            let carry = emit_bool_to_u32(lowering, carry_cond);
            let lhs_xor_rhs = lowering.builder.ins().bxor(rn, op2);
            let lhs_xor_res = lowering.builder.ins().bxor(rn, result);
            let bits = lowering.builder.ins().band(lhs_xor_rhs, lhs_xor_res);
            let bits = lowering.builder.ins().band(bits, sign_mask);
            let overflow_cond = lowering.builder.ins().icmp_imm(IntCC::NotEqual, bits, 0);
            let overflow = emit_bool_to_u32(lowering, overflow_cond);
            (result, carry, overflow)
        }
        CompareOp::Cmn => {
            let result = lowering.builder.ins().iadd(rn, op2);
            let zero = lowering.iconst_u32(0);
            let carry = emit_add_carry(lowering, rn, op2, zero);
            let overflow = emit_add_overflow(lowering, rn, op2, result);
            (result, carry, overflow)
        }
    };

    emit_update_apsr_nzcv(lowering, result, carry, overflow);
    emit_return_size(lowering, insn);
}

fn emit_tst(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_test(lowering, insn, TestOp::Tst);
}

fn emit_teq(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_test(lowering, insn, TestOp::Teq);
}

fn emit_test(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>, op: TestOp) {
    check_cc(lowering, insn);

    let rn = emit_read_reg(lowering, insn.data.arm_operands.rn);
    let (op2, carry) = emit_resolve_op2(lowering);
    let result = match op {
        TestOp::Tst => lowering.builder.ins().band(rn, op2),
        TestOp::Teq => lowering.builder.ins().bxor(rn, op2),
    };

    emit_update_apsr_nzc(lowering, result, carry);
    emit_return_size(lowering, insn);
}

fn emit_mov(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_move(lowering, insn, false);
}

fn emit_movs(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_move(lowering, insn, false);
}

fn emit_mvn(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_move(lowering, insn, true);
}

fn emit_move(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>, invert: bool) {
    check_cc(lowering, insn);

    let rd = insn.data.arm_operands.rd;
    let (value, carry) = emit_resolve_op2(lowering);
    let result = if invert {
        lowering.builder.ins().bnot(value)
    } else {
        value
    };

    emit_write_reg(lowering, rd, result);
    if insn.data.update_flags() {
        emit_update_apsr_nzc(lowering, result, carry);
    }
    emit_return_for_rd(lowering, insn, rd);
}

fn emit_and(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_logic(lowering, insn, LogicOp::And);
}

fn emit_orr(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_logic(lowering, insn, LogicOp::Orr);
}

fn emit_eor(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_logic(lowering, insn, LogicOp::Eor);
}

fn emit_bic(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_logic(lowering, insn, LogicOp::Bic);
}

fn emit_orn(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_logic(lowering, insn, LogicOp::Orn);
}

fn emit_logic(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>, op: LogicOp) {
    check_cc(lowering, insn);

    let rd = insn.data.arm_operands.rd;
    let rn = emit_read_reg(lowering, insn.data.arm_operands.rn);
    let (op2, carry) = emit_resolve_op2(lowering);
    let result = match op {
        LogicOp::And => lowering.builder.ins().band(rn, op2),
        LogicOp::Orr => lowering.builder.ins().bor(rn, op2),
        LogicOp::Eor => lowering.builder.ins().bxor(rn, op2),
        LogicOp::Bic => {
            let inverted = lowering.builder.ins().bnot(op2);
            lowering.builder.ins().band(rn, inverted)
        }
        LogicOp::Orn => {
            let inverted = lowering.builder.ins().bnot(op2);
            lowering.builder.ins().bor(rn, inverted)
        }
    };

    emit_write_reg(lowering, rd, result);
    if insn.data.update_flags() {
        emit_update_apsr_nzc(lowering, result, carry);
    }
    emit_return_for_rd(lowering, insn, rd);
}

fn emit_add(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_calculate(lowering, insn, CalcOp::Add);
}

fn emit_adc(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_calculate(lowering, insn, CalcOp::Adc);
}

fn emit_sub(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_calculate(lowering, insn, CalcOp::Sub);
}

fn emit_sbc(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_calculate(lowering, insn, CalcOp::Sbc);
}

fn emit_rsb(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_calculate(lowering, insn, CalcOp::Rsb);
}

fn emit_mul(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_calculate(lowering, insn, CalcOp::Mul);
}

fn emit_udiv(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_calculate(lowering, insn, CalcOp::Udiv);
}

fn emit_mls(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    emit_calculate(lowering, insn, CalcOp::Mls);
}

fn emit_calculate(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>, op: CalcOp) {
    check_cc(lowering, insn);

    let rd = insn.data.arm_operands.rd;
    let rn = emit_read_reg(lowering, insn.data.arm_operands.rn);
    let (op2, _) = emit_resolve_op2(lowering);
    let zero = lowering.iconst_u32(0);

    let result = match op {
        CalcOp::Add => lowering.builder.ins().iadd(rn, op2),
        CalcOp::Adc => {
            let carry_in = emit_current_carry(lowering);
            let partial = lowering.builder.ins().iadd(rn, op2);
            lowering.builder.ins().iadd(partial, carry_in)
        }
        CalcOp::Sub => lowering.builder.ins().isub(rn, op2),
        CalcOp::Sbc => {
            let carry_in = emit_current_carry(lowering);
            let one = lowering.iconst_u32(1);
            let borrow = lowering.builder.ins().bxor(carry_in, one);
            let rhs = lowering.builder.ins().iadd(op2, borrow);
            lowering.builder.ins().isub(rn, rhs)
        }
        CalcOp::Rsb => lowering.builder.ins().isub(op2, rn),
        CalcOp::Mul => lowering.builder.ins().imul(rn, op2),
        CalcOp::Udiv => emit_udiv_or_zero(lowering, rn, op2),
        CalcOp::Mls => {
            let ra = reg_operand(insn, 3);
            let ra = emit_read_reg(lowering, ra);
            let product = lowering.builder.ins().imul(rn, op2);
            lowering.builder.ins().isub(ra, product)
        }
    };

    emit_write_reg(lowering, rd, result);

    match op {
        CalcOp::Add => {
            if insn.data.update_flags() {
                let carry = emit_add_carry(lowering, rn, op2, zero);
                let overflow = emit_add_overflow(lowering, rn, op2, result);
                emit_update_apsr_nzcv(lowering, result, carry, overflow);
            }
        }
        CalcOp::Adc => {
            if insn.data.update_flags() {
                let carry_in = emit_current_carry(lowering);
                let carry = emit_add_carry(lowering, rn, op2, carry_in);
                let rhs = lowering.builder.ins().iadd(op2, carry_in);
                let overflow = emit_add_overflow(lowering, rn, rhs, result);
                emit_update_apsr_nzcv(lowering, result, carry, overflow);
            }
        }
        CalcOp::Sub => {
            if insn.data.update_flags() {
                let carry_cond = lowering
                    .builder
                    .ins()
                    .icmp(IntCC::UnsignedGreaterThanOrEqual, rn, op2);
                let carry = emit_bool_to_u32(lowering, carry_cond);
                let overflow = emit_sub_overflow(lowering, rn, op2, result);
                emit_update_apsr_nzcv(lowering, result, carry, overflow);
            }
        }
        CalcOp::Sbc => {
            if insn.data.update_flags() {
                let carry_in = emit_current_carry(lowering);
                let one = lowering.iconst_u32(1);
                let borrow = lowering.builder.ins().bxor(carry_in, one);
                let rhs = lowering.builder.ins().iadd(op2, borrow);
                let carry_cond = lowering
                    .builder
                    .ins()
                    .icmp(IntCC::UnsignedGreaterThanOrEqual, rn, rhs);
                let carry = emit_bool_to_u32(lowering, carry_cond);
                let overflow = emit_sub_overflow(lowering, rn, rhs, result);
                emit_update_apsr_nzcv(lowering, result, carry, overflow);
            }
        }
        CalcOp::Rsb => {
            if insn.data.update_flags() {
                let carry_cond = lowering
                    .builder
                    .ins()
                    .icmp(IntCC::UnsignedGreaterThanOrEqual, op2, rn);
                let carry = emit_bool_to_u32(lowering, carry_cond);
                let overflow = emit_sub_overflow(lowering, op2, rn, result);
                emit_update_apsr_nzcv(lowering, result, carry, overflow);
            }
        }
        CalcOp::Mul => {
            if insn.data.update_flags() {
                emit_update_apsr_nz(lowering, result);
            }
        }
        CalcOp::Udiv | CalcOp::Mls => {}
    }

    emit_return_for_rd(lowering, insn, rd);
}

fn emit_udiv_or_zero(lowering: &mut LoweringContext<'_, '_>, lhs: Value, rhs: Value) -> Value {
    lowering.call_value(lowering.helpers.udiv_or_zero, &[lhs, rhs])
}

fn emit_shift(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    check_cc(lowering, insn);

    let rd = insn.data.arm_operands.rd;
    let (result, carry) = emit_compute_shift(lowering);
    emit_write_reg(lowering, rd, result);
    if insn.data.update_flags() {
        emit_update_apsr_nzc(lowering, result, carry);
    }
    emit_return_for_rd(lowering, insn, rd);
}

fn emit_add_carry(
    lowering: &mut LoweringContext<'_, '_>,
    lhs: Value,
    rhs: Value,
    extra: Value,
) -> Value {
    let lhs64 = lowering.builder.ins().uextend(types::I64, lhs);
    let rhs64 = lowering.builder.ins().uextend(types::I64, rhs);
    let extra64 = lowering.builder.ins().uextend(types::I64, extra);
    let partial = lowering.builder.ins().iadd(lhs64, rhs64);
    let wide = lowering.builder.ins().iadd(partial, extra64);
    let overflow = lowering
        .builder
        .ins()
        .icmp_imm(IntCC::UnsignedGreaterThan, wide, 0xFFFF_FFFF);
    emit_bool_to_u32(lowering, overflow)
}

fn emit_add_overflow(
    lowering: &mut LoweringContext<'_, '_>,
    lhs: Value,
    rhs: Value,
    result: Value,
) -> Value {
    let lhs_xor_res = lowering.builder.ins().bxor(lhs, result);
    let rhs_xor_res = lowering.builder.ins().bxor(rhs, result);
    let bits = lowering.builder.ins().band(lhs_xor_res, rhs_xor_res);
    let sign_mask = lowering.iconst_u32(0x8000_0000);
    let bits = lowering.builder.ins().band(bits, sign_mask);
    let cond = lowering.builder.ins().icmp_imm(IntCC::NotEqual, bits, 0);
    emit_bool_to_u32(lowering, cond)
}

fn emit_sub_overflow(
    lowering: &mut LoweringContext<'_, '_>,
    lhs: Value,
    rhs: Value,
    result: Value,
) -> Value {
    let lhs_xor_rhs = lowering.builder.ins().bxor(lhs, rhs);
    let lhs_xor_res = lowering.builder.ins().bxor(lhs, result);
    let bits = lowering.builder.ins().band(lhs_xor_rhs, lhs_xor_res);
    let sign_mask = lowering.iconst_u32(0x8000_0000);
    let bits = lowering.builder.ins().band(bits, sign_mask);
    let cond = lowering.builder.ins().icmp_imm(IntCC::NotEqual, bits, 0);
    emit_bool_to_u32(lowering, cond)
}

fn imm_operand(insn: &JitInstruction<'_>, index: usize) -> u32 {
    match insn.data.get_operand(index) {
        Some(op) => match op.op_type {
            ArmOperandType::Imm(value) => value as u32,
            _ => 0,
        },
        None => 0,
    }
}

fn reg_operand(insn: &JitInstruction<'_>, index: usize) -> u32 {
    match insn.data.get_operand(index) {
        Some(op) => match op.op_type {
            ArmOperandType::Reg(reg) => insn.data.resolve_reg(reg),
            _ => 0,
        },
        None => 0,
    }
}