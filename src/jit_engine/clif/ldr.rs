use capstone::arch::arm::ArmInsn;
use cranelift::prelude::*;

use crate::jit_engine::clif::instructions::{InsDef, check_cc};
use crate::jit_engine::engine::LoweringContext;
use crate::jit_engine::table::JitInstruction;

pub(crate) static LDR_DEF: LdrDef = LdrDef;
pub(crate) static LDRB_DEF: LdrbDef = LdrbDef;
pub(crate) static LDRSB_DEF: LdrsbDef = LdrsbDef;
pub(crate) static LDRH_DEF: LdrhDef = LdrhDef;
pub(crate) static LDRSH_DEF: LdrshDef = LdrshDef;

pub(crate) fn defs() -> [&'static dyn InsDef; 5] {
    [&LDR_DEF, &LDRB_DEF, &LDRSB_DEF, &LDRH_DEF, &LDRSH_DEF]
}

pub(crate) fn find_def(insn_id: u32) -> Option<&'static dyn InsDef> {
    match insn_id {
        x if x == ArmInsn::ARM_INS_LDR as u32 => Some(&LDR_DEF),
        x if x == ArmInsn::ARM_INS_LDRB as u32 => Some(&LDRB_DEF),
        x if x == ArmInsn::ARM_INS_LDRSB as u32 => Some(&LDRSB_DEF),
        x if x == ArmInsn::ARM_INS_LDRH as u32 => Some(&LDRH_DEF),
        x if x == ArmInsn::ARM_INS_LDRSH as u32 => Some(&LDRSH_DEF),
        _ => None,
    }
}

pub(crate) struct LdrDef;

impl InsDef for LdrDef {
    fn insn_id(&self) -> u32 {
        ArmInsn::ARM_INS_LDR as u32
    }

    fn mnemonic(&self) -> &'static str {
        "LDR"
    }

    fn supports(&self, insn: &JitInstruction<'_>) -> bool {
        !insn.data.arm_operands.mem_has_index
    }

    fn execute(&self, lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
        emit_load(lowering, insn, LoadKind::Word);
    }
}

pub(crate) struct LdrbDef;

impl InsDef for LdrbDef {
    fn insn_id(&self) -> u32 {
        ArmInsn::ARM_INS_LDRB as u32
    }

    fn mnemonic(&self) -> &'static str {
        "LDRB"
    }

    fn supports(&self, insn: &JitInstruction<'_>) -> bool {
        !insn.data.arm_operands.mem_has_index
    }

    fn execute(&self, lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
        emit_load(lowering, insn, LoadKind::Byte);
    }
}

pub(crate) struct LdrsbDef;

impl InsDef for LdrsbDef {
    fn insn_id(&self) -> u32 {
        ArmInsn::ARM_INS_LDRSB as u32
    }

    fn mnemonic(&self) -> &'static str {
        "LDRSB"
    }

    fn supports(&self, insn: &JitInstruction<'_>) -> bool {
        !insn.data.arm_operands.mem_has_index
    }

    fn execute(&self, lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
        emit_load(lowering, insn, LoadKind::SignedByte);
    }
}

pub(crate) struct LdrhDef;

impl InsDef for LdrhDef {
    fn insn_id(&self) -> u32 {
        ArmInsn::ARM_INS_LDRH as u32
    }

    fn mnemonic(&self) -> &'static str {
        "LDRH"
    }

    fn supports(&self, insn: &JitInstruction<'_>) -> bool {
        !insn.data.arm_operands.mem_has_index
    }

    fn execute(&self, lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
        emit_load(lowering, insn, LoadKind::Halfword);
    }
}

pub(crate) struct LdrshDef;

impl InsDef for LdrshDef {
    fn insn_id(&self) -> u32 {
        ArmInsn::ARM_INS_LDRSH as u32
    }

    fn mnemonic(&self) -> &'static str {
        "LDRSH"
    }

    fn supports(&self, insn: &JitInstruction<'_>) -> bool {
        !insn.data.arm_operands.mem_has_index
    }

    fn execute(&self, lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
        emit_load(lowering, insn, LoadKind::SignedHalfword);
    }
}

enum LoadKind {
    Word,
    Byte,
    SignedByte,
    Halfword,
    SignedHalfword,
}

fn emit_load(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>, kind: LoadKind) {
    check_cc(lowering, insn);

    let (rd, addr) = ldr_operand_resolve(lowering, insn);
    let load_value = emit_load_value(lowering, addr, kind);

    lowering.call_void(
        lowering.helpers.write_reg,
        &[lowering.cpu_ptr, rd, load_value],
    );

    let pc_update = if insn.data.arm_operands.rd == 15 {
        lowering.iconst_u32(0)
    } else {
        lowering.iconst_u32(insn.data.size())
    };
    lowering.builder.ins().return_(&[pc_update]);
}

fn emit_load_value(lowering: &mut LoweringContext<'_, '_>, addr: Value, kind: LoadKind) -> Value {
    match kind {
        LoadKind::Word => {
            let align_mask = lowering.iconst_u32(!3u32);
            let aligned = lowering.builder.ins().band(addr, align_mask);
            lowering.call_value(lowering.helpers.read_u32, &[lowering.cpu_ptr, aligned])
        }
        LoadKind::Byte => lowering.call_value(lowering.helpers.read_u8, &[lowering.cpu_ptr, addr]),
        LoadKind::SignedByte => {
            let value = lowering.call_value(lowering.helpers.read_u8, &[lowering.cpu_ptr, addr]);
            let value = lowering.builder.ins().ireduce(types::I8, value);
            lowering.builder.ins().sextend(types::I32, value)
        }
        LoadKind::Halfword => {
            lowering.call_value(lowering.helpers.read_u16, &[lowering.cpu_ptr, addr])
        }
        LoadKind::SignedHalfword => {
            let value = lowering.call_value(lowering.helpers.read_u16, &[lowering.cpu_ptr, addr]);
            let value = lowering.builder.ins().ireduce(types::I16, value);
            lowering.builder.ins().sextend(types::I32, value)
        }
    }
}

fn ldr_operand_resolve(
    lowering: &mut LoweringContext<'_, '_>,
    insn: &JitInstruction<'_>,
) -> (Value, Value) {
    let rd = lowering.iconst_u32(insn.data.arm_operands.rd);
    let rn = lowering.iconst_u32(insn.data.arm_operands.rn);
    let base = lowering.call_value(lowering.helpers.read_reg, &[lowering.cpu_ptr, rn]);
    let disp = lowering.iconst_i32(insn.data.arm_operands.mem_disp);

    if !insn.data.arm_operands.mem_writeback {
        let addr = lowering.builder.ins().iadd(base, disp);
        return (rd, addr);
    }

    if insn.data.arm_operands.mem_post_index {
        let mem_post_imm = lowering.iconst_i32(insn.data.arm_operands.mem_post_imm);
        let new_base = lowering.builder.ins().iadd(base, mem_post_imm);
        lowering.call_void(
            lowering.helpers.write_reg,
            &[lowering.cpu_ptr, rn, new_base],
        );
        (rd, base)
    } else {
        let addr = lowering.builder.ins().iadd(base, disp);
        lowering.call_void(lowering.helpers.write_reg, &[lowering.cpu_ptr, rn, addr]);
        (rd, addr)
    }
}
