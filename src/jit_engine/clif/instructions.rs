use crate::arch::ArmCC;
use cranelift::prelude::*;

use crate::jit_engine::clif::adr;
use crate::jit_engine::clif::control;
use crate::jit_engine::clif::data;
use crate::jit_engine::clif::ldr;
use crate::jit_engine::clif::memory;
use crate::jit_engine::clif::misc;
use crate::jit_engine::engine::LoweringContext;
use crate::jit_engine::table::JitInstruction;
use crate::opcodes::decoded::{DecodedOperandKind, DecodedShift};

pub trait InsDef {
    fn insn_id(&self) -> u32;

    fn mnemonic(&self) -> &'static str;

    fn supports(&self, _insn: &JitInstruction) -> bool {
        true
    }

    fn execute(&self, lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction);
}

pub fn find_def(insn_id: u32) -> Option<&'static dyn InsDef> {
    adr::find_def(insn_id)
        .or_else(|| ldr::find_def(insn_id))
        .or_else(|| data::find_def(insn_id))
        .or_else(|| control::find_def(insn_id))
        .or_else(|| memory::find_def(insn_id))
        .or_else(|| misc::find_def(insn_id))
}

pub fn with_cc<F>(
    lowering: &mut LoweringContext<'_, '_>,
    insn: &JitInstruction,
    emit: F,
) where
    F: FnOnce(&mut LoweringContext<'_, '_>),
{
    let cc = insn.data.arm_operands.condition;
    if cc == ArmCC::ARM_CC_AL {
        emit(lowering);
        return;
    }

    let cond = emit_check_condition(lowering, cc);

    let execute_block = lowering.builder.create_block();
    let skip_block = lowering.builder.create_block();
    let join_block = lowering.builder.create_block();
    lowering
        .builder
        .ins()
        .brif(cond, execute_block, &[], skip_block, &[]);

    lowering.builder.switch_to_block(skip_block);
    lowering.builder.seal_block(skip_block);
    lowering.advance_pc_for_insn(insn);
    lowering.builder.ins().jump(join_block, &[]);

    lowering.builder.switch_to_block(execute_block);
    lowering.builder.seal_block(execute_block);
    emit(lowering);
    lowering.builder.ins().jump(join_block, &[]);

    lowering.builder.seal_block(join_block);
    lowering.builder.switch_to_block(join_block);
}

pub fn emit_size_value(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction) -> Value {
    lowering.iconst_u32(insn.data.size())
}

pub fn emit_pc_update_for_rd(
    lowering: &mut LoweringContext<'_, '_>,
    insn: &JitInstruction,
    rd: u32,
) {
    lowering.advance_pc_for_rd(insn, rd);
}

pub fn emit_read_reg_value(lowering: &mut LoweringContext<'_, '_>, reg: Value) -> Value {
    let pc_reg = lowering.iconst_u32(15);
    let is_pc = lowering.builder.ins().icmp(IntCC::Equal, reg, pc_reg);
    let pc_block = lowering.builder.create_block();
    let read_block = lowering.builder.create_block();
    let join_block = lowering.builder.create_block();
    lowering.builder.append_block_param(join_block, types::I32);
    lowering
        .builder
        .ins()
        .brif(is_pc, pc_block, &[], read_block, &[]);

    lowering.builder.switch_to_block(pc_block);
    lowering.builder.seal_block(pc_block);
    let pc_value = lowering.current_pc_plus_4();
    lowering.builder.ins().jump(join_block, &[pc_value.into()]);

    lowering.builder.switch_to_block(read_block);
    lowering.builder.seal_block(read_block);
    let value = lowering.load_dynamic_cpu_reg(reg);
    lowering.builder.ins().jump(join_block, &[value.into()]);

    lowering.builder.seal_block(join_block);
    lowering.builder.switch_to_block(join_block);
    lowering.builder.block_params(join_block)[0]
}

pub fn emit_read_reg(lowering: &mut LoweringContext<'_, '_>, reg: u32) -> Value {
    lowering.read_cached_reg(reg)
}

