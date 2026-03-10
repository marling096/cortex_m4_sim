use capstone::arch::arm::ArmCC;
use cranelift::prelude::*;

use crate::jit_engine::clif::adr;
use crate::jit_engine::clif::control;
use crate::jit_engine::clif::data;
use crate::jit_engine::clif::ldr;
use crate::jit_engine::clif::memory;
use crate::jit_engine::clif::misc;
use crate::jit_engine::engine::LoweringContext;
use crate::jit_engine::table::JitInstruction;

pub trait InsDef {
    fn insn_id(&self) -> u32;

    fn mnemonic(&self) -> &'static str;

    fn supports(&self, _insn: &JitInstruction<'_>) -> bool {
        true
    }

    fn execute(&self, lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>);
}

pub fn find_def(insn_id: u32) -> Option<&'static dyn InsDef> {
    adr::find_def(insn_id)
        .or_else(|| ldr::find_def(insn_id))
        .or_else(|| data::find_def(insn_id))
        .or_else(|| control::find_def(insn_id))
        .or_else(|| memory::find_def(insn_id))
        .or_else(|| misc::find_def(insn_id))
}

pub fn check_cc(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    let cc = insn.data.arm_operands.condition;
    if cc == ArmCC::ARM_CC_AL {
        return;
    }

    let cc_value = lowering.iconst_u32(cc as u32);
    let cond = lowering.call_value(
        lowering.helpers.check_condition,
        &[lowering.cpu_ptr, cc_value],
    );

    let execute_block = lowering.builder.create_block();
    let skip_block = lowering.builder.create_block();
    lowering
        .builder
        .ins()
        .brif(cond, execute_block, &[], skip_block, &[]);

    lowering.builder.switch_to_block(skip_block);
    lowering.builder.seal_block(skip_block);
    let skipped = lowering.iconst_u32(insn.data.size());
    lowering.builder.ins().return_(&[skipped]);

    lowering.builder.switch_to_block(execute_block);
    lowering.builder.seal_block(execute_block);
}

pub fn emit_size_value(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) -> Value {
    lowering.iconst_u32(insn.data.size())
}

pub fn emit_return_size(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    let size = emit_size_value(lowering, insn);
    lowering.builder.ins().return_(&[size]);
}

pub fn emit_pc_update_for_rd(
    lowering: &mut LoweringContext<'_, '_>,
    insn: &JitInstruction<'_>,
    rd: u32,
) -> Value {
    if rd == 15 {
        lowering.iconst_u32(0)
    } else {
        emit_size_value(lowering, insn)
    }
}

pub fn emit_return_for_rd(
    lowering: &mut LoweringContext<'_, '_>,
    insn: &JitInstruction<'_>,
    rd: u32,
) {
    let pc_update = emit_pc_update_for_rd(lowering, insn, rd);
    lowering.builder.ins().return_(&[pc_update]);
}

pub fn emit_read_reg(lowering: &mut LoweringContext<'_, '_>, reg: u32) -> Value {
    let reg = lowering.iconst_u32(reg);
    lowering.call_value(lowering.helpers.read_reg, &[lowering.cpu_ptr, reg])
}

pub fn emit_write_reg(lowering: &mut LoweringContext<'_, '_>, reg: u32, value: Value) {
    let reg = lowering.iconst_u32(reg);
    lowering.call_void(lowering.helpers.write_reg, &[lowering.cpu_ptr, reg, value]);
}

pub fn emit_bool_to_u32(lowering: &mut LoweringContext<'_, '_>, cond: Value) -> Value {
    let one = lowering.iconst_u32(1);
    let zero = lowering.iconst_u32(0);
    lowering.builder.ins().select(cond, one, zero)
}

pub fn emit_read_apsr(lowering: &mut LoweringContext<'_, '_>) -> Value {
    lowering.call_value(lowering.helpers.read_apsr, &[lowering.cpu_ptr])
}

pub fn emit_current_carry(lowering: &mut LoweringContext<'_, '_>) -> Value {
    let apsr = emit_read_apsr(lowering);
    let shifted = lowering.builder.ins().ushr_imm(apsr, 29);
    let mask = lowering.iconst_u32(1);
    lowering.builder.ins().band(shifted, mask)
}

pub fn emit_update_apsr_n(lowering: &mut LoweringContext<'_, '_>, value: Value) {
    lowering.call_void(lowering.helpers.update_apsr_n, &[lowering.cpu_ptr, value]);
}

pub fn emit_update_apsr_z(lowering: &mut LoweringContext<'_, '_>, value: Value) {
    lowering.call_void(lowering.helpers.update_apsr_z, &[lowering.cpu_ptr, value]);
}

pub fn emit_update_apsr_c(lowering: &mut LoweringContext<'_, '_>, value: Value) {
    lowering.call_void(lowering.helpers.update_apsr_c, &[lowering.cpu_ptr, value]);
}

pub fn emit_update_apsr_v(lowering: &mut LoweringContext<'_, '_>, value: Value) {
    lowering.call_void(lowering.helpers.update_apsr_v, &[lowering.cpu_ptr, value]);
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

pub fn emit_resolve_op2(lowering: &mut LoweringContext<'_, '_>) -> (Value, Value) {
    let packed = lowering.call_value(
        lowering.helpers.resolve_op2_packed,
        &[lowering.cpu_ptr, lowering.instr_ptr],
    );
    let value = lowering.builder.ins().ireduce(types::I32, packed);
    let carry64 = lowering.builder.ins().ushr_imm(packed, 32);
    let carry = lowering.builder.ins().ireduce(types::I32, carry64);
    (value, carry)
}

pub fn emit_resolve_simple_op2(lowering: &mut LoweringContext<'_, '_>) -> Value {
    lowering.call_value(
        lowering.helpers.resolve_simple_op2_value,
        &[lowering.cpu_ptr, lowering.instr_ptr],
    )
}

pub fn emit_resolve_mem_rt_addr(lowering: &mut LoweringContext<'_, '_>) -> (Value, Value) {
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
    let packed = lowering.call_value(
        lowering.helpers.compute_shift_packed,
        &[lowering.cpu_ptr, lowering.instr_ptr],
    );
    let value = lowering.builder.ins().ireduce(types::I32, packed);
    let carry64 = lowering.builder.ins().ushr_imm(packed, 32);
    let carry = lowering.builder.ins().ireduce(types::I32, carry64);
    (value, carry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opcodes::instruction::OpcodeTable;

    #[test]
    fn jit_registry_covers_all_opcode_definitions() {
        let table = OpcodeTable::new();
        let mut missing = Vec::new();

        for defs in table.get_table().values() {
            for def in defs {
                if find_def(def.insnid).is_none() {
                    missing.push(format!("{} ({})", def.name, def.insnid));
                }
            }
        }

        assert!(
            missing.is_empty(),
            "missing jit defs for: {}",
            missing.join(", ")
        );
    }
}