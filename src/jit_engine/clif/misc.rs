use crate::arch::ArmInsn;

use crate::jit_engine::clif::instructions::{InsDef, emit_size_value, with_cc};
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
                $emit(lowering, insn)
            }
        }
    };
}

define_def!(NopDef, NOP_DEF, ARM_INS_NOP, "NOP", emit_noop);
define_def!(HintDef, HINT_DEF, ARM_INS_HINT, "Hint", emit_noop);
define_def!(ItDef, IT_DEF, ARM_INS_IT, "IT", emit_noop);
define_def!(BkptDef, BKPT_DEF, ARM_INS_BKPT, "BKPT", emit_bkpt);

pub(crate) fn find_def(insn_id: u32) -> Option<&'static dyn InsDef> {
    match insn_id {
        x if x == ArmInsn::ARM_INS_NOP as u32 => Some(&NOP_DEF),
        x if x == ArmInsn::ARM_INS_HINT as u32 => Some(&HINT_DEF),
        x if x == ArmInsn::ARM_INS_IT as u32 => Some(&IT_DEF),
        x if x == ArmInsn::ARM_INS_BKPT as u32 => Some(&BKPT_DEF),
        _ => None,
    }
}

fn emit_noop(lowering: &mut LoweringContext<'_, '_>, insn: &JitInstruction<'_>) {
    with_cc(lowering, insn, |lowering| {
        let pc_update = emit_size_value(lowering, insn);
        lowering.set_pc_update(pc_update);
    })
}

fn emit_bkpt(lowering: &mut LoweringContext<'_, '_>, _insn: &JitInstruction<'_>) {
    lowering.flush_dirty_state();
    lowering.call_void(
        lowering.helpers.execute_bkpt,
        &[lowering.cpu_ptr, lowering.instr_ptr],
    );
    let zero = lowering.iconst_u32(0);
    lowering.set_pc_update(zero);
}