pub fn emit_write_reg(lowering: &mut LoweringContext<'_, '_>, reg: u32, value: Value) {
    lowering.write_cached_reg(reg, value);
}

pub fn emit_bool_to_u32(lowering: &mut LoweringContext<'_, '_>, cond: Value) -> Value {
    let one = lowering.iconst_u32(1);
    let zero = lowering.iconst_u32(0);
    lowering.builder.ins().select(cond, one, zero)
}

pub fn emit_read_apsr(lowering: &mut LoweringContext<'_, '_>) -> Value {
    lowering.read_cached_apsr()
}

pub fn emit_current_carry(lowering: &mut LoweringContext<'_, '_>) -> Value {
    let apsr = emit_read_apsr(lowering);
    let shifted = lowering.builder.ins().ushr_imm(apsr, 29);
    let mask = lowering.iconst_u32(1);
    lowering.builder.ins().band(shifted, mask)
}

pub fn emit_update_apsr_n(lowering: &mut LoweringContext<'_, '_>, value: Value) {
    let apsr = emit_read_apsr(lowering);
    let sign = lowering.builder.ins().ushr_imm(value, 31);
    let updated = emit_replace_apsr_bit(lowering, apsr, 31, sign);
    lowering.write_cached_apsr(updated);
}

pub fn emit_update_apsr_z(lowering: &mut LoweringContext<'_, '_>, value: Value) {
    let apsr = emit_read_apsr(lowering);
    let is_zero = lowering.builder.ins().icmp_imm(IntCC::Equal, value, 0);
    let zero = emit_bool_to_u32(lowering, is_zero);
    let updated = emit_replace_apsr_bit(lowering, apsr, 30, zero);
    lowering.write_cached_apsr(updated);
}

pub fn emit_update_apsr_c(lowering: &mut LoweringContext<'_, '_>, value: Value) {
    let apsr = emit_read_apsr(lowering);
    let updated = emit_replace_apsr_bit(lowering, apsr, 29, value);
    lowering.write_cached_apsr(updated);
}

pub fn emit_update_apsr_v(lowering: &mut LoweringContext<'_, '_>, value: Value) {
    let apsr = emit_read_apsr(lowering);
    let updated = emit_replace_apsr_bit(lowering, apsr, 28, value);
    lowering.write_cached_apsr(updated);
}

pub fn emit_update_apsr_nz(lowering: &mut LoweringContext<'_, '_>, value: Value) {
    emit_update_apsr_n(lowering, value);
    emit_update_apsr_z(lowering, value);
}

pub fn emit_update_apsr_nzc(
    lowering: &mut LoweringContext<'_, '_>,
    value: Value,
    carry: Value,
) {
    emit_update_apsr_n(lowering, value);
    emit_update_apsr_z(lowering, value);
    emit_update_apsr_c(lowering, carry);
}

pub fn emit_update_apsr_nzcv(
    lowering: &mut LoweringContext<'_, '_>,
    value: Value,
    carry: Value,
    overflow: Value,
) {
    emit_update_apsr_n(lowering, value);
    emit_update_apsr_z(lowering, value);
    emit_update_apsr_c(lowering, carry);
    emit_update_apsr_v(lowering, overflow);
}

pub fn emit_resolve_op2(
    lowering: &mut LoweringContext<'_, '_>,
    insn: &JitInstruction,
) -> (Value, Value) {
    if let Some((value, carry)) = emit_static_op2(lowering, insn) {
        return (value, carry);
    }

    lowering.flush_dirty_state();
    let packed = lowering.call_value(
        lowering.helpers.resolve_op2_packed,
        &[lowering.cpu_ptr, lowering.instr_ptr],
    );
    let value = lowering.builder.ins().ireduce(types::I32, packed);
    let carry64 = lowering.builder.ins().ushr_imm(packed, 32);
    let carry = lowering.builder.ins().ireduce(types::I32, carry64);
    (value, carry)
}

