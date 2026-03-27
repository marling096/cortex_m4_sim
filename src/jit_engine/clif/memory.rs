use crate::arch::ArmInsn;
use cranelift::prelude::*;

use crate::jit_engine::clif::instructions::{
    InsDef, emit_read_reg, emit_read_reg_value, emit_resolve_mem_rt_addr, emit_size_value,
    emit_write_reg, with_cc,
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

            fn execute(&self, lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction) {
                $emit(lowering, insn)
            }
        }
    };
}

define_def!(StrDef, STR_DEF, ARM_INS_STR, "STR", emit_str);
define_def!(StrbDef, STRB_DEF, ARM_INS_STRB, "STRB", emit_strb);
define_def!(StrhDef, STRH_DEF, ARM_INS_STRH, "STRH", emit_strh);
define_def!(LdmDef, LDM_DEF, ARM_INS_LDM, "LDM", emit_ldm);
define_def!(StmDef, STM_DEF, ARM_INS_STM, "STM", emit_stm);
define_def!(PushDef, PUSH_DEF, ARM_INS_PUSH, "PUSH", emit_push);
define_def!(PopDef, POP_DEF, ARM_INS_POP, "POP", emit_pop);

pub(crate) fn find_def(insn_id: u32) -> Option<&'static dyn InsDef> {
    match insn_id {
        x if x == ArmInsn::ARM_INS_STR as u32 => Some(&STR_DEF),
        x if x == ArmInsn::ARM_INS_STRB as u32 => Some(&STRB_DEF),
        x if x == ArmInsn::ARM_INS_STRH as u32 => Some(&STRH_DEF),
        x if x == ArmInsn::ARM_INS_LDM as u32 => Some(&LDM_DEF),
        x if x == ArmInsn::ARM_INS_STM as u32 => Some(&STM_DEF),
        x if x == ArmInsn::ARM_INS_PUSH as u32 => Some(&PUSH_DEF),
        x if x == ArmInsn::ARM_INS_POP as u32 => Some(&POP_DEF),
        _ => None,
    }
}

fn emit_str(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction) {
    emit_store(lowering, insn, StoreKind::Word)
}

fn emit_strb(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction) {
    emit_store(lowering, insn, StoreKind::Byte)
}

fn emit_strh(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction) {
    emit_store(lowering, insn, StoreKind::Halfword)
}

enum StoreKind {
    Word,
    Byte,
    Halfword,
}

fn emit_store(
    lowering: &mut LoweringContext<'_, '_>,
    insn: &JitInstruction,
    kind: StoreKind,
) {
    with_cc(lowering, insn, |lowering| {
        let (rt, addr) = emit_resolve_mem_rt_addr(lowering);
        let value = emit_read_reg_value(lowering, rt);

        match kind {
            StoreKind::Word => {
                let mask = lowering.iconst_u32(!3u32);
                let aligned = lowering.builder.ins().band(addr, mask);
                lowering.call_void(lowering.helpers.write_u32, &[lowering.cpu_ptr, aligned, value]);
            }
            StoreKind::Byte => lowering.call_void(
                lowering.helpers.write_u8,
                &[lowering.cpu_ptr, addr, value],
            ),
            StoreKind::Halfword => lowering.call_void(
                lowering.helpers.write_u16,
                &[lowering.cpu_ptr, addr, value],
            ),
        }

        let pc_update = emit_size_value(lowering, insn);
        lowering.advance_pc(pc_update);
    })
}

