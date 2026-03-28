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
    Some(emit_shift_by_kind(lowering, value, current_carry, shift))
}

fn emit_shift_by_kind(
    lowering: &mut LoweringContext<'_, '_>,
    value: Value,
    current_carry: Value,
    shift: DecodedShift,
) -> (Value, Value) {
    match shift {
        DecodedShift::Invalid
        | DecodedShift::Lsl(0)
        | DecodedShift::Lsr(0)
        | DecodedShift::Asr(0) => (value, current_carry),
        DecodedShift::Lsl(amount) => match amount {
            1..=31 => {
                let result = lowering.builder.ins().ishl_imm(value, i64::from(amount));
                let carry = emit_shifted_bit(lowering, value, amount - 1, true);
                (result, carry)
            }
            32 => {
                let result = lowering.iconst_u32(0);
                let carry = emit_shifted_bit(lowering, value, 0, false);
                (result, carry)
            }
            _ => (lowering.iconst_u32(0), lowering.iconst_u32(0)),
        },
        DecodedShift::Lsr(amount) => match amount {
            1..=31 => {
                let result = lowering.builder.ins().ushr_imm(value, i64::from(amount));
                let carry = emit_shifted_bit(lowering, value, amount - 1, false);
                (result, carry)
            }
            32 => {
                let result = lowering.iconst_u32(0);
                let carry = emit_shifted_bit(lowering, value, 31, false);
                (result, carry)
            }
            _ => (lowering.iconst_u32(0), lowering.iconst_u32(0)),
        },
        DecodedShift::Asr(amount) => match amount {
            1..=31 => {
                let result = lowering.builder.ins().sshr_imm(value, i64::from(amount));
                let carry = emit_shifted_bit(lowering, value, amount - 1, false);
                (result, carry)
            }
            _ => {
                let result = lowering.builder.ins().sshr_imm(value, 31);
                let carry = emit_shifted_bit(lowering, value, 31, false);
                (result, carry)
            }
        },
        DecodedShift::Ror(amount) => {
            if amount == 0 {
                (value, current_carry)
            } else {
                let shift_mod = amount % 32;
                if shift_mod == 0 {
                    let carry = emit_shifted_bit(lowering, value, 31, false);
                    (value, carry)
                } else {
                    let right = lowering.builder.ins().ushr_imm(value, i64::from(shift_mod));
                    let left = lowering
                        .builder
                        .ins()
                        .ishl_imm(value, i64::from(32 - shift_mod));
                    let result = lowering.builder.ins().bor(right, left);
                    let carry = emit_shifted_bit(lowering, result, 31, false);
                    (result, carry)
                }
            }
        }
        DecodedShift::Rrx(_) => {
            let one = lowering.iconst_u32(1);
            let carry_in = lowering.builder.ins().band(current_carry, one);
            let carry_shifted = lowering.builder.ins().ishl_imm(carry_in, 31);
            let shifted = lowering.builder.ins().ushr_imm(value, 1);
            let result = lowering.builder.ins().bor(shifted, carry_shifted);
            let carry_out = lowering.builder.ins().band(value, one);
            (result, carry_out)
        }
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

pub fn emit_resolve_mem_rt_addr(
    lowering: &mut LoweringContext<'_, '_>,
    insn: &JitInstruction,
) -> (u32, Value) {
    let rt = insn.data.arm_operands.rd;
    let writeback = insn.data.writeback();
    let post_index = insn.data.arm_operands.mem_post_index;
    let op2 = insn.data.get_operand(1).expect("missing mem operand");

    let (base_reg, base_val, disp, index_offset) = match &op2.op_type {
        DecodedOperandKind::Mem(mem) => {
            let base_reg = mem.base;
            let base_val = emit_read_reg(lowering, base_reg);
            let disp = mem.disp;
            let index_offset = if let Some(index_reg) = mem.index {
                let value = emit_read_reg(lowering, index_reg);
                let current_carry = emit_current_carry(lowering);
                let (shifted, _) = emit_shift_by_kind(lowering, value, current_carry, op2.shift);
                shifted
            } else {
                lowering.iconst_u32(0)
            };
            (base_reg, base_val, disp, index_offset)
        }
        _ => panic!("operand2 is not a memory operand"),
    };

    let disp_value = lowering.iconst_i32(disp);
    let pre_offset = lowering.builder.ins().iadd(index_offset, disp_value);
    if !writeback {
        let addr = lowering.builder.ins().iadd(base_val, pre_offset);
        return (rt, addr);
    }

    if post_index {
        let post_offset = match insn.data.get_operand(2).map(|operand| &operand.op_type) {
            Some(DecodedOperandKind::Imm(imm)) => lowering.iconst_u32(*imm as u32),
            Some(DecodedOperandKind::Reg(reg)) => emit_read_reg(lowering, *reg),
            _ => panic!("third operand is not an immediate/register"),
        };
        let new_base = lowering.builder.ins().iadd(base_val, post_offset);
        emit_write_reg(lowering, base_reg, new_base);
        (rt, base_val)
    } else {
        let addr = lowering.builder.ins().iadd(base_val, pre_offset);
        emit_write_reg(lowering, base_reg, addr);
        (rt, addr)
    }
}

pub fn emit_compute_shift(
    lowering: &mut LoweringContext<'_, '_>,
    insn: &JitInstruction,
) -> (Value, Value) {
    let rm_val = emit_read_reg(lowering, insn.data.arm_operands.rn);
    let current_carry = emit_current_carry(lowering);

    let shift_amount = match &insn.data.arm_operands.op2 {
        Some(op) => match op.op_type {
            DecodedOperandKind::Imm(imm) => {
                return emit_compute_shift_imm(
                    lowering,
                    insn.insn_id,
                    rm_val,
                    current_carry,
                    (imm as u32) & 0xFF,
                )
            }
            DecodedOperandKind::Reg(reg) => {
                let value = emit_read_reg(lowering, reg);
                let mask = lowering.iconst_u32(0xFF);
                lowering.builder.ins().band(value, mask)
            }
            _ => {
                lowering.flush_dirty_state();
                let packed = lowering.call_value(
                    lowering.helpers.compute_shift_packed,
                    &[lowering.cpu_ptr, lowering.instr_ptr],
                );
                let value = lowering.builder.ins().ireduce(types::I32, packed);
                let carry64 = lowering.builder.ins().ushr_imm(packed, 32);
                let carry = lowering.builder.ins().ireduce(types::I32, carry64);
                return (value, carry);
            }
        },
        None => return (rm_val, current_carry),
    };

    emit_compute_shift_dynamic(lowering, insn.insn_id, rm_val, current_carry, shift_amount)
}

fn emit_compute_shift_imm(
    lowering: &mut LoweringContext<'_, '_>,
    insn_id: u32,
    rm_val: Value,
    current_carry: Value,
    shift_amount: u32,
) -> (Value, Value) {
    match insn_id {
        x if x == crate::arch::ArmInsn::ARM_INS_ASR as u32 => {
            emit_shift_by_kind(lowering, rm_val, current_carry, DecodedShift::Asr(shift_amount))
        }
        x if x == crate::arch::ArmInsn::ARM_INS_LSL as u32 => {
            emit_shift_by_kind(lowering, rm_val, current_carry, DecodedShift::Lsl(shift_amount))
        }
        x if x == crate::arch::ArmInsn::ARM_INS_LSR as u32 => {
            emit_shift_by_kind(lowering, rm_val, current_carry, DecodedShift::Lsr(shift_amount))
        }
        x if x == crate::arch::ArmInsn::ARM_INS_ROR as u32 => {
            emit_shift_by_kind(lowering, rm_val, current_carry, DecodedShift::Ror(shift_amount))
        }
        x if x == crate::arch::ArmInsn::ARM_INS_RRX as u32 => {
            emit_shift_by_kind(lowering, rm_val, current_carry, DecodedShift::Rrx(1))
        }
        _ => (rm_val, current_carry),
    }
}

fn emit_compute_shift_dynamic(
    lowering: &mut LoweringContext<'_, '_>,
    insn_id: u32,
    rm_val: Value,
    current_carry: Value,
    shift_amount: Value,
) -> (Value, Value) {
    let one = lowering.iconst_u32(1);
    let zero = lowering.iconst_u32(0);

    let zero_block = lowering.builder.create_block();
    let nonzero_block = lowering.builder.create_block();
    let join_block = lowering.builder.create_block();
    lowering.builder.append_block_param(join_block, types::I32);
    lowering.builder.append_block_param(join_block, types::I32);

    let is_zero = lowering.builder.ins().icmp_imm(IntCC::Equal, shift_amount, 0);
    lowering
        .builder
        .ins()
        .brif(is_zero, zero_block, &[], nonzero_block, &[]);

    lowering.builder.switch_to_block(zero_block);
    lowering.builder.seal_block(zero_block);
    lowering
        .builder
        .ins()
        .jump(join_block, &[rm_val.into(), current_carry.into()]);

    lowering.builder.switch_to_block(nonzero_block);
    lowering.builder.seal_block(nonzero_block);

    let pair = match insn_id {
        x if x == crate::arch::ArmInsn::ARM_INS_LSL as u32 => {
            let too_large_block = lowering.builder.create_block();
            let eq_32_block = lowering.builder.create_block();
            let mid_block = lowering.builder.create_block();
            let range_block = lowering.builder.create_block();
            let is_gt32 = lowering
                .builder
                .ins()
                .icmp_imm(IntCC::UnsignedGreaterThan, shift_amount, 32);
            lowering
                .builder
                .ins()
                .brif(is_gt32, too_large_block, &[], range_block, &[]);

            lowering.builder.switch_to_block(too_large_block);
            lowering.builder.seal_block(too_large_block);
            lowering
                .builder
                .ins()
                .jump(join_block, &[zero.into(), zero.into()]);

            lowering.builder.switch_to_block(range_block);
            lowering.builder.seal_block(range_block);
            let is_eq32 = lowering.builder.ins().icmp_imm(IntCC::Equal, shift_amount, 32);
            lowering
                .builder
                .ins()
                .brif(is_eq32, eq_32_block, &[], mid_block, &[]);

            lowering.builder.switch_to_block(eq_32_block);
            lowering.builder.seal_block(eq_32_block);
            let carry = lowering.builder.ins().band(rm_val, one);
            lowering
                .builder
                .ins()
                .jump(join_block, &[zero.into(), carry.into()]);

            lowering.builder.switch_to_block(mid_block);
            lowering.builder.seal_block(mid_block);
            let result = lowering.builder.ins().ishl(rm_val, shift_amount);
            let thirty_two = lowering.iconst_u32(32);
            let back_shift = lowering.builder.ins().isub(thirty_two, shift_amount);
            let carry_shifted = lowering.builder.ins().ushr(rm_val, back_shift);
            let carry = lowering.builder.ins().band(carry_shifted, one);
            (result, carry)
        }
        x if x == crate::arch::ArmInsn::ARM_INS_LSR as u32 => {
            let too_large_block = lowering.builder.create_block();
            let eq_32_block = lowering.builder.create_block();
            let mid_block = lowering.builder.create_block();
            let range_block = lowering.builder.create_block();
            let is_gt32 = lowering
                .builder
                .ins()
                .icmp_imm(IntCC::UnsignedGreaterThan, shift_amount, 32);
            lowering
                .builder
                .ins()
                .brif(is_gt32, too_large_block, &[], range_block, &[]);

            lowering.builder.switch_to_block(too_large_block);
            lowering.builder.seal_block(too_large_block);
            lowering
                .builder
                .ins()
                .jump(join_block, &[zero.into(), zero.into()]);

            lowering.builder.switch_to_block(range_block);
            lowering.builder.seal_block(range_block);
            let is_eq32 = lowering.builder.ins().icmp_imm(IntCC::Equal, shift_amount, 32);
            lowering
                .builder
                .ins()
                .brif(is_eq32, eq_32_block, &[], mid_block, &[]);

            lowering.builder.switch_to_block(eq_32_block);
            lowering.builder.seal_block(eq_32_block);
            let carry = emit_shifted_bit(lowering, rm_val, 31, false);
            lowering
                .builder
                .ins()
                .jump(join_block, &[zero.into(), carry.into()]);

            lowering.builder.switch_to_block(mid_block);
            lowering.builder.seal_block(mid_block);
            let result = lowering.builder.ins().ushr(rm_val, shift_amount);
            let amount_minus_one = lowering.builder.ins().isub(shift_amount, one);
            let carry_shifted = lowering.builder.ins().ushr(rm_val, amount_minus_one);
            let carry = lowering.builder.ins().band(carry_shifted, one);
            (result, carry)
        }
        x if x == crate::arch::ArmInsn::ARM_INS_ASR as u32 => {
            let large_block = lowering.builder.create_block();
            let mid_block = lowering.builder.create_block();
            let is_gte32 = lowering
                .builder
                .ins()
                .icmp_imm(IntCC::UnsignedGreaterThanOrEqual, shift_amount, 32);
            lowering
                .builder
                .ins()
                .brif(is_gte32, large_block, &[], mid_block, &[]);

            lowering.builder.switch_to_block(large_block);
            lowering.builder.seal_block(large_block);
            let sign = emit_shifted_bit(lowering, rm_val, 31, false);
            let all_ones = lowering.iconst_u32(u32::MAX);
            let sign_is_set = lowering.builder.ins().icmp_imm(IntCC::NotEqual, sign, 0);
            let result = lowering.builder.ins().select(sign_is_set, all_ones, zero);
            let carry = emit_shifted_bit(lowering, rm_val, 31, false);
            lowering
                .builder
                .ins()
                .jump(join_block, &[result.into(), carry.into()]);

            lowering.builder.switch_to_block(mid_block);
            lowering.builder.seal_block(mid_block);
            let result = lowering.builder.ins().sshr(rm_val, shift_amount);
            let amount_minus_one = lowering.builder.ins().isub(shift_amount, one);
            let carry_shifted = lowering.builder.ins().ushr(rm_val, amount_minus_one);
            let carry = lowering.builder.ins().band(carry_shifted, one);
            (result, carry)
        }
        x if x == crate::arch::ArmInsn::ARM_INS_ROR as u32 => {
            let shift_mask = lowering.iconst_u32(31);
            let masked_shift = lowering.builder.ins().band(shift_amount, shift_mask);
            let shift_zero_block = lowering.builder.create_block();
            let shift_nonzero_block = lowering.builder.create_block();
            let shift_is_zero = lowering.builder.ins().icmp_imm(IntCC::Equal, masked_shift, 0);
            lowering
                .builder
                .ins()
                .brif(shift_is_zero, shift_zero_block, &[], shift_nonzero_block, &[]);

            lowering.builder.switch_to_block(shift_zero_block);
            lowering.builder.seal_block(shift_zero_block);
            let carry = emit_shifted_bit(lowering, rm_val, 31, false);
            lowering
                .builder
                .ins()
                .jump(join_block, &[rm_val.into(), carry.into()]);

            lowering.builder.switch_to_block(shift_nonzero_block);
            lowering.builder.seal_block(shift_nonzero_block);
            let right = lowering.builder.ins().ushr(rm_val, masked_shift);
            let thirty_two = lowering.iconst_u32(32);
            let left_shift = lowering.builder.ins().isub(thirty_two, masked_shift);
            let left = lowering.builder.ins().ishl(rm_val, left_shift);
            let result = lowering.builder.ins().bor(right, left);
            let carry = emit_shifted_bit(lowering, result, 31, false);
            (result, carry)
        }
        x if x == crate::arch::ArmInsn::ARM_INS_RRX as u32 => {
            let carry_shifted = lowering.builder.ins().ishl_imm(current_carry, 31);
            let shifted = lowering.builder.ins().ushr_imm(rm_val, 1);
            let result = lowering.builder.ins().bor(shifted, carry_shifted);
            let carry = lowering.builder.ins().band(rm_val, one);
            (result, carry)
        }
        _ => (rm_val, current_carry),
    };

    lowering
        .builder
        .ins()
        .jump(join_block, &[pair.0.into(), pair.1.into()]);

    lowering.builder.seal_block(join_block);
    lowering.builder.switch_to_block(join_block);
    (
        lowering.builder.block_params(join_block)[0],
        lowering.builder.block_params(join_block)[1],
    )
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