fn emit_static_op2(
    lowering: &mut LoweringContext<'_, '_>,
    insn: &JitInstruction,
) -> Option<(Value, Value)> {
    let current_carry = emit_current_carry(lowering);
    let op2 = insn.data.arm_operands.op2.as_ref()?;

    match op2.op_type {
        DecodedOperandKind::Imm(imm) => Some((lowering.iconst_u32(imm as u32), current_carry)),
        DecodedOperandKind::Reg(reg) => {
            let value = emit_read_reg(lowering, reg);
            emit_static_shifted_reg_op2(lowering, value, current_carry, op2.shift)
        }
        _ => None,
    }
}

fn emit_static_shifted_reg_op2(
    lowering: &mut LoweringContext<'_, '_>,
    value: Value,
    current_carry: Value,
    shift: DecodedShift,
) -> Option<(Value, Value)> {
    match shift {
        DecodedShift::Invalid
        | DecodedShift::Lsl(0)
        | DecodedShift::Lsr(0)
        | DecodedShift::Asr(0) => {
            Some((value, current_carry))
        }
        DecodedShift::Lsl(amount) => match amount {
            1..=31 => {
                let result = lowering.builder.ins().ishl_imm(value, i64::from(amount));
                let carry = emit_shifted_bit(lowering, value, amount - 1, true);
                Some((result, carry))
            }
            32 => {
                let result = lowering.iconst_u32(0);
                let carry = emit_shifted_bit(lowering, value, 0, false);
                Some((result, carry))
            }
            _ => Some((lowering.iconst_u32(0), lowering.iconst_u32(0))),
        },
        DecodedShift::Lsr(amount) => match amount {
            1..=31 => {
                let result = lowering.builder.ins().ushr_imm(value, i64::from(amount));
                let carry = emit_shifted_bit(lowering, value, amount - 1, false);
                Some((result, carry))
            }
            32 => {
                let result = lowering.iconst_u32(0);
                let carry = emit_shifted_bit(lowering, value, 31, false);
                Some((result, carry))
            }
            _ => Some((lowering.iconst_u32(0), lowering.iconst_u32(0))),
        },
        DecodedShift::Asr(amount) => match amount {
            1..=31 => {
                let result = lowering.builder.ins().sshr_imm(value, i64::from(amount));
                let carry = emit_shifted_bit(lowering, value, amount - 1, false);
                Some((result, carry))
            }
            _ => {
                let result = lowering.builder.ins().sshr_imm(value, 31);
                let carry = emit_shifted_bit(lowering, value, 31, false);
                Some((result, carry))
            }
        },
        DecodedShift::Ror(_) | DecodedShift::Rrx(_) => None,
    }
}

fn emit_shifted_bit(
    lowering: &mut LoweringContext<'_, '_>,
    value: Value,
    bit_index: u32,
    from_left: bool,
) -> Value {
    let shift = if from_left {
        31u32.saturating_sub(bit_index)
    } else {
        bit_index
    };
    let shifted = lowering.builder.ins().ushr_imm(value, i64::from(shift));
    let mask = lowering.iconst_u32(1);
    lowering.builder.ins().band(shifted, mask)
}

pub fn emit_resolve_mem_rt_addr(lowering: &mut LoweringContext<'_, '_>) -> (Value, Value) {
    lowering.flush_dirty_state();
    let packed = lowering.call_value(
        lowering.helpers.resolve_mem_rt_addr,
        &[lowering.cpu_ptr, lowering.instr_ptr],
    );
    let rt = lowering.builder.ins().ireduce(types::I32, packed);
    let addr64 = lowering.builder.ins().ushr_imm(packed, 32);
    let addr = lowering.builder.ins().ireduce(types::I32, addr64);
    (rt, addr)
}

pub fn emit_compute_shift(lowering: &mut LoweringContext<'_, '_>) -> (Value, Value) {
    lowering.flush_dirty_state();
    let packed = lowering.call_value(
        lowering.helpers.compute_shift_packed,
        &[lowering.cpu_ptr, lowering.instr_ptr],
    );
    let value = lowering.builder.ins().ireduce(types::I32, packed);
    let carry64 = lowering.builder.ins().ushr_imm(packed, 32);
    let carry = lowering.builder.ins().ireduce(types::I32, carry64);
    (value, carry)
}