fn emit_ldm(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction) {
    with_cc(lowering, insn, |lowering| {
        if insn.data.transed_operands.is_empty() {
            let pc_update = emit_size_value(lowering, insn);
            lowering.advance_pc(pc_update);
            return;
        }

        let base_reg = insn.data.transed_operands[0];
        let mut addr = emit_read_reg(lowering, base_reg);
        let mut loads_pc = false;

        for &reg in &insn.data.transed_operands[1..] {
            let value = lowering.call_value(lowering.helpers.read_u32, &[lowering.cpu_ptr, addr]);
            emit_write_reg(lowering, reg, value);
            addr = lowering.builder.ins().iadd_imm(addr, 4);
            if reg == 15 {
                loads_pc = true;
            }
        }

        if insn.data.writeback() {
            emit_write_reg(lowering, base_reg, addr);
        }

        if !loads_pc {
            let pc_update = emit_size_value(lowering, insn);
            lowering.advance_pc(pc_update);
        }
    })
}

fn emit_stm(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction) {
    with_cc(lowering, insn, |lowering| {
        if insn.data.transed_operands.is_empty() {
            let pc_update = emit_size_value(lowering, insn);
            lowering.advance_pc(pc_update);
            return;
        }

        let base_reg = insn.data.transed_operands[0];
        let mut addr = emit_read_reg(lowering, base_reg);

        for &reg in &insn.data.transed_operands[1..] {
            let value = emit_read_reg(lowering, reg);
            lowering.call_void(lowering.helpers.write_u32, &[lowering.cpu_ptr, addr, value]);
            addr = lowering.builder.ins().iadd_imm(addr, 4);
        }

        if insn.data.writeback() {
            emit_write_reg(lowering, base_reg, addr);
        }

        let pc_update = emit_size_value(lowering, insn);
        lowering.advance_pc(pc_update);
    })
}

fn emit_push(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction) {
    with_cc(lowering, insn, |lowering| {
        let count = insn.data.transed_operands.len() as u32;
        let sp = emit_read_reg(lowering, 13);
        let offset = lowering.iconst_u32(count.wrapping_mul(4));
        let new_sp = lowering.builder.ins().isub(sp, offset);
        let mut addr = new_sp;

        for &reg in &insn.data.transed_operands {
            let value = emit_read_reg(lowering, reg);
            lowering.call_void(lowering.helpers.write_u32, &[lowering.cpu_ptr, addr, value]);
            addr = lowering.builder.ins().iadd_imm(addr, 4);
        }

        emit_write_reg(lowering, 13, new_sp);
        let pc_update = emit_size_value(lowering, insn);
        lowering.advance_pc(pc_update);
    })
}

fn emit_pop(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction) {
    with_cc(lowering, insn, |lowering| {
        let mut sp = emit_read_reg(lowering, 13);
        let mut popped_pc = None;

        for &reg in &insn.data.transed_operands {
            let value = lowering.call_value(lowering.helpers.read_u32, &[lowering.cpu_ptr, sp]);
            sp = lowering.builder.ins().iadd_imm(sp, 4);
            if reg == 15 {
                popped_pc = Some(value);
            } else {
                emit_write_reg(lowering, reg, value);
            }
        }

        emit_write_reg(lowering, 13, sp);

        if let Some(pc_value) = popped_pc {
            let handled = lowering.call_value(
                lowering.helpers.try_exception_return,
                &[lowering.cpu_ptr, pc_value],
            );
            let handled_block = lowering.builder.create_block();
            let continue_block = lowering.builder.create_block();
            let join_block = lowering.builder.create_block();
            lowering
                .builder
                .ins()
                .brif(handled, handled_block, &[], continue_block, &[]);

            lowering.builder.switch_to_block(handled_block);
            lowering.builder.seal_block(handled_block);
            lowering.builder.ins().jump(join_block, &[]);

            lowering.builder.switch_to_block(continue_block);
            lowering.builder.seal_block(continue_block);
            let mask = lowering.iconst_u32(!1u32);
            let aligned_pc = lowering.builder.ins().band(pc_value, mask);
            emit_write_reg(lowering, 15, aligned_pc);
            lowering.builder.ins().jump(join_block, &[]);

            lowering.builder.seal_block(join_block);
            lowering.builder.switch_to_block(join_block);
            return;
        }

        let pc_update = emit_size_value(lowering, insn);
        lowering.advance_pc(pc_update);
    })
}