fn emit_replace_apsr_bit(
    lowering: &mut LoweringContext<'_, '_>,
    apsr: Value,
    bit: u8,
    flag: Value,
) -> Value {
    let one = lowering.iconst_u32(1);
    let flag = lowering.builder.ins().band(flag, one);
    let shifted = lowering.builder.ins().ishl_imm(flag, i64::from(bit));
    let mask = lowering.iconst_u32(!(1u32 << bit));
    let cleared = lowering.builder.ins().band(apsr, mask);
    lowering.builder.ins().bor(cleared, shifted)
}

fn emit_check_condition(lowering: &mut LoweringContext<'_, '_>, cc: ArmCC) -> Value {
    let apsr = emit_read_apsr(lowering);
    let mask = lowering.iconst_u32(1);
    let n_shifted = lowering.builder.ins().ushr_imm(apsr, 31);
    let z_shifted = lowering.builder.ins().ushr_imm(apsr, 30);
    let c_shifted = lowering.builder.ins().ushr_imm(apsr, 29);
    let v_shifted = lowering.builder.ins().ushr_imm(apsr, 28);
    let n = lowering.builder.ins().band(n_shifted, mask);
    let z = lowering.builder.ins().band(z_shifted, mask);
    let c = lowering.builder.ins().band(c_shifted, mask);
    let v = lowering.builder.ins().band(v_shifted, mask);
    let zero = lowering.iconst_u32(0);
    let one = lowering.iconst_u32(1);

    match cc {
        ArmCC::ARM_CC_EQ => lowering.builder.ins().icmp(IntCC::Equal, z, one),
        ArmCC::ARM_CC_NE => lowering.builder.ins().icmp(IntCC::Equal, z, zero),
        ArmCC::ARM_CC_HS => lowering.builder.ins().icmp(IntCC::Equal, c, one),
        ArmCC::ARM_CC_LO => lowering.builder.ins().icmp(IntCC::Equal, c, zero),
        ArmCC::ARM_CC_MI => lowering.builder.ins().icmp(IntCC::Equal, n, one),
        ArmCC::ARM_CC_PL => lowering.builder.ins().icmp(IntCC::Equal, n, zero),
        ArmCC::ARM_CC_VS => lowering.builder.ins().icmp(IntCC::Equal, v, one),
        ArmCC::ARM_CC_VC => lowering.builder.ins().icmp(IntCC::Equal, v, zero),
        ArmCC::ARM_CC_HI => {
            let c_set = lowering.builder.ins().icmp(IntCC::Equal, c, one);
            let z_clear = lowering.builder.ins().icmp(IntCC::Equal, z, zero);
            lowering.builder.ins().band(c_set, z_clear)
        }
        ArmCC::ARM_CC_LS => {
            let c_clear = lowering.builder.ins().icmp(IntCC::Equal, c, zero);
            let z_set = lowering.builder.ins().icmp(IntCC::Equal, z, one);
            lowering.builder.ins().bor(c_clear, z_set)
        }
        ArmCC::ARM_CC_GE => lowering.builder.ins().icmp(IntCC::Equal, n, v),
        ArmCC::ARM_CC_LT => lowering.builder.ins().icmp(IntCC::NotEqual, n, v),
        ArmCC::ARM_CC_GT => {
            let z_clear = lowering.builder.ins().icmp(IntCC::Equal, z, zero);
            let n_eq_v = lowering.builder.ins().icmp(IntCC::Equal, n, v);
            lowering.builder.ins().band(z_clear, n_eq_v)
        }
        ArmCC::ARM_CC_LE => {
            let z_set = lowering.builder.ins().icmp(IntCC::Equal, z, one);
            let n_ne_v = lowering.builder.ins().icmp(IntCC::NotEqual, n, v);
            lowering.builder.ins().bor(z_set, n_ne_v)
        }
        _ => lowering.builder.ins().icmp(IntCC::Equal, one, one),
    }
}

