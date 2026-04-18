use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::arch::ArmInsn;
use cranelift::codegen::ir::{FuncRef, StackSlot, StackSlotData, StackSlotKind};
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module, default_libcall_names};
use rustc_hash::FxHashMap;

use crate::context::CpuContext;
use crate::cpu::Cpu;
use crate::jit_engine::clif::instructions as jit_instructions;
use crate::jit_engine::table::{JitBlockRange, JitBlockTable, JitBlockTableBuilder, JitInstruction};
use crate::opcodes::decoded::{
    DecodedOperandKind, operand_resolver_multi_runtime, runtime_read_reg,
};
use crate::opcodes::thumb_runtime;

pub type JitBlockFn = unsafe extern "C" fn(*mut Cpu) -> u32;

#[derive(Debug)]
pub enum JitError {
    Backend(String),
    MissingInstruction { pc: u32 },
}

impl fmt::Display for JitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JitError::Backend(message) => write!(f, "JIT backend error: {message}"),
            JitError::MissingInstruction { pc } => {
                write!(f, "no decoded instruction for PC 0x{pc:08X}")
            }
        }
    }
}

impl std::error::Error for JitError {}

impl From<cranelift_module::ModuleError> for JitError {
    fn from(value: cranelift_module::ModuleError) -> Self {
        JitError::Backend(value.to_string())
    }
}

struct CompiledBlock {
    entry: JitBlockFn,
    end_pc: u32,
    instruction_count: usize,
}

#[derive(Clone, Copy)]
struct StepBlockCache {
    start_pc: u32,
    end_pc: u32,
    entry: JitBlockFn,
    instruction_count: usize,
}

const CACHED_REG_COUNT: usize = 16;

struct BlockStateCache {
    reg_values: [StackSlot; CACHED_REG_COUNT],
    reg_valid: [StackSlot; CACHED_REG_COUNT],
    reg_dirty: [StackSlot; CACHED_REG_COUNT],
    apsr_value: StackSlot,
    apsr_valid: StackSlot,
    apsr_dirty: StackSlot,
}

impl BlockStateCache {
    fn create_slot(builder: &mut FunctionBuilder<'_>) -> StackSlot {
        builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            4,
            2,
        ))
    }

    fn new(builder: &mut FunctionBuilder<'_>) -> Self {
        Self {
            reg_values: std::array::from_fn(|_| Self::create_slot(builder)),
            reg_valid: std::array::from_fn(|_| Self::create_slot(builder)),
            reg_dirty: std::array::from_fn(|_| Self::create_slot(builder)),
            apsr_value: Self::create_slot(builder),
            apsr_valid: Self::create_slot(builder),
            apsr_dirty: Self::create_slot(builder),
        }
    }
}

struct JitRuntimeCounters {
    finish_block_step_cycles_calls: AtomicU64,
    fallback_calls: AtomicU64,
    read_reg_calls: AtomicU64,
    write_reg_calls: AtomicU64,
    read_apsr_calls: AtomicU64,
    mem_read_calls: AtomicU64,
    mem_write_calls: AtomicU64,
    flag_update_calls: AtomicU64,
    exception_return_calls: AtomicU64,
    resolve_op2_calls: AtomicU64,
    resolve_mem_rt_addr_calls: AtomicU64,
    compute_shift_calls: AtomicU64,
    bkpt_calls: AtomicU64,
    udiv_calls: AtomicU64,
}

impl JitRuntimeCounters {
    const fn new() -> Self {
        Self {
            finish_block_step_cycles_calls: AtomicU64::new(0),
            fallback_calls: AtomicU64::new(0),
            read_reg_calls: AtomicU64::new(0),
            write_reg_calls: AtomicU64::new(0),
            read_apsr_calls: AtomicU64::new(0),
            mem_read_calls: AtomicU64::new(0),
            mem_write_calls: AtomicU64::new(0),
            flag_update_calls: AtomicU64::new(0),
            exception_return_calls: AtomicU64::new(0),
            resolve_op2_calls: AtomicU64::new(0),
            resolve_mem_rt_addr_calls: AtomicU64::new(0),
            compute_shift_calls: AtomicU64::new(0),
            bkpt_calls: AtomicU64::new(0),
            udiv_calls: AtomicU64::new(0),
        }
    }

    fn reset(&self) {
        self.finish_block_step_cycles_calls.store(0, Ordering::Relaxed);
        self.fallback_calls.store(0, Ordering::Relaxed);
        self.read_reg_calls.store(0, Ordering::Relaxed);
        self.write_reg_calls.store(0, Ordering::Relaxed);
        self.read_apsr_calls.store(0, Ordering::Relaxed);
        self.mem_read_calls.store(0, Ordering::Relaxed);
        self.mem_write_calls.store(0, Ordering::Relaxed);
        self.flag_update_calls.store(0, Ordering::Relaxed);
        self.exception_return_calls.store(0, Ordering::Relaxed);
        self.resolve_op2_calls.store(0, Ordering::Relaxed);
        self.resolve_mem_rt_addr_calls.store(0, Ordering::Relaxed);
        self.compute_shift_calls.store(0, Ordering::Relaxed);
        self.bkpt_calls.store(0, Ordering::Relaxed);
        self.udiv_calls.store(0, Ordering::Relaxed);
    }
}

static JIT_RUNTIME_COUNTERS: JitRuntimeCounters = JitRuntimeCounters::new();

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JitStatsSnapshot {
    pub compiled_blocks: u64,
    pub compiled_suffix_blocks: u64,
    pub compiled_block_instructions: u64,
    pub executed_blocks: u64,
    pub executed_block_instructions: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub finish_block_step_cycles_calls: u64,
    pub fallback_calls: u64,
    pub read_reg_calls: u64,
    pub write_reg_calls: u64,
    pub read_apsr_calls: u64,
    pub mem_read_calls: u64,
    pub mem_write_calls: u64,
    pub flag_update_calls: u64,
    pub exception_return_calls: u64,
    pub resolve_op2_calls: u64,
    pub resolve_mem_rt_addr_calls: u64,
    pub compute_shift_calls: u64,
    pub bkpt_calls: u64,
    pub udiv_calls: u64,
}

impl JitStatsSnapshot {
    pub fn delta_since(self, previous: Self) -> Self {
        Self {
            compiled_blocks: self.compiled_blocks.saturating_sub(previous.compiled_blocks),
            compiled_suffix_blocks: self
                .compiled_suffix_blocks
                .saturating_sub(previous.compiled_suffix_blocks),
            compiled_block_instructions: self
                .compiled_block_instructions
                .saturating_sub(previous.compiled_block_instructions),
            executed_blocks: self.executed_blocks.saturating_sub(previous.executed_blocks),
            executed_block_instructions: self
                .executed_block_instructions
                .saturating_sub(previous.executed_block_instructions),
            cache_hits: self.cache_hits.saturating_sub(previous.cache_hits),
            cache_misses: self.cache_misses.saturating_sub(previous.cache_misses),
            finish_block_step_cycles_calls: self
                .finish_block_step_cycles_calls
                .saturating_sub(previous.finish_block_step_cycles_calls),
            fallback_calls: self.fallback_calls.saturating_sub(previous.fallback_calls),
            read_reg_calls: self.read_reg_calls.saturating_sub(previous.read_reg_calls),
            write_reg_calls: self.write_reg_calls.saturating_sub(previous.write_reg_calls),
            read_apsr_calls: self.read_apsr_calls.saturating_sub(previous.read_apsr_calls),
            mem_read_calls: self.mem_read_calls.saturating_sub(previous.mem_read_calls),
            mem_write_calls: self.mem_write_calls.saturating_sub(previous.mem_write_calls),
            flag_update_calls: self.flag_update_calls.saturating_sub(previous.flag_update_calls),
            exception_return_calls: self
                .exception_return_calls
                .saturating_sub(previous.exception_return_calls),
            resolve_op2_calls: self.resolve_op2_calls.saturating_sub(previous.resolve_op2_calls),
            resolve_mem_rt_addr_calls: self
                .resolve_mem_rt_addr_calls
                .saturating_sub(previous.resolve_mem_rt_addr_calls),
            compute_shift_calls: self
                .compute_shift_calls
                .saturating_sub(previous.compute_shift_calls),
            bkpt_calls: self.bkpt_calls.saturating_sub(previous.bkpt_calls),
            udiv_calls: self.udiv_calls.saturating_sub(previous.udiv_calls),
        }
    }

    pub fn average_compiled_block_len(&self) -> f64 {
        if self.compiled_blocks == 0 {
            0.0
        } else {
            self.compiled_block_instructions as f64 / self.compiled_blocks as f64
        }
    }

    pub fn average_executed_block_len(&self) -> f64 {
        if self.executed_blocks == 0 {
            0.0
        } else {
            self.executed_block_instructions as f64 / self.executed_blocks as f64
        }
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    pub fn helper_calls(&self) -> u64 {
        self.finish_block_step_cycles_calls
            + self.fallback_calls
            + self.read_reg_calls
            + self.write_reg_calls
            + self.read_apsr_calls
            + self.mem_read_calls
            + self.mem_write_calls
            + self.flag_update_calls
            + self.exception_return_calls
            + self.resolve_op2_calls
            + self.resolve_mem_rt_addr_calls
            + self.compute_shift_calls
            + self.bkpt_calls
            + self.udiv_calls
    }

    pub fn helper_calls_per_guest_instruction(&self) -> f64 {
        if self.executed_block_instructions == 0 {
            0.0
        } else {
            self.helper_calls() as f64 / self.executed_block_instructions as f64
        }
    }
}

pub(crate) struct RuntimeFunctions {
    pub(crate) finish_block_step_cycles: FuncId,
    pub(crate) read_reg: FuncId,
    pub(crate) write_reg: FuncId,
    pub(crate) read_apsr: FuncId,
    pub(crate) write_apsr: FuncId,
    pub(crate) read_u8: FuncId,
    pub(crate) read_u16: FuncId,
    pub(crate) read_u32: FuncId,
    pub(crate) write_u8: FuncId,
    pub(crate) write_u16: FuncId,
    pub(crate) write_u32: FuncId,
    pub(crate) try_exception_return: FuncId,
    pub(crate) resolve_mem_rt_addr: FuncId,
    pub(crate) compute_shift_packed: FuncId,
    pub(crate) execute_bkpt: FuncId,
    pub(crate) udiv_or_zero: FuncId,
    pub(crate) fallback_exec: FuncId,
}

pub(crate) struct LoweringContext<'a, 'b> {
    pub builder: &'a mut FunctionBuilder<'b>,
    pub module: &'a mut JITModule,
    pub helpers: &'a RuntimeFunctions,
    cache: &'a BlockStateCache,
    pub ptr_ty: Type,
    pub cpu_ptr: Value,
    pub instr_ptr: Value,
    pub current_pc: Value,
}

impl<'a, 'b> LoweringContext<'a, 'b> {
    pub(crate) fn iconst_u32(&mut self, value: u32) -> Value {
        self.builder
            .ins()
            .iconst(types::I32, i64::from(value as i32))
    }

    pub(crate) fn iconst_i32(&mut self, value: i32) -> Value {
        self.builder.ins().iconst(types::I32, i64::from(value))
    }

    pub(crate) fn iconst_ptr(&mut self, value: *const ()) -> Value {
        self.builder
            .ins()
            .iconst(self.ptr_ty, value as usize as i64)
    }

    pub(crate) fn import_func(&mut self, func_id: FuncId) -> FuncRef {
        self.module.declare_func_in_func(func_id, self.builder.func)
    }

    pub(crate) fn call_value(&mut self, func_id: FuncId, args: &[Value]) -> Value {
        let func_ref = self.import_func(func_id);
        let call = self.builder.ins().call(func_ref, args);
        self.builder.inst_results(call)[0]
    }

    pub(crate) fn call_void(&mut self, func_id: FuncId, args: &[Value]) {
        let func_ref = self.import_func(func_id);
        self.builder.ins().call(func_ref, args);
    }

    fn ptr_cast_u32(&mut self, value: Value) -> Value {
        if self.ptr_ty == types::I32 {
            value
        } else {
            self.builder.ins().uextend(self.ptr_ty, value)
        }
    }

    fn load_cpu_i32(&mut self, offset: i32) -> Value {
        self.builder
            .ins()
            .load(types::I32, MemFlags::new(), self.cpu_ptr, offset)
    }

    fn load_cpu_ptr(&mut self, offset: i32) -> Value {
        self.builder
            .ins()
            .load(self.ptr_ty, MemFlags::new(), self.cpu_ptr, offset)
    }

    fn store_cpu_i32(&mut self, offset: i32, value: Value) {
        self.builder
            .ins()
            .store(MemFlags::new(), value, self.cpu_ptr, offset);
    }

    pub(crate) fn load_cpu_reg(&mut self, reg: u32) -> Value {
        self.load_cpu_i32(Cpu::jit_reg_offset(reg))
    }

    pub(crate) fn load_dynamic_cpu_reg(&mut self, reg: Value) -> Value {
        let reg_index = self.ptr_cast_u32(reg);
        let byte_offset = self.builder.ins().ishl_imm(reg_index, 2);
        let reg_base = self
            .builder
            .ins()
            .iconst(self.ptr_ty, i64::from(Cpu::jit_reg_base_offset()));
        let total_offset = self.builder.ins().iadd(byte_offset, reg_base);
        let addr = self.builder.ins().iadd(self.cpu_ptr, total_offset);
        self.builder
            .ins()
            .load(types::I32, MemFlags::new(), addr, 0)
    }

    pub(crate) fn store_cpu_reg(&mut self, reg: u32, value: Value) {
        self.store_cpu_i32(Cpu::jit_reg_offset(reg), value);
    }

    fn store_buffer_u32(&mut self, base_ptr: Value, byte_offset: Value, value: Value) {
        let ptr_offset = self.ptr_cast_u32(byte_offset);
        let addr = self.builder.ins().iadd(base_ptr, ptr_offset);
        self.builder.ins().store(MemFlags::new(), value, addr, 0);
    }

    fn load_buffer_u32(&mut self, base_ptr: Value, byte_offset: Value) -> Value {
        let ptr_offset = self.ptr_cast_u32(byte_offset);
        let addr = self.builder.ins().iadd(base_ptr, ptr_offset);
        self.builder
            .ins()
            .load(types::I32, MemFlags::new(), addr, 0)
    }

    fn emit_buffer_masked_write(
        &mut self,
        base_ptr: Value,
        aligned_offset: Value,
        shift: Value,
        mask: u32,
        value: Value,
    ) {
        let current = self.load_buffer_u32(base_ptr, aligned_offset);
        let mask_value = self.iconst_u32(mask);
        let dynamic_mask = self.builder.ins().ishl(mask_value, shift);
        let keep_mask = self.builder.ins().bnot(dynamic_mask);
        let preserved = self.builder.ins().band(current, keep_mask);
        let masked_value = self.builder.ins().band(value, mask_value);
        let shifted_value = self.builder.ins().ishl(masked_value, shift);
        let updated = self.builder.ins().bor(preserved, shifted_value);
        self.store_buffer_u32(base_ptr, aligned_offset, updated);
    }

    fn classify_flash_or_alias(
        &mut self,
        addr: Value,
        flash_offset_max: i64,
    ) -> (Value, Value, Value) {
        let flash_base = self.iconst_u32(Cpu::jit_flash_base());
        let flash_offset = self.builder.ins().isub(addr, flash_base);
        let is_flash = self
            .builder
            .ins()
            .icmp_imm(IntCC::UnsignedLessThanOrEqual, flash_offset, flash_offset_max);

        let flash_alias_base = self.iconst_u32(Cpu::jit_flash_alias_base());
        let flash_alias_offset = self.builder.ins().isub(addr, flash_alias_base);
        let is_flash_alias = self.builder.ins().icmp_imm(
            IntCC::UnsignedLessThanOrEqual,
            flash_alias_offset,
            flash_offset_max,
        );

        let is_flash_or_alias = self.builder.ins().bor(is_flash, is_flash_alias);
        let selected_offset = self
            .builder
            .ins()
            .select(is_flash, flash_offset, flash_alias_offset);
        (is_flash, is_flash_or_alias, selected_offset)
    }

    pub(crate) fn emit_read_u32(&mut self, addr: Value) -> Value {
        let ram_offset_max = i64::from(Cpu::jit_ram_len().saturating_sub(4));
        let flash_offset_max = i64::from(Cpu::jit_flash_len().saturating_sub(4));

        let ram_base = self.iconst_u32(Cpu::jit_ram_base());
        let ram_offset = self.builder.ins().isub(addr, ram_base);
        let is_ram = self
            .builder
            .ins()
            .icmp_imm(IntCC::UnsignedLessThanOrEqual, ram_offset, ram_offset_max);

        let (_is_flash, is_flash_or_alias, flash_or_alias_offset) =
            self.classify_flash_or_alias(addr, flash_offset_max);

        let ram_block = self.builder.create_block();
        let not_ram_block = self.builder.create_block();
        let flash_or_alias_block = self.builder.create_block();
        let fallback_block = self.builder.create_block();
        let join_block = self.builder.create_block();
        self.builder.append_block_param(join_block, types::I32);

        self.builder
            .ins()
            .brif(is_ram, ram_block, &[], not_ram_block, &[]);

        self.builder.switch_to_block(ram_block);
        self.builder.seal_block(ram_block);
        let ram_ptr = self.load_cpu_ptr(Cpu::jit_ram_ptr_offset());
        let ram_value = self.load_buffer_u32(ram_ptr, ram_offset);
        self.builder.ins().jump(join_block, &[ram_value.into()]);

        self.builder.switch_to_block(not_ram_block);
        self.builder.seal_block(not_ram_block);
        self.builder
            .ins()
            .brif(is_flash_or_alias, flash_or_alias_block, &[], fallback_block, &[]);

        self.builder.switch_to_block(flash_or_alias_block);
        self.builder.seal_block(flash_or_alias_block);
        let flash_ptr = self.load_cpu_ptr(Cpu::jit_flash_ptr_offset());
        let flash_alias_value = self.load_buffer_u32(flash_ptr, flash_or_alias_offset);
        self.builder
            .ins()
            .jump(join_block, &[flash_alias_value.into()]);

        self.builder.switch_to_block(fallback_block);
        self.builder.seal_block(fallback_block);
        let value = self.call_value(self.helpers.read_u32, &[self.cpu_ptr, addr]);
        self.builder.ins().jump(join_block, &[value.into()]);

        self.builder.seal_block(join_block);
        self.builder.switch_to_block(join_block);
        self.builder.block_params(join_block)[0]
    }

    pub(crate) fn emit_read_u8(&mut self, addr: Value) -> Value {
        let align_mask = self.iconst_u32(!3u32);
        let aligned = self.builder.ins().band(addr, align_mask);
        let word = self.emit_read_u32(aligned);
        let byte_index = self.builder.ins().band_imm(addr, 3);
        let shift = self.builder.ins().ishl_imm(byte_index, 3);
        let shifted = self.builder.ins().ushr(word, shift);
        let mask = self.iconst_u32(0xFF);
        self.builder.ins().band(shifted, mask)
    }

    pub(crate) fn emit_read_u16(&mut self, addr: Value) -> Value {
        let align_mask = self.iconst_u32(!3u32);
        let aligned = self.builder.ins().band(addr, align_mask);
        let word = self.emit_read_u32(aligned);
        let halfword_index = self.builder.ins().band_imm(addr, 2);
        let shift = self.builder.ins().ishl_imm(halfword_index, 3);
        let shifted = self.builder.ins().ushr(word, shift);
        let mask = self.iconst_u32(0xFFFF);
        self.builder.ins().band(shifted, mask)
    }

    pub(crate) fn emit_write_u8(&mut self, addr: Value, value: Value) {
        let ram_offset_max = i64::from(Cpu::jit_ram_len().saturating_sub(4));
        let flash_offset_max = i64::from(Cpu::jit_flash_len().saturating_sub(4));
        let align_mask = self.iconst_u32(!3u32);
        let aligned_addr = self.builder.ins().band(addr, align_mask);

        let ram_base = self.iconst_u32(Cpu::jit_ram_base());
        let ram_offset = self.builder.ins().isub(aligned_addr, ram_base);
        let is_ram = self
            .builder
            .ins()
            .icmp_imm(IntCC::UnsignedLessThanOrEqual, ram_offset, ram_offset_max);

        let ram_block = self.builder.create_block();
        let not_ram_block = self.builder.create_block();
        let flash_block = self.builder.create_block();
        let not_flash_block = self.builder.create_block();
        let flash_alias_block = self.builder.create_block();
        let fallback_block = self.builder.create_block();
        let join_block = self.builder.create_block();

        self.builder
            .ins()
            .brif(is_ram, ram_block, &[], not_ram_block, &[]);

        self.builder.switch_to_block(ram_block);
        self.builder.seal_block(ram_block);
        let ram_ptr = self.load_cpu_ptr(Cpu::jit_ram_ptr_offset());
        let byte_index = self.builder.ins().band_imm(addr, 3);
        let shift = self.builder.ins().ishl_imm(byte_index, 3);
        self.emit_buffer_masked_write(ram_ptr, ram_offset, shift, 0xFF, value);
        self.builder.ins().jump(join_block, &[]);

        self.builder.switch_to_block(not_ram_block);
        self.builder.seal_block(not_ram_block);
        let flash_base = self.iconst_u32(Cpu::jit_flash_base());
        let flash_offset = self.builder.ins().isub(aligned_addr, flash_base);
        let is_flash = self
            .builder
            .ins()
            .icmp_imm(IntCC::UnsignedLessThanOrEqual, flash_offset, flash_offset_max);
        self.builder
            .ins()
            .brif(is_flash, flash_block, &[], not_flash_block, &[]);

        self.builder.switch_to_block(flash_block);
        self.builder.seal_block(flash_block);
        let flash_ptr = self.load_cpu_ptr(Cpu::jit_flash_ptr_offset());
        let byte_index = self.builder.ins().band_imm(addr, 3);
        let shift = self.builder.ins().ishl_imm(byte_index, 3);
        self.emit_buffer_masked_write(flash_ptr, flash_offset, shift, 0xFF, value);
        self.builder.ins().jump(join_block, &[]);

        self.builder.switch_to_block(not_flash_block);
        self.builder.seal_block(not_flash_block);
        let flash_alias_base = self.iconst_u32(Cpu::jit_flash_alias_base());
        let flash_alias_offset = self.builder.ins().isub(aligned_addr, flash_alias_base);
        let is_flash_alias = self.builder.ins().icmp_imm(
            IntCC::UnsignedLessThanOrEqual,
            flash_alias_offset,
            flash_offset_max,
        );
        self.builder
            .ins()
            .brif(is_flash_alias, flash_alias_block, &[], fallback_block, &[]);

        self.builder.switch_to_block(flash_alias_block);
        self.builder.seal_block(flash_alias_block);
        self.builder.ins().jump(join_block, &[]);

        self.builder.switch_to_block(fallback_block);
        self.builder.seal_block(fallback_block);
        self.call_void(self.helpers.write_u8, &[self.cpu_ptr, addr, value]);
        self.builder.ins().jump(join_block, &[]);

        self.builder.seal_block(join_block);
        self.builder.switch_to_block(join_block);
    }

    pub(crate) fn emit_write_u16(&mut self, addr: Value, value: Value) {
        let ram_offset_max = i64::from(Cpu::jit_ram_len().saturating_sub(4));
        let flash_offset_max = i64::from(Cpu::jit_flash_len().saturating_sub(4));
        let align_mask = self.iconst_u32(!3u32);
        let aligned_addr = self.builder.ins().band(addr, align_mask);

        let ram_base = self.iconst_u32(Cpu::jit_ram_base());
        let ram_offset = self.builder.ins().isub(aligned_addr, ram_base);
        let is_ram = self
            .builder
            .ins()
            .icmp_imm(IntCC::UnsignedLessThanOrEqual, ram_offset, ram_offset_max);

        let ram_block = self.builder.create_block();
        let not_ram_block = self.builder.create_block();
        let flash_block = self.builder.create_block();
        let not_flash_block = self.builder.create_block();
        let flash_alias_block = self.builder.create_block();
        let fallback_block = self.builder.create_block();
        let join_block = self.builder.create_block();

        self.builder
            .ins()
            .brif(is_ram, ram_block, &[], not_ram_block, &[]);

        self.builder.switch_to_block(ram_block);
        self.builder.seal_block(ram_block);
        let ram_ptr = self.load_cpu_ptr(Cpu::jit_ram_ptr_offset());
        let halfword_index = self.builder.ins().band_imm(addr, 2);
        let shift = self.builder.ins().ishl_imm(halfword_index, 3);
        self.emit_buffer_masked_write(ram_ptr, ram_offset, shift, 0xFFFF, value);
        self.builder.ins().jump(join_block, &[]);

        self.builder.switch_to_block(not_ram_block);
        self.builder.seal_block(not_ram_block);
        let flash_base = self.iconst_u32(Cpu::jit_flash_base());
        let flash_offset = self.builder.ins().isub(aligned_addr, flash_base);
        let is_flash = self
            .builder
            .ins()
            .icmp_imm(IntCC::UnsignedLessThanOrEqual, flash_offset, flash_offset_max);
        self.builder
            .ins()
            .brif(is_flash, flash_block, &[], not_flash_block, &[]);

        self.builder.switch_to_block(flash_block);
        self.builder.seal_block(flash_block);
        let flash_ptr = self.load_cpu_ptr(Cpu::jit_flash_ptr_offset());
        let halfword_index = self.builder.ins().band_imm(addr, 2);
        let shift = self.builder.ins().ishl_imm(halfword_index, 3);
        self.emit_buffer_masked_write(flash_ptr, flash_offset, shift, 0xFFFF, value);
        self.builder.ins().jump(join_block, &[]);

        self.builder.switch_to_block(not_flash_block);
        self.builder.seal_block(not_flash_block);
        let flash_alias_base = self.iconst_u32(Cpu::jit_flash_alias_base());
        let flash_alias_offset = self.builder.ins().isub(aligned_addr, flash_alias_base);
        let is_flash_alias = self.builder.ins().icmp_imm(
            IntCC::UnsignedLessThanOrEqual,
            flash_alias_offset,
            flash_offset_max,
        );
        self.builder
            .ins()
            .brif(is_flash_alias, flash_alias_block, &[], fallback_block, &[]);

        self.builder.switch_to_block(flash_alias_block);
        self.builder.seal_block(flash_alias_block);
        self.builder.ins().jump(join_block, &[]);

        self.builder.switch_to_block(fallback_block);
        self.builder.seal_block(fallback_block);
        self.call_void(self.helpers.write_u16, &[self.cpu_ptr, addr, value]);
        self.builder.ins().jump(join_block, &[]);

        self.builder.seal_block(join_block);
        self.builder.switch_to_block(join_block);
    }

    pub(crate) fn emit_write_u32(&mut self, addr: Value, value: Value) {
        let ram_offset_max = i64::from(Cpu::jit_ram_len().saturating_sub(4));
        let flash_offset_max = i64::from(Cpu::jit_flash_len().saturating_sub(4));

        let ram_base = self.iconst_u32(Cpu::jit_ram_base());
        let ram_offset = self.builder.ins().isub(addr, ram_base);
        let is_ram = self
            .builder
            .ins()
            .icmp_imm(IntCC::UnsignedLessThanOrEqual, ram_offset, ram_offset_max);

        let ram_block = self.builder.create_block();
        let not_ram_block = self.builder.create_block();
        let flash_block = self.builder.create_block();
        let not_flash_block = self.builder.create_block();
        let flash_alias_block = self.builder.create_block();
        let fallback_block = self.builder.create_block();
        let join_block = self.builder.create_block();

        self.builder
            .ins()
            .brif(is_ram, ram_block, &[], not_ram_block, &[]);

        self.builder.switch_to_block(ram_block);
        self.builder.seal_block(ram_block);
        let ram_ptr = self.load_cpu_ptr(Cpu::jit_ram_ptr_offset());
        self.store_buffer_u32(ram_ptr, ram_offset, value);
        self.builder.ins().jump(join_block, &[]);

        self.builder.switch_to_block(not_ram_block);
        self.builder.seal_block(not_ram_block);
        let flash_base = self.iconst_u32(Cpu::jit_flash_base());
        let flash_offset = self.builder.ins().isub(addr, flash_base);
        let is_flash = self
            .builder
            .ins()
            .icmp_imm(IntCC::UnsignedLessThanOrEqual, flash_offset, flash_offset_max);
        self.builder
            .ins()
            .brif(is_flash, flash_block, &[], not_flash_block, &[]);

        self.builder.switch_to_block(flash_block);
        self.builder.seal_block(flash_block);
        let flash_ptr = self.load_cpu_ptr(Cpu::jit_flash_ptr_offset());
        self.store_buffer_u32(flash_ptr, flash_offset, value);
        self.builder.ins().jump(join_block, &[]);

        self.builder.switch_to_block(not_flash_block);
        self.builder.seal_block(not_flash_block);
        let flash_alias_base = self.iconst_u32(Cpu::jit_flash_alias_base());
        let flash_alias_offset = self.builder.ins().isub(addr, flash_alias_base);
        let is_flash_alias = self.builder.ins().icmp_imm(
            IntCC::UnsignedLessThanOrEqual,
            flash_alias_offset,
            flash_offset_max,
        );
        self.builder
            .ins()
            .brif(is_flash_alias, flash_alias_block, &[], fallback_block, &[]);

        self.builder.switch_to_block(flash_alias_block);
        self.builder.seal_block(flash_alias_block);
        self.builder.ins().jump(join_block, &[]);

        self.builder.switch_to_block(fallback_block);
        self.builder.seal_block(fallback_block);
        self.call_void(self.helpers.write_u32, &[self.cpu_ptr, addr, value]);
        self.builder.ins().jump(join_block, &[]);

        self.builder.seal_block(join_block);
        self.builder.switch_to_block(join_block);
    }

    fn load_i32_slot(&mut self, slot: StackSlot) -> Value {
        self.builder.ins().stack_load(types::I32, slot, 0)
    }

    fn store_i32_slot(&mut self, slot: StackSlot, value: Value) {
        self.builder.ins().stack_store(value, slot, 0);
    }

    fn set_slot_flag(&mut self, slot: StackSlot, flag: bool) {
        let value = self.iconst_u32(u32::from(flag));
        self.store_i32_slot(slot, value);
    }

    pub(crate) fn initialize_cache_state(&mut self) {
        for index in 0..CACHED_REG_COUNT {
            self.set_slot_flag(self.cache.reg_valid[index], false);
            self.set_slot_flag(self.cache.reg_dirty[index], false);
        }
        self.set_slot_flag(self.cache.apsr_valid, false);
        self.set_slot_flag(self.cache.apsr_dirty, false);
    }

    pub(crate) fn current_pc_plus_4(&mut self) -> Value {
        let offset = self.iconst_u32(4);
        self.builder.ins().iadd(self.current_pc, offset)
    }

    pub(crate) fn read_cached_reg(&mut self, reg: u32) -> Value {
        if reg == 15 {
            return self.current_pc_plus_4();
        }

        let valid_slot = self.cache.reg_valid[reg as usize];
        let value_slot = self.cache.reg_values[reg as usize];
        let valid = self.load_i32_slot(valid_slot);
        let is_valid = self.builder.ins().icmp_imm(IntCC::NotEqual, valid, 0);
        let hit_block = self.builder.create_block();
        let miss_block = self.builder.create_block();
        let join_block = self.builder.create_block();
        self.builder.append_block_param(join_block, types::I32);
        self.builder
            .ins()
            .brif(is_valid, hit_block, &[], miss_block, &[]);

        self.builder.switch_to_block(hit_block);
        self.builder.seal_block(hit_block);
        let cached = self.load_i32_slot(value_slot);
        self.builder.ins().jump(join_block, &[cached.into()]);

        self.builder.switch_to_block(miss_block);
        self.builder.seal_block(miss_block);
        let loaded = self.load_cpu_reg(reg);
        self.store_i32_slot(value_slot, loaded);
        self.set_slot_flag(valid_slot, true);
        self.builder.ins().jump(join_block, &[loaded.into()]);

        self.builder.seal_block(join_block);
        self.builder.switch_to_block(join_block);
        self.builder.block_params(join_block)[0]
    }

    pub(crate) fn write_cached_reg(&mut self, reg: u32, value: Value) {
        let value_slot = self.cache.reg_values[reg as usize];
        let valid_slot = self.cache.reg_valid[reg as usize];
        let dirty_slot = self.cache.reg_dirty[reg as usize];
        self.store_i32_slot(value_slot, value);
        self.set_slot_flag(valid_slot, true);
        self.set_slot_flag(dirty_slot, true);
    }

    pub(crate) fn read_cached_apsr(&mut self) -> Value {
        let valid = self.load_i32_slot(self.cache.apsr_valid);
        let is_valid = self.builder.ins().icmp_imm(IntCC::NotEqual, valid, 0);
        let hit_block = self.builder.create_block();
        let miss_block = self.builder.create_block();
        let join_block = self.builder.create_block();
        self.builder.append_block_param(join_block, types::I32);
        self.builder
            .ins()
            .brif(is_valid, hit_block, &[], miss_block, &[]);

        self.builder.switch_to_block(hit_block);
        self.builder.seal_block(hit_block);
        let cached = self.load_i32_slot(self.cache.apsr_value);
        self.builder.ins().jump(join_block, &[cached.into()]);

        self.builder.switch_to_block(miss_block);
        self.builder.seal_block(miss_block);
        let loaded = self.call_value(self.helpers.read_apsr, &[self.cpu_ptr]);
        self.store_i32_slot(self.cache.apsr_value, loaded);
        self.set_slot_flag(self.cache.apsr_valid, true);
        self.builder.ins().jump(join_block, &[loaded.into()]);

        self.builder.seal_block(join_block);
        self.builder.switch_to_block(join_block);
        self.builder.block_params(join_block)[0]
    }

    pub(crate) fn write_cached_apsr(&mut self, value: Value) {
        self.store_i32_slot(self.cache.apsr_value, value);
        self.set_slot_flag(self.cache.apsr_valid, true);
        self.set_slot_flag(self.cache.apsr_dirty, true);
    }

    pub(crate) fn flush_dirty_state(&mut self) {
        for reg in 0..CACHED_REG_COUNT as u32 {
            let dirty_slot = self.cache.reg_dirty[reg as usize];
            let dirty = self.load_i32_slot(dirty_slot);
            let is_dirty = self.builder.ins().icmp_imm(IntCC::NotEqual, dirty, 0);
            let flush_block = self.builder.create_block();
            let continue_block = self.builder.create_block();
            self.builder
                .ins()
                .brif(is_dirty, flush_block, &[], continue_block, &[]);

            self.builder.switch_to_block(flush_block);
            self.builder.seal_block(flush_block);
            let value = self.load_i32_slot(self.cache.reg_values[reg as usize]);
            self.store_cpu_reg(reg, value);
            self.set_slot_flag(dirty_slot, false);
            self.builder.ins().jump(continue_block, &[]);

            self.builder.switch_to_block(continue_block);
            self.builder.seal_block(continue_block);
        }

        let apsr_dirty = self.load_i32_slot(self.cache.apsr_dirty);
        let apsr_is_dirty = self.builder.ins().icmp_imm(IntCC::NotEqual, apsr_dirty, 0);
        let flush_block = self.builder.create_block();
        let continue_block = self.builder.create_block();
        self.builder
            .ins()
            .brif(apsr_is_dirty, flush_block, &[], continue_block, &[]);

        self.builder.switch_to_block(flush_block);
        self.builder.seal_block(flush_block);
        let apsr = self.load_i32_slot(self.cache.apsr_value);
        self.call_void(self.helpers.write_apsr, &[self.cpu_ptr, apsr]);
        self.set_slot_flag(self.cache.apsr_dirty, false);
        self.builder.ins().jump(continue_block, &[]);

        self.builder.switch_to_block(continue_block);
        self.builder.seal_block(continue_block);
    }

    pub(crate) fn emit_fallback(&mut self) -> Value {
        self.flush_dirty_state();
        let visible_pc = self.current_pc_plus_4();
        self.store_cpu_reg(15, visible_pc);
        self.call_value(self.helpers.fallback_exec, &[self.cpu_ptr, self.instr_ptr])
    }

    pub(crate) fn apply_pc_update(&mut self, pc_update: Value) {
        let needs_update = self.builder.ins().icmp_imm(IntCC::NotEqual, pc_update, 0);
        let update_block = self.builder.create_block();
        let continue_block = self.builder.create_block();
        self.builder
            .ins()
            .brif(needs_update, update_block, &[], continue_block, &[]);

        self.builder.switch_to_block(update_block);
        self.builder.seal_block(update_block);
        let next_pc = self.builder.ins().iadd(self.current_pc, pc_update);
        self.write_cached_reg(15, next_pc);
        self.builder.ins().jump(continue_block, &[]);

        self.builder.switch_to_block(continue_block);
        self.builder.seal_block(continue_block);
    }

    pub(crate) fn advance_pc(&mut self, update: Value) {
        self.apply_pc_update(update);
    }

    pub(crate) fn advance_pc_for_insn(&mut self, insn: &JitInstruction) {
        let update = self.iconst_u32(insn.data.size());
        self.apply_pc_update(update);
    }

    pub(crate) fn advance_pc_for_rd(&mut self, insn: &JitInstruction, rd: u32) {
        if rd != 15 {
            self.advance_pc_for_insn(insn);
        }
    }
}

pub struct JitEngine {
    module: JITModule,
    builder_ctx: FunctionBuilderContext,
    helpers: RuntimeFunctions,
    blocks: FxHashMap<u32, CompiledBlock>,
    step_block_cache: Option<StepBlockCache>,
    next_function_index: u32,
    compiled_blocks: u64,
    compiled_suffix_blocks: u64,
    compiled_block_instructions: u64,
    executed_blocks: u64,
    executed_block_instructions: u64,
    cache_hits: u64,
    cache_misses: u64,
}

impl JitEngine {
    pub fn new() -> Result<Self, JitError> {
        JIT_RUNTIME_COUNTERS.reset();
        let mut builder = JITBuilder::new(default_libcall_names())
            .map_err(|err| JitError::Backend(err.to_string()))?;
        builder.symbol(
            "jit_finish_block_step_cycles",
            jit_finish_block_step_cycles as *const u8,
        );
        builder.symbol("jit_read_reg", jit_read_reg as *const u8);
        builder.symbol("jit_write_reg", jit_write_reg as *const u8);
        builder.symbol("jit_read_apsr", jit_read_apsr as *const u8);
        builder.symbol("jit_write_apsr", jit_write_apsr as *const u8);
        builder.symbol("jit_read_u8", jit_read_u8 as *const u8);
        builder.symbol("jit_read_u16", jit_read_u16 as *const u8);
        builder.symbol("jit_read_u32", jit_read_u32 as *const u8);
        builder.symbol("jit_write_u8", jit_write_u8 as *const u8);
        builder.symbol("jit_write_u16", jit_write_u16 as *const u8);
        builder.symbol("jit_write_u32", jit_write_u32 as *const u8);
        builder.symbol(
            "jit_try_exception_return",
            jit_try_exception_return as *const u8,
        );
        builder.symbol(
            "jit_resolve_mem_rt_addr",
            jit_resolve_mem_rt_addr as *const u8,
        );
        builder.symbol(
            "jit_compute_shift_packed",
            jit_compute_shift_packed as *const u8,
        );
        builder.symbol("jit_execute_bkpt", jit_execute_bkpt as *const u8);
        builder.symbol("jit_udiv_or_zero", jit_udiv_or_zero as *const u8);
        builder.symbol("jit_execute_fallback", jit_execute_fallback as *const u8);

        let mut module = JITModule::new(builder);
        let helpers = RuntimeFunctions::declare(&mut module)?;

        Ok(Self {
            module,
            builder_ctx: FunctionBuilderContext::new(),
            helpers,
            blocks: FxHashMap::default(),
            step_block_cache: None,
            next_function_index: 0,
            compiled_blocks: 0,
            compiled_suffix_blocks: 0,
            compiled_block_instructions: 0,
            executed_blocks: 0,
            executed_block_instructions: 0,
            cache_hits: 0,
            cache_misses: 0,
        })
    }

    pub fn stats_snapshot(&self) -> JitStatsSnapshot {
        JitStatsSnapshot {
            compiled_blocks: self.compiled_blocks,
            compiled_suffix_blocks: self.compiled_suffix_blocks,
            compiled_block_instructions: self.compiled_block_instructions,
            executed_blocks: self.executed_blocks,
            executed_block_instructions: self.executed_block_instructions,
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            finish_block_step_cycles_calls: JIT_RUNTIME_COUNTERS
                .finish_block_step_cycles_calls
                .load(Ordering::Relaxed),
            fallback_calls: JIT_RUNTIME_COUNTERS.fallback_calls.load(Ordering::Relaxed),
            read_reg_calls: JIT_RUNTIME_COUNTERS.read_reg_calls.load(Ordering::Relaxed),
            write_reg_calls: JIT_RUNTIME_COUNTERS.write_reg_calls.load(Ordering::Relaxed),
            read_apsr_calls: JIT_RUNTIME_COUNTERS.read_apsr_calls.load(Ordering::Relaxed),
            mem_read_calls: JIT_RUNTIME_COUNTERS.mem_read_calls.load(Ordering::Relaxed),
            mem_write_calls: JIT_RUNTIME_COUNTERS.mem_write_calls.load(Ordering::Relaxed),
            flag_update_calls: JIT_RUNTIME_COUNTERS
                .flag_update_calls
                .load(Ordering::Relaxed),
            exception_return_calls: JIT_RUNTIME_COUNTERS
                .exception_return_calls
                .load(Ordering::Relaxed),
            resolve_op2_calls: JIT_RUNTIME_COUNTERS.resolve_op2_calls.load(Ordering::Relaxed),
            resolve_mem_rt_addr_calls: JIT_RUNTIME_COUNTERS
                .resolve_mem_rt_addr_calls
                .load(Ordering::Relaxed),
            compute_shift_calls: JIT_RUNTIME_COUNTERS
                .compute_shift_calls
                .load(Ordering::Relaxed),
            bkpt_calls: JIT_RUNTIME_COUNTERS.bkpt_calls.load(Ordering::Relaxed),
            udiv_calls: JIT_RUNTIME_COUNTERS.udiv_calls.load(Ordering::Relaxed),
        }
    }

    pub fn compiled_block_count(&self) -> usize {
        self.blocks.len()
    }

    pub fn compiled_entry(&self, pc: u32) -> Option<JitBlockFn> {
        self.blocks.get(&pc).map(|block| block.entry)
    }

    pub fn compiled_entries(&self) -> Vec<(u32, JitBlockFn)> {
        let mut entries: Vec<_> = self
            .blocks
            .iter()
            .map(|(pc, block)| (*pc, block.entry))
            .collect();
        entries.sort_unstable_by_key(|(pc, _)| *pc);
        entries
    }

    pub fn compile_table(&mut self, table: &JitBlockTable) -> Result<Vec<(u32, JitBlockFn)>, JitError> {
        for block in table.iter_blocks() {
            if self
                .blocks
                .get(&block.start_pc)
                .is_some_and(|compiled| compiled.end_pc == block.end_pc)
            {
                continue;
            }

            let compiled = self.compile_block_from_pc(table, block, block.start_pc)?;
            self.blocks.insert(block.start_pc, compiled);
            if self
                .step_block_cache
                .as_ref()
                .is_some_and(|cached| cached.start_pc == block.start_pc && cached.end_pc != block.end_pc)
            {
                self.step_block_cache = None;
            }
        }

        Ok(self.compiled_entries())
    }

    pub fn step(&mut self, cpu: &mut Cpu, table: &JitBlockTable) -> Result<u32, JitError> {
        if let Some(cycles) = cpu.begin_step() {
            return Ok(cycles);
        }

        let current_pc = cpu.next_pc;
        let block = table
            .block_containing(current_pc)
            .ok_or(JitError::MissingInstruction { pc: current_pc })?;
        self.execute_block(cpu, table, current_pc, block)
    }

    #[inline(always)]
    pub fn step_block(
        &mut self,
        cpu: &mut Cpu,
        table: &JitBlockTable,
        start_pc: u32,
        block: &JitBlockRange,
    ) -> Result<u32, JitError> {
        if let Some(cached) = self
            .step_block_cache
            .as_ref()
            .filter(|cached| cached.start_pc == start_pc && cached.end_pc == block.end_pc)
            .copied()
        {
            self.cache_hits = self.cache_hits.saturating_add(1);
            self.executed_blocks = self.executed_blocks.saturating_add(1);
            self.executed_block_instructions = self
                .executed_block_instructions
                .saturating_add(cached.instruction_count as u64);
            return Ok(unsafe { (cached.entry)(cpu as *mut Cpu) });
        }

        self.execute_block(cpu, table, start_pc, block)
    }

    #[inline(always)]
    pub fn step_block_builder(
        &mut self,
        cpu: &mut Cpu,
        builder: &JitBlockTableBuilder,
        start_pc: u32,
        block: &JitBlockRange,
    ) -> Result<u32, JitError> {
        if let Some(cached) = self
            .step_block_cache
            .as_ref()
            .filter(|cached| cached.start_pc == start_pc && cached.end_pc == block.end_pc)
            .copied()
        {
            self.cache_hits = self.cache_hits.saturating_add(1);
            self.executed_blocks = self.executed_blocks.saturating_add(1);
            self.executed_block_instructions = self
                .executed_block_instructions
                .saturating_add(cached.instruction_count as u64);
            return Ok(unsafe { (cached.entry)(cpu as *mut Cpu) });
        }

        self.execute_block_from_builder(cpu, builder, start_pc, block)
    }

    #[inline(always)]
    pub fn try_step_cached_block_builder(
        &mut self,
        cpu: &mut Cpu,
        builder: &mut JitBlockTableBuilder,
        start_pc: u32,
    ) -> Result<Option<(u32, usize)>, JitError> {
        let Some(compiled) = self.blocks.get(&start_pc) else {
            return Ok(None);
        };

        let Some(block) = builder.block_starting_at(start_pc) else {
            if self
                .step_block_cache
                .as_ref()
                .is_some_and(|cached| cached.start_pc == start_pc)
            {
                self.step_block_cache = None;
            }
            return Ok(None);
        };

        if compiled.end_pc != block.end_pc {
            if self
                .step_block_cache
                .as_ref()
                .is_some_and(|cached| cached.start_pc == start_pc)
            {
                self.step_block_cache = None;
            }
            return Ok(None);
        }

        let cached = StepBlockCache {
            start_pc,
            end_pc: compiled.end_pc,
            entry: compiled.entry,
            instruction_count: compiled.instruction_count,
        };
        self.step_block_cache = Some(cached);
        self.cache_hits = self.cache_hits.saturating_add(1);
        self.executed_blocks = self.executed_blocks.saturating_add(1);
        self.executed_block_instructions = self
            .executed_block_instructions
            .saturating_add(cached.instruction_count as u64);
        let cycles = unsafe { (cached.entry)(cpu as *mut Cpu) };
        Ok(Some((cycles, block.instruction_count)))
    }

    pub fn step_resolved(
        &mut self,
        cpu: &mut Cpu,
        table: &JitBlockTable,
        start_pc: u32,
        _start_ins: &JitInstruction,
        block: &JitBlockRange,
    ) -> Result<u32, JitError> {
        self.execute_block(cpu, table, start_pc, block)
    }

    pub fn run(&mut self, cpu: &mut Cpu, table: &JitBlockTable) -> Result<(), JitError> {
        const DEFAULT_REPORT_WINDOW: u32 = 10_000;

        let report_window = std::env::var("SIM_REPORT_WINDOW")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(DEFAULT_REPORT_WINDOW);

        let no_throttle = std::env::var("SIM_NO_THROTTLE")
            .map(|v| v != "0")
            .unwrap_or(false);
        let peripheral_tick_batch = std::env::var("SIM_PERIPH_TICK_BATCH")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(1);

        println!(
            "Simulator mode: JIT | throttle: {} | periph batch: {}",
            if no_throttle { "OFF" } else { "ON" },
            peripheral_tick_batch
        );

        self.run_fast(
            cpu,
            table,
            no_throttle,
            peripheral_tick_batch,
            report_window,
        )
    }

    fn run_fast<'a>(
        &mut self,
        cpu: &mut Cpu,
        table: &JitBlockTable,
        no_throttle: bool,
        peripheral_tick_batch: u32,
        report_window: u32,
    ) -> Result<(), JitError> {
        let mut fetch_count = 0u32;
        let mut window_start = Instant::now();
        let report_window_f64 = report_window as f64;
        let mut system_cycles = 0u64;
        let mut pending_peripheral_cycles = 0u32;

        cpu.refresh_peripheral_due_cycle(system_cycles, peripheral_tick_batch);
        cpu.take_and_clear_peripheral_schedule_dirty();

        let machine_cycle = u32::from(cpu.machine_cycle).max(1);

        loop {
            let loop_start = if no_throttle {
                None
            } else {
                Some(Instant::now())
            };

            let elapsed_cycles = self.step(cpu, table)?;
            system_cycles = system_cycles.saturating_add(elapsed_cycles as u64);
            pending_peripheral_cycles = pending_peripheral_cycles.saturating_add(elapsed_cycles);

            Self::maybe_drive_peripherals(
                cpu,
                &mut pending_peripheral_cycles,
                system_cycles,
                peripheral_tick_batch,
            );

            fetch_count += 1;
            if fetch_count >= report_window {
                let elapsed_secs = window_start.elapsed().as_secs_f64();
                if elapsed_secs > 0.0 {
                    let actual_freq_hz = report_window_f64 / elapsed_secs;
                    println!(
                        "Actual Execution Frequency ({} steps): {:.6} MHz",
                        report_window,
                        actual_freq_hz / 1_000_000.0
                    );
                }
                fetch_count = 0;
                window_start = Instant::now();
            }

            if let Some(loop_start) = loop_start {
                let frequency = cpu
                    .frequency
                    .load(std::sync::atomic::Ordering::Relaxed)
                    .max(1);
                let nanos_per_tick =
                    1_000_000_000u64 / (u64::from(frequency) * u64::from(machine_cycle));
                let tick_duration = Duration::from_nanos(nanos_per_tick.max(1));
                let elapsed = loop_start.elapsed();
                if elapsed < tick_duration {
                    thread::sleep(tick_duration - elapsed);
                }
            }
        }
    }

    fn maybe_drive_peripherals(
        cpu: &mut Cpu,
        pending_peripheral_cycles: &mut u32,
        system_cycles: u64,
        max_lag_cycles: u32,
    ) {
        if cpu.take_and_clear_peripheral_schedule_dirty() {
            cpu.refresh_peripheral_due_cycle(system_cycles, max_lag_cycles);
        }

        if *pending_peripheral_cycles == 0 {
            return;
        }

        if system_cycles < cpu.peripheral_due_cycle() {
            return;
        }

        let cycles = *pending_peripheral_cycles;
        *pending_peripheral_cycles = 0;
        cpu.peripheral_step_n(cycles);
        cpu.refresh_peripheral_due_cycle(system_cycles, max_lag_cycles);
    }

    fn execute_block<'a>(
        &mut self,
        cpu: &mut Cpu,
        table: &JitBlockTable,
        start_pc: u32,
        block: &JitBlockRange,
    ) -> Result<u32, JitError> {
        let compiled = self.get_or_compile_block_from_pc(table, block, start_pc)?;
        let cached = StepBlockCache {
            start_pc,
            end_pc: compiled.end_pc,
            entry: compiled.entry,
            instruction_count: compiled.instruction_count,
        };
        self.step_block_cache = Some(cached);
        self.executed_blocks = self.executed_blocks.saturating_add(1);
        self.executed_block_instructions = self
            .executed_block_instructions
            .saturating_add(cached.instruction_count as u64);
        Ok(unsafe { (cached.entry)(cpu as *mut Cpu) })
    }

    fn execute_block_from_builder(
        &mut self,
        cpu: &mut Cpu,
        builder: &JitBlockTableBuilder,
        start_pc: u32,
        block: &JitBlockRange,
    ) -> Result<u32, JitError> {
        let compiled = self.get_or_compile_block_from_builder(builder, block, start_pc)?;
        let cached = StepBlockCache {
            start_pc,
            end_pc: compiled.end_pc,
            entry: compiled.entry,
            instruction_count: compiled.instruction_count,
        };
        self.step_block_cache = Some(cached);
        self.executed_blocks = self.executed_blocks.saturating_add(1);
        self.executed_block_instructions = self
            .executed_block_instructions
            .saturating_add(cached.instruction_count as u64);
        Ok(unsafe { (cached.entry)(cpu as *mut Cpu) })
    }

    fn get_or_compile_block_from_pc<'a>(
        &mut self,
        table: &JitBlockTable,
        block: &JitBlockRange,
        start_pc: u32,
    ) -> Result<&CompiledBlock, JitError> {
        let is_cache_hit = self
            .blocks
            .get(&start_pc)
            .is_some_and(|compiled| compiled.end_pc == block.end_pc);

        if !is_cache_hit {
            self.cache_misses = self.cache_misses.saturating_add(1);
            let compiled = self.compile_block_from_pc(table, block, start_pc)?;
            self.blocks.insert(start_pc, compiled);
        } else {
            self.cache_hits = self.cache_hits.saturating_add(1);
        }

        Ok(self
            .blocks
            .get(&start_pc)
            .expect("compiled block missing after insert"))
    }

    fn get_or_compile_block_from_builder(
        &mut self,
        builder: &JitBlockTableBuilder,
        block: &JitBlockRange,
        start_pc: u32,
    ) -> Result<&CompiledBlock, JitError> {
        let is_cache_hit = self
            .blocks
            .get(&start_pc)
            .is_some_and(|compiled| compiled.end_pc == block.end_pc);

        if !is_cache_hit {
            self.cache_misses = self.cache_misses.saturating_add(1);
            let compiled = self.compile_block_from_builder(builder, block, start_pc)?;
            self.blocks.insert(start_pc, compiled);
        } else {
            self.cache_hits = self.cache_hits.saturating_add(1);
        }

        Ok(self
            .blocks
            .get(&start_pc)
            .expect("compiled block missing after insert"))
    }

    fn compile_block_from_pc<'a>(
        &mut self,
        table: &JitBlockTable,
        block: &JitBlockRange,
        start_pc: u32,
    ) -> Result<CompiledBlock, JitError> {
        let mut entries = Vec::with_capacity(block.instruction_count);
        let mut current_pc = start_pc;

        loop {
            let ins = table
                .get(current_pc)
                .ok_or(JitError::MissingInstruction { pc: current_pc })?;
            entries.push((current_pc, ins));
            if current_pc == block.end_pc {
                break;
            }
            current_pc = current_pc.wrapping_add(ins.data.size());
        }

        self.compiled_blocks = self.compiled_blocks.saturating_add(1);
        if start_pc != block.start_pc {
            self.compiled_suffix_blocks = self.compiled_suffix_blocks.saturating_add(1);
        }
        self.compiled_block_instructions = self
            .compiled_block_instructions
            .saturating_add(entries.len() as u64);

        self.compile_sequence(start_pc, entries)
    }

    fn compile_block_from_builder(
        &mut self,
        builder: &JitBlockTableBuilder,
        block: &JitBlockRange,
        start_pc: u32,
    ) -> Result<CompiledBlock, JitError> {
        let entries = builder
            .block_entries(start_pc, block)
            .ok_or(JitError::MissingInstruction { pc: start_pc })?;

        self.compiled_blocks = self.compiled_blocks.saturating_add(1);
        if start_pc != block.start_pc {
            self.compiled_suffix_blocks = self.compiled_suffix_blocks.saturating_add(1);
        }
        self.compiled_block_instructions = self
            .compiled_block_instructions
            .saturating_add(entries.len() as u64);

        self.compile_sequence(start_pc, entries)
    }

    fn compile_sequence<'a>(
        &mut self,
        pc: u32,
        entries: Vec<(u32, &JitInstruction)>,
    ) -> Result<CompiledBlock, JitError> {
        let end_pc = entries.last().map(|(current_pc, _)| *current_pc).unwrap_or(pc);
        let ptr_ty = self.module.target_config().pointer_type();
        let mut ctx = self.module.make_context();
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(ptr_ty));
        sig.returns.push(AbiParam::new(types::I32));
        ctx.func.signature = sig.clone();

        let func_name = format!("jit_block_{pc:08x}_{}", self.next_function_index);
        self.next_function_index = self.next_function_index.wrapping_add(1);
        let func_id = self
            .module
            .declare_function(&func_name, Linkage::Local, &sig)?;

        {
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut self.builder_ctx);
            let cache = BlockStateCache::new(&mut builder);
            let entry_block = builder.create_block();
            let exit_block = builder.create_block();
            builder.append_block_param(exit_block, types::I32);
            let instruction_blocks: Vec<_> = (0..entries.len())
                .map(|_| {
                    let block = builder.create_block();
                    builder.append_block_param(block, types::I32);
                    block
                })
                .collect();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            let cpu_ptr = builder.block_params(entry_block)[0];
            let zero = builder.ins().iconst(types::I32, 0);
            {
                let zero_ptr = builder.ins().iconst(ptr_ty, 0);
                let mut lowering = LoweringContext {
                    builder: &mut builder,
                    module: &mut self.module,
                    helpers: &self.helpers,
                    cache: &cache,
                    ptr_ty,
                    cpu_ptr,
                    instr_ptr: zero_ptr,
                    current_pc: zero,
                };
                lowering.initialize_cache_state();
            }

            if let Some(first_block) = instruction_blocks.first() {
                builder.ins().jump(*first_block, &[zero.into()]);
            } else {
                builder.ins().jump(exit_block, &[zero.into()]);
            }

            for (index, (current_pc, ins)) in entries.iter().enumerate() {
                let current_block = instruction_blocks[index];
                builder.switch_to_block(current_block);
                builder.seal_block(current_block);
                let carried_total = builder.block_params(current_block)[0];
                let current_pc_value = builder
                    .ins()
                    .iconst(types::I32, i64::from(*current_pc as i32));

                {
                    let zero_ptr = builder.ins().iconst(ptr_ty, 0);
                    let mut lowering = LoweringContext {
                        builder: &mut builder,
                        module: &mut self.module,
                        helpers: &self.helpers,
                        cache: &cache,
                        ptr_ty,
                        cpu_ptr,
                        instr_ptr: zero_ptr,
                        current_pc: current_pc_value,
                    };
                    lowering.instr_ptr = lowering.iconst_ptr(*ins as *const _ as *const ());
                    Self::lower_instruction(&mut lowering, ins);
                }

                let execute_cycles = builder
                    .ins()
                    .iconst(types::I32, i64::from(ins.execute_cycles as i32));
                let updated_total = builder.ins().iadd(carried_total, execute_cycles);

                if index + 1 == entries.len() {
                    builder.ins().jump(exit_block, &[updated_total.into()]);
                } else {
                    builder
                        .ins()
                        .jump(instruction_blocks[index + 1], &[updated_total.into()]);
                }
            }

            builder.switch_to_block(exit_block);
            builder.seal_block(exit_block);
            let total_cycles = builder.block_params(exit_block)[0];
            let zero_ptr = builder.ins().iconst(ptr_ty, 0);
            let mut lowering = LoweringContext {
                builder: &mut builder,
                module: &mut self.module,
                helpers: &self.helpers,
                cache: &cache,
                ptr_ty,
                cpu_ptr,
                instr_ptr: zero_ptr,
                current_pc: zero,
            };
            lowering.flush_dirty_state();
            let committed_cycles = lowering.call_value(
                lowering.helpers.finish_block_step_cycles,
                &[lowering.cpu_ptr, total_cycles],
            );
            lowering.builder.ins().return_(&[committed_cycles]);

            builder.finalize();
        }

        self.module.define_function(func_id, &mut ctx)?;
        self.module.clear_context(&mut ctx);
        self.module.finalize_definitions()?;

        let code = self.module.get_finalized_function(func_id);
        let entry = unsafe { std::mem::transmute::<*const u8, JitBlockFn>(code) };

        Ok(CompiledBlock {
            entry,
            end_pc,
            instruction_count: entries.len(),
        })
    }

    fn lower_instruction(lowering: &mut LoweringContext<'_, '_>, ins: &JitInstruction) {
        match ins.def.or_else(|| jit_instructions::find_def(ins.insn_id)) {
            Some(def) if def.supports(ins) => def.execute(lowering, ins),
            _ => {
                let pc_update = lowering.emit_fallback();
                lowering.apply_pc_update(pc_update);
            }
        }
    }
}

impl RuntimeFunctions {
    fn declare(module: &mut JITModule) -> Result<Self, JitError> {
        let ptr_ty = module.target_config().pointer_type();

        let finish_block_step_cycles = declare_import(module, "jit_finish_block_step_cycles", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(types::I32));
            sig.returns.push(AbiParam::new(types::I32));
        })?;

        let read_reg = declare_import(module, "jit_read_reg", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(types::I32));
            sig.returns.push(AbiParam::new(types::I32));
        })?;

        let write_reg = declare_import(module, "jit_write_reg", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(types::I32));
            sig.params.push(AbiParam::new(types::I32));
        })?;

        let read_apsr = declare_import(module, "jit_read_apsr", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.returns.push(AbiParam::new(types::I32));
        })?;

        let write_apsr = declare_import(module, "jit_write_apsr", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(types::I32));
        })?;

        let read_u8 = declare_import(module, "jit_read_u8", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(types::I32));
            sig.returns.push(AbiParam::new(types::I32));
        })?;

        let read_u16 = declare_import(module, "jit_read_u16", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(types::I32));
            sig.returns.push(AbiParam::new(types::I32));
        })?;

        let read_u32 = declare_import(module, "jit_read_u32", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(types::I32));
            sig.returns.push(AbiParam::new(types::I32));
        })?;

        let write_u8 = declare_import(module, "jit_write_u8", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(types::I32));
            sig.params.push(AbiParam::new(types::I32));
        })?;

        let write_u16 = declare_import(module, "jit_write_u16", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(types::I32));
            sig.params.push(AbiParam::new(types::I32));
        })?;

        let write_u32 = declare_import(module, "jit_write_u32", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(types::I32));
            sig.params.push(AbiParam::new(types::I32));
        })?;

        let try_exception_return = declare_import(module, "jit_try_exception_return", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(types::I32));
            sig.returns.push(AbiParam::new(types::I32));
        })?;

        let resolve_mem_rt_addr = declare_import(module, "jit_resolve_mem_rt_addr", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(ptr_ty));
            sig.returns.push(AbiParam::new(types::I64));
        })?;

        let compute_shift_packed = declare_import(module, "jit_compute_shift_packed", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(ptr_ty));
            sig.returns.push(AbiParam::new(types::I64));
        })?;

        let execute_bkpt = declare_import(module, "jit_execute_bkpt", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(ptr_ty));
        })?;

        let udiv_or_zero = declare_import(module, "jit_udiv_or_zero", |sig| {
            sig.params.push(AbiParam::new(types::I32));
            sig.params.push(AbiParam::new(types::I32));
            sig.returns.push(AbiParam::new(types::I32));
        })?;

        let fallback_exec = declare_import(module, "jit_execute_fallback", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(ptr_ty));
            sig.returns.push(AbiParam::new(types::I32));
        })?;

        Ok(Self {
            finish_block_step_cycles,
            read_reg,
            write_reg,
            read_apsr,
            write_apsr,
            read_u8,
            read_u16,
            read_u32,
            write_u8,
            write_u16,
            write_u32,
            try_exception_return,
            resolve_mem_rt_addr,
            compute_shift_packed,
            execute_bkpt,
            udiv_or_zero,
            fallback_exec,
        })
    }
}

fn declare_import<F>(module: &mut JITModule, name: &str, build_sig: F) -> Result<FuncId, JitError>
where
    F: FnOnce(&mut cranelift::codegen::ir::Signature),
{
    let mut sig = module.make_signature();
    build_sig(&mut sig);
    Ok(module.declare_function(name, Linkage::Import, &sig)?)
}

extern "C" fn jit_finish_block_step_cycles(
    cpu: *mut Cpu,
    execute_cycles: u32,
) -> u32 {
    JIT_RUNTIME_COUNTERS
        .finish_block_step_cycles_calls
        .fetch_add(1, Ordering::Relaxed);
    let cpu = unsafe { &mut *cpu };
    cpu.finish_block_step_cycles(execute_cycles)
}

extern "C" fn jit_read_reg(cpu: *mut Cpu, reg: u32) -> u32 {
    JIT_RUNTIME_COUNTERS
        .read_reg_calls
        .fetch_add(1, Ordering::Relaxed);
    let cpu = unsafe { &*cpu };
    cpu.read_reg(reg)
}

extern "C" fn jit_write_reg(cpu: *mut Cpu, reg: u32, value: u32) {
    JIT_RUNTIME_COUNTERS
        .write_reg_calls
        .fetch_add(1, Ordering::Relaxed);
    let cpu = unsafe { &mut *cpu };
    cpu.write_reg(reg, value);
}

extern "C" fn jit_read_apsr(cpu: *mut Cpu) -> u32 {
    JIT_RUNTIME_COUNTERS
        .read_apsr_calls
        .fetch_add(1, Ordering::Relaxed);
    let cpu = unsafe { &*cpu };
    cpu.read_apsr()
}

extern "C" fn jit_write_apsr(cpu: *mut Cpu, value: u32) {
    JIT_RUNTIME_COUNTERS
        .flag_update_calls
        .fetch_add(1, Ordering::Relaxed);
    let cpu = unsafe { &mut *cpu };
    cpu.write_apsr(value);
}

extern "C" fn jit_read_u32(cpu: *mut Cpu, addr: u32) -> u32 {
    JIT_RUNTIME_COUNTERS
        .mem_read_calls
        .fetch_add(1, Ordering::Relaxed);
    let cpu = unsafe { &*cpu };
    cpu.read_mem(addr)
}

extern "C" fn jit_read_u8(cpu: *mut Cpu, addr: u32) -> u32 {
    JIT_RUNTIME_COUNTERS
        .mem_read_calls
        .fetch_add(1, Ordering::Relaxed);
    let cpu = unsafe { &*cpu };
    let word = cpu.read_mem(addr & !3);
    let shift = (addr & 3) * 8;
    (word >> shift) & 0xFF
}

extern "C" fn jit_read_u16(cpu: *mut Cpu, addr: u32) -> u32 {
    JIT_RUNTIME_COUNTERS
        .mem_read_calls
        .fetch_add(1, Ordering::Relaxed);
    let cpu = unsafe { &*cpu };
    let word = cpu.read_mem(addr & !3);
    let shift = (addr & 2) * 8;
    (word >> shift) & 0xFFFF
}

extern "C" fn jit_write_u32(cpu: *mut Cpu, addr: u32, value: u32) {
    JIT_RUNTIME_COUNTERS
        .mem_write_calls
        .fetch_add(1, Ordering::Relaxed);
    let cpu = unsafe { &mut *cpu };
    cpu.write_mem(addr, value);
}

extern "C" fn jit_write_u8(cpu: *mut Cpu, addr: u32, value: u32) {
    JIT_RUNTIME_COUNTERS
        .mem_write_calls
        .fetch_add(1, Ordering::Relaxed);
    let cpu = unsafe { &mut *cpu };
    let aligned_addr = addr & !3;
    let shift = (addr & 3) * 8;
    let mask = !(0xFF << shift);
    let current = cpu.read_mem(aligned_addr);
    let new_value = (current & mask) | ((value & 0xFF) << shift);
    cpu.write_mem(aligned_addr, new_value);
}

extern "C" fn jit_write_u16(cpu: *mut Cpu, addr: u32, value: u32) {
    JIT_RUNTIME_COUNTERS
        .mem_write_calls
        .fetch_add(1, Ordering::Relaxed);
    let cpu = unsafe { &mut *cpu };
    let aligned_addr = addr & !3;
    let shift = (addr & 2) * 8;
    let mask = !(0xFFFF << shift);
    let current = cpu.read_mem(aligned_addr);
    let new_value = (current & mask) | ((value & 0xFFFF) << shift);
    cpu.write_mem(aligned_addr, new_value);
}

extern "C" fn jit_try_exception_return(cpu: *mut Cpu, value: u32) -> u32 {
    JIT_RUNTIME_COUNTERS
        .exception_return_calls
        .fetch_add(1, Ordering::Relaxed);
    let cpu = unsafe { &mut *cpu };
    u32::from(cpu.try_exception_return(value))
}

extern "C" fn jit_resolve_mem_rt_addr(cpu: *mut Cpu, instr: *const ()) -> u64 {
    JIT_RUNTIME_COUNTERS
        .resolve_mem_rt_addr_calls
        .fetch_add(1, Ordering::Relaxed);
    let cpu = unsafe { &mut *cpu };
    let instr = unsafe { &*(instr as *const JitInstruction) };
    let (rt, addr) = operand_resolver_multi_runtime(cpu, &instr.data);
    (u64::from(addr) << 32) | u64::from(rt)
}

extern "C" fn jit_compute_shift_packed(cpu: *mut Cpu, instr: *const ()) -> u64 {
    JIT_RUNTIME_COUNTERS
        .compute_shift_calls
        .fetch_add(1, Ordering::Relaxed);
    let cpu = unsafe { &mut *cpu };
    let instr = unsafe { &*(instr as *const JitInstruction) };
    let rm = instr.data.arm_operands.rn;
    let rm_val = runtime_read_reg(cpu, &instr.data, rm);
    let current_c = ((cpu.read_apsr() >> 29) & 1) as u8;
    let shift_amount = match &instr.data.arm_operands.op2 {
        Some(op) => match op.op_type {
            DecodedOperandKind::Imm(imm) => (imm as u32) & 0xFF,
            DecodedOperandKind::Reg(reg) => runtime_read_reg(cpu, &instr.data, reg) & 0xFF,
            _ => 0,
        },
        None => 0,
    };

    let (result, carry) = match instr.insn_id {
        x if x == ArmInsn::ARM_INS_ASR as u32 => {
            if shift_amount == 0 {
                (rm_val, current_c)
            } else if shift_amount >= 32 {
                let result = if (rm_val & 0x8000_0000) != 0 {
                    u32::MAX
                } else {
                    0
                };
                (result, ((rm_val >> 31) & 1) as u8)
            } else {
                (
                    ((rm_val as i32) >> shift_amount) as u32,
                    ((rm_val >> (shift_amount - 1)) & 1) as u8,
                )
            }
        }
        x if x == ArmInsn::ARM_INS_LSL as u32 => {
            if shift_amount == 0 {
                (rm_val, current_c)
            } else if shift_amount > 32 {
                (0, 0)
            } else if shift_amount == 32 {
                (0, (rm_val & 1) as u8)
            } else {
                (
                    rm_val.wrapping_shl(shift_amount),
                    ((rm_val >> (32 - shift_amount)) & 1) as u8,
                )
            }
        }
        x if x == ArmInsn::ARM_INS_LSR as u32 => {
            if shift_amount == 0 {
                (rm_val, current_c)
            } else if shift_amount > 32 {
                (0, 0)
            } else if shift_amount == 32 {
                (0, ((rm_val >> 31) & 1) as u8)
            } else {
                (rm_val >> shift_amount, ((rm_val >> (shift_amount - 1)) & 1) as u8)
            }
        }
        x if x == ArmInsn::ARM_INS_ROR as u32 => {
            if shift_amount == 0 {
                (rm_val, current_c)
            } else {
                let shift = shift_amount & 31;
                if shift == 0 {
                    (rm_val, ((rm_val >> 31) & 1) as u8)
                } else {
                    let result = rm_val.rotate_right(shift);
                    (result, ((result >> 31) & 1) as u8)
                }
            }
        }
        x if x == ArmInsn::ARM_INS_RRX as u32 => {
            let result = ((current_c as u32) << 31) | (rm_val >> 1);
            (result, (rm_val & 1) as u8)
        }
        _ => (rm_val, current_c),
    };

    ((carry as u64) << 32) | u64::from(result)
}

extern "C" fn jit_execute_bkpt(cpu: *mut Cpu, instr: *const ()) {
    JIT_RUNTIME_COUNTERS
        .bkpt_calls
        .fetch_add(1, Ordering::Relaxed);
    let cpu = unsafe { &mut *cpu };
    let instr = unsafe { &*(instr as *const JitInstruction) };

    let imm = match &instr.data.arm_operands.op2 {
        Some(op) => match op.op_type {
            DecodedOperandKind::Imm(imm) => imm as u32,
            DecodedOperandKind::Reg(reg) => runtime_read_reg(cpu, &instr.data, reg),
            _ => 0,
        },
        None => 0,
    };

    println!("BKPT #{}", imm);
}

extern "C" fn jit_udiv_or_zero(lhs: u32, rhs: u32) -> u32 {
    JIT_RUNTIME_COUNTERS
        .udiv_calls
        .fetch_add(1, Ordering::Relaxed);
    if rhs == 0 { 0 } else { lhs / rhs }
}

extern "C" fn jit_execute_fallback(cpu: *mut Cpu, instr: *const ()) -> u32 {
    JIT_RUNTIME_COUNTERS
        .fallback_calls
        .fetch_add(1, Ordering::Relaxed);
    let cpu = unsafe { &mut *cpu };
    let instr = unsafe { &*(instr as *const JitInstruction) };
    thumb_runtime::execute_current(cpu, instr.data.address())
        .unwrap_or_else(|err| panic!("jit fallback execute failed at 0x{:08X}: {err}", instr.data.address()))
        .pc_update
}

#[cfg(test)]
mod tests {
    use super::*;
    use capstone::arch;
    use capstone::prelude::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicU32;

    use crate::cpu::Cpu;
    use crate::jit_engine::table::JitBlockTableBuilder;
    use crate::peripheral::bus::Bus;
    use crate::peripheral::nvic::Nvic;
    use crate::peripheral::rcc::Rcc;
    use crate::peripheral::scb::Scb;
    use crate::peripheral::systick::SysTick;

    fn build_thumb_capstone() -> Capstone {
        Capstone::new()
            .arm()
            .mode(arch::arm::ArchMode::Thumb)
            .extra_mode([arch::arm::ArchExtraMode::MClass].iter().copied())
            .detail(true)
            .build()
            .expect("failed to create capstone")
    }

    fn build_cpu() -> Cpu {
        let mut ppb = Bus::new();
        ppb.register_peripheral(Box::new(SysTick::new(0xE000_E010, 0xE000_E01F)));
        ppb.register_peripheral(Box::new(Nvic::new(0xE000_E100, 0xE000_E4EF)));
        ppb.register_peripheral(Box::new(Scb::new(0xE000_ED00, 0xE000_ED3C)));

        Cpu::new(Arc::new(AtomicU32::new(8_000_000)), 1, Bus::new(), ppb)
    }

    fn build_cpu_with_rcc() -> Cpu {
        let shared_freq = Arc::new(AtomicU32::new(8_000_000));
        let mut bus = Bus::new();
        bus.register_peripheral(Box::new(Rcc::new(0x4002_0000, 0x4002_1024, shared_freq.clone())));

        let mut ppb = Bus::new();
        ppb.register_peripheral(Box::new(SysTick::new(0xE000_E010, 0xE000_E01F)));
        ppb.register_peripheral(Box::new(Nvic::new(0xE000_E100, 0xE000_E4EF)));
        ppb.register_peripheral(Box::new(Scb::new(0xE000_ED00, 0xE000_ED3C)));

        Cpu::new(shared_freq, 1, bus, ppb)
    }


    #[test]
    fn jit_falls_back_for_unsupported_instruction() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
        .expect("failed to build jit table");

        let mut cpu = build_cpu();
        cpu.next_pc = 0x0800_0000;

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let cycles = engine.step(&mut cpu, &table).expect("jit step failed");

        assert_eq!(cycles, 1);
        assert_eq!(cpu.next_pc, 0x0800_0002);
        assert_eq!(engine.compiled_block_count(), 1);
        assert!(engine.blocks.get(&0x0800_0000).is_some());
    }

    #[test]
    fn jit_step_executes_full_fallthrough_block() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x00, 0xBF, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
        .expect("failed to build jit table");

        let mut cpu = build_cpu();
        cpu.next_pc = 0x0800_0000;

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let cycles = engine.step(&mut cpu, &table).expect("jit block step failed");

        assert_eq!(cycles, 2);
        assert_eq!(cpu.next_pc, 0x0800_0004);
        assert_eq!(engine.compiled_block_count(), 1);
        assert!(engine.blocks.get(&0x0800_0000).is_some());
        assert!(engine.blocks.get(&0x0800_0002).is_none());
    }

    #[test]
    fn jit_step_reads_pc_from_ir_without_prefetch_helper() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x78, 0x46, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
        .expect("failed to build jit table");

        let mut cpu = build_cpu();
        cpu.next_pc = 0x0800_0000;

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let cycles = engine.step(&mut cpu, &table).expect("jit block step failed");

        assert_eq!(cycles, 2);
        assert_eq!(cpu.read_reg(0), 0x0800_0004);
        assert_eq!(cpu.read_reg(15), 0x0800_0004);
        assert_eq!(cpu.next_pc, 0x0800_0004);
    }

    #[test]
    fn jit_step_reuses_cached_reg_write_within_block() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x01, 0x20, 0x40, 0x1C, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
        .expect("failed to build jit table");

        let mut cpu = build_cpu();
        cpu.next_pc = 0x0800_0000;

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let before = engine.stats_snapshot();
        let cycles = engine.step(&mut cpu, &table).expect("jit block step failed");
        let delta = engine.stats_snapshot().delta_since(before);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.read_reg(0), 2);
        assert_eq!(cpu.next_pc, 0x0800_0006);
        assert_eq!(delta.read_reg_calls, 0);
        assert_eq!(delta.write_reg_calls, 0);
    }

    #[test]
    fn jit_step_uses_cached_apsr_for_conditional_branch() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x00, 0x28, 0x00, 0xD0, 0x00, 0xBF, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
        .expect("failed to build jit table");

        let mut cpu = build_cpu();
        cpu.next_pc = 0x0800_0000;
        cpu.write_reg(0, 0);

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let cycles = engine.step(&mut cpu, &table).expect("jit block step failed");

        assert_eq!(cycles, 3);
        assert_eq!(cpu.next_pc, 0x0800_0006);
    }

    #[test]
    fn jit_step_mirrors_fallthrough_pc_at_block_exit() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x00, 0xBF, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
        .expect("failed to build jit table");

        let mut cpu = build_cpu();
        cpu.next_pc = 0x0800_0000;

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let cycles = engine.step(&mut cpu, &table).expect("jit block step failed");

        assert_eq!(cycles, 2);
        assert_eq!(cpu.read_reg(15), 0x0800_0004);
        assert_eq!(cpu.next_pc, 0x0800_0004);
    }

    #[test]
    fn jit_resolve_op2_inlines_immediate_operand() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x01, 0x20, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
        .expect("failed to build jit table");

        let mut cpu = build_cpu();
        cpu.next_pc = 0x0800_0000;

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let before = engine.stats_snapshot();
        let cycles = engine.step(&mut cpu, &table).expect("jit move-immediate step failed");
        let delta = engine.stats_snapshot().delta_since(before);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.read_reg(0), 1);
        assert_eq!(delta.resolve_op2_calls, 0);
    }

    #[test]
    fn jit_resolve_op2_inlines_register_operand_without_shift() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x08, 0x40, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
        .expect("failed to build jit table");

        let mut cpu = build_cpu();
        cpu.next_pc = 0x0800_0000;
        cpu.write_reg(0, 0xF0F0_F0F0);
        cpu.write_reg(1, 0x00FF_00FF);

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let before = engine.stats_snapshot();
        let cycles = engine.step(&mut cpu, &table).expect("jit register-op2 step failed");
        let delta = engine.stats_snapshot().delta_since(before);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.read_reg(0), 0x00F0_00F0);
        assert_eq!(delta.resolve_op2_calls, 0);
    }

    #[test]
    fn jit_branch_immediate_uses_ir_constant_target() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x00, 0xE0, 0x00, 0xBF, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
        .expect("failed to build jit table");

        let mut cpu = build_cpu();
        cpu.next_pc = 0x0800_0000;

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let before = engine.stats_snapshot();
        let cycles = engine.step(&mut cpu, &table).expect("jit branch step failed");
        let delta = engine.stats_snapshot().delta_since(before);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.next_pc, 0x0800_0004);
        assert_eq!(delta.resolve_op2_calls, 0);
        assert_eq!(delta.read_reg_calls, 0);
    }

    #[test]
    fn jit_step_accumulates_block_cycles_without_mid_block_drains() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x08, 0x68, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
        .expect("failed to build jit table");

        let mut cpu = build_cpu();
        cpu.next_pc = 0x0800_0000;
        cpu.write_reg(1, 0x2000_0000);
        cpu.write_mem(0x2000_0000, 0x1122_3344);

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let cycles = engine
            .step(&mut cpu, &table)
            .expect("jit block step failed");

        assert_eq!(cycles, 3);
        assert_eq!(cpu.next_pc, 0x0800_0004);
        assert_eq!(cpu.read_reg(0), 0x1122_3344);
        assert_eq!(cpu.begin_step(), None);
    }

    #[test]
    fn jit_word_store_to_ram_uses_lowered_fast_path() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x08, 0x60, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
            .expect("failed to build jit table");

        let mut cpu = build_cpu();
        cpu.next_pc = 0x0800_0000;
        cpu.write_reg(0, 0xAABB_CCDD);
        cpu.write_reg(1, 0x2000_0000);

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let before = engine.stats_snapshot();
        let cycles = engine.step(&mut cpu, &table).expect("jit store step failed");
        let delta = engine.stats_snapshot().delta_since(before);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.read_mem(0x2000_0000), 0xAABB_CCDD);
        assert_eq!(cpu.next_pc, 0x0800_0004);
        assert_eq!(delta.mem_write_calls, 0);
        assert_eq!(delta.resolve_mem_rt_addr_calls, 0);
    }

    #[test]
    fn jit_word_load_from_ram_uses_lowered_fast_path() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x08, 0x68, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
            .expect("failed to build jit table");

        let mut cpu = build_cpu();
        cpu.next_pc = 0x0800_0000;
        cpu.write_reg(1, 0x2000_0000);
        cpu.write_mem(0x2000_0000, 0x1122_3344);

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let before = engine.stats_snapshot();
        let cycles = engine.step(&mut cpu, &table).expect("jit load step failed");
        let delta = engine.stats_snapshot().delta_since(before);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.next_pc, 0x0800_0004);
        assert_eq!(cpu.read_reg(0), 0x1122_3344);
        assert_eq!(delta.mem_read_calls, 0);
        assert_eq!(delta.resolve_mem_rt_addr_calls, 0);
    }

    #[test]
    fn jit_word_store_to_ppb_keeps_helper_fallback() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x08, 0x60, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
            .expect("failed to build jit table");

        let mut cpu = build_cpu();
        cpu.next_pc = 0x0800_0000;
        cpu.write_reg(0, 1);
        cpu.write_reg(1, 0xE000_E100);

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let before = engine.stats_snapshot();
        let cycles = engine.step(&mut cpu, &table).expect("jit store step failed");
        let delta = engine.stats_snapshot().delta_since(before);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.next_pc, 0x0800_0004);
        assert_eq!(delta.mem_write_calls, 1);
        assert_eq!(delta.resolve_mem_rt_addr_calls, 0);
    }

    #[test]
    fn jit_shift_register_uses_ir_lowering() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x0C, 0xFA, 0x00, 0xF3, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
            .expect("failed to build jit table");

        let mut cpu = build_cpu();
        cpu.next_pc = 0x0800_0000;
        cpu.write_reg(12, 3);
        cpu.write_reg(0, 4);

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let before = engine.stats_snapshot();
        let cycles = engine.step(&mut cpu, &table).expect("jit shift step failed");
        let delta = engine.stats_snapshot().delta_since(before);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.read_reg(3), 48);
        assert_eq!(delta.compute_shift_calls, 0);
    }

    #[test]
    fn jit_udiv_uses_ir_zero_guard() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0xB5, 0xFB, 0xF3, 0xF5, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
            .expect("failed to build jit table");

        let mut cpu = build_cpu();
        cpu.next_pc = 0x0800_0000;
        cpu.write_reg(5, 100);
        cpu.write_reg(3, 4);

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let before = engine.stats_snapshot();
        let cycles = engine.step(&mut cpu, &table).expect("jit udiv step failed");
        let delta = engine.stats_snapshot().delta_since(before);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.read_reg(5), 25);
        assert_eq!(delta.udiv_calls, 0);
    }

    #[test]
    fn jit_startup_poll_loop_exits_after_timeout() {
        let code = [
            0x30, 0x48, // ldr r0, [pc, #0xc0]
            0x00, 0x68, // ldr r0, [r0]
            0x00, 0xF4, 0x00, 0x30, // and r0, r0, #0x20000
            0x00, 0x90, // str r0, [sp]
            0x01, 0x98, // ldr r0, [sp, #4]
            0x40, 0x1C, // adds r0, r0, #1
            0x01, 0x90, // str r0, [sp, #4]
            0x00, 0x98, // ldr r0, [sp]
            0x18, 0xB9, // cbnz r0, #0x0800045e
            0x01, 0x98, // ldr r0, [sp, #4]
            0xB0, 0xF5, 0xA0, 0x6F, // cmp.w r0, #0x500
            0xF1, 0xD1, // bne #0x08000442
        ];

        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&code, 0x0800_0442)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
            .expect("failed to build jit table");

        let mut cpu = build_cpu();
        cpu.load_code_bytes(0x0800_0504, &0x2000_0020u32.to_le_bytes());
        cpu.write_mem(0x2000_0020, 0);
        cpu.write_sp(0x2000_0100);
        cpu.write_mem(0x2000_0100, 0);
        cpu.write_mem(0x2000_0104, 0);
        cpu.next_pc = 0x0800_0442;

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        for _ in 0..4000 {
            if cpu.next_pc == 0x0800_045E {
                break;
            }
            engine
                .step(&mut cpu, &table)
                .expect("jit poll loop step failed");
        }

        assert_eq!(cpu.next_pc, 0x0800_045E);
        assert_eq!(cpu.read_mem(0x2000_0104), 0x500);
    }

    #[test]
    fn jit_rcc_enable_write_is_visible_to_following_poll() {
        let code = [
            0x33, 0x48, // ldr r0, [pc, #0xcc]
            0x00, 0x68, // ldr r0, [r0]
            0x40, 0xF4, 0x80, 0x30, // orr r0, r0, #0x10000
            0x31, 0x49, // ldr r1, [pc, #0xc4]
            0x08, 0x60, // str r0, [r1]
            0x00, 0xBF, // nop
            0x30, 0x48, // ldr r0, [pc, #0xc0]
            0x00, 0x68, // ldr r0, [r0]
            0x00, 0xF4, 0x00, 0x30, // and r0, r0, #0x20000
            0x00, 0x90, // str r0, [sp]
            0x01, 0x98, // ldr r0, [sp, #4]
            0x40, 0x1C, // adds r0, r0, #1
            0x01, 0x90, // str r0, [sp, #4]
            0x00, 0x98, // ldr r0, [sp]
            0x18, 0xB9, // cbnz r0, #0x0800045e
            0x01, 0x98, // ldr r0, [sp, #4]
            0xB0, 0xF5, 0xA0, 0x6F, // cmp.w r0, #0x500
            0xF1, 0xD1, // bne #0x08000442
        ];

        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&code, 0x0800_0434)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
            .expect("failed to build jit table");

        let mut cpu = build_cpu_with_rcc();
        cpu.load_code_bytes(0x0800_0504, &0x4002_1000u32.to_le_bytes());
        cpu.write_sp(0x2000_0100);
        cpu.write_mem(0x2000_0100, 0);
        cpu.write_mem(0x2000_0104, 0);
        cpu.next_pc = 0x0800_0434;
        cpu.write_mem(0x4002_1000, 0x0000_0083);

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        for _ in 0..32 {
            if cpu.next_pc == 0x0800_045E {
                break;
            }
            engine
                .step(&mut cpu, &table)
                .expect("jit rcc poll step failed");
        }

        assert_eq!(cpu.next_pc, 0x0800_045E);
        assert_ne!(cpu.read_mem(0x4002_1000) & 0x0002_0000, 0);
        assert_eq!(cpu.read_mem(0x2000_0104), 1);
    }

    #[test]
    fn jit_push_frame_preserves_startup_poll_locals() {
        let code = [
            0x0C, 0xB5, // push {r2, r3, lr}
            0x00, 0x20, // movs r0, #0
            0x01, 0x90, // str r0, [sp, #4]
            0x00, 0x90, // str r0, [sp]
            0x33, 0x48, // ldr r0, [pc, #0xcc]
            0x00, 0x68, // ldr r0, [r0]
            0x40, 0xF4, 0x80, 0x30, // orr r0, r0, #0x10000
            0x31, 0x49, // ldr r1, [pc, #0xc4]
            0x08, 0x60, // str r0, [r1]
            0x00, 0xBF, // nop
            0x30, 0x48, // ldr r0, [pc, #0xc0]
            0x00, 0x68, // ldr r0, [r0]
            0x00, 0xF4, 0x00, 0x30, // and r0, r0, #0x20000
            0x00, 0x90, // str r0, [sp]
            0x01, 0x98, // ldr r0, [sp, #4]
            0x40, 0x1C, // adds r0, r0, #1
            0x01, 0x90, // str r0, [sp, #4]
            0x00, 0x98, // ldr r0, [sp]
            0x18, 0xB9, // cbnz r0, #0x0800045e
            0x01, 0x98, // ldr r0, [sp, #4]
            0xB0, 0xF5, 0xA0, 0x6F, // cmp.w r0, #0x500
            0xF1, 0xD1, // bne #0x08000442
        ];

        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&code, 0x0800_042C)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
            .expect("failed to build jit table");

        let mut cpu = build_cpu_with_rcc();
        cpu.load_code_bytes(0x0800_0504, &0x4002_1000u32.to_le_bytes());
        cpu.write_sp(0x2000_0110);
        cpu.write_reg(2, 0x2222_2222);
        cpu.write_reg(3, 0x3333_3333);
        cpu.write_reg(14, 0xEEEE_EEEE);
        cpu.next_pc = 0x0800_042C;
        cpu.write_mem(0x4002_1000, 0x0000_0083);

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        for _ in 0..32 {
            if cpu.next_pc == 0x0800_045E {
                break;
            }
            engine
                .step(&mut cpu, &table)
                .expect("jit push-frame poll step failed");
        }

        assert_eq!(cpu.next_pc, 0x0800_045E);
        assert_eq!(cpu.read_sp(), 0x2000_0104);
        assert_eq!(cpu.read_mem(0x2000_0104), 0x0002_0000);
        assert_eq!(cpu.read_mem(0x2000_0108), 1);
        assert_eq!(cpu.read_mem(0x2000_010C), 0xEEEE_EEEE);
    }

    #[test]
    fn jit_runtime_builder_rcc_poll_matches_predecoded_path() {
        let code = [
            0x0C, 0xB5, // push {r2, r3, lr}
            0x00, 0x20, // movs r0, #0
            0x01, 0x90, // str r0, [sp, #4]
            0x00, 0x90, // str r0, [sp]
            0x33, 0x48, // ldr r0, [pc, #0xcc]
            0x00, 0x68, // ldr r0, [r0]
            0x40, 0xF4, 0x80, 0x30, // orr r0, r0, #0x10000
            0x31, 0x49, // ldr r1, [pc, #0xc4]
            0x08, 0x60, // str r0, [r1]
            0x00, 0xBF, // nop
            0x30, 0x48, // ldr r0, [pc, #0xc0]
            0x00, 0x68, // ldr r0, [r0]
            0x00, 0xF4, 0x00, 0x30, // and r0, r0, #0x20000
            0x00, 0x90, // str r0, [sp]
            0x01, 0x98, // ldr r0, [sp, #4]
            0x40, 0x1C, // adds r0, r0, #1
            0x01, 0x90, // str r0, [sp, #4]
            0x00, 0x98, // ldr r0, [sp]
            0x18, 0xB9, // cbnz r0, #0x0800045e
            0x01, 0x98, // ldr r0, [sp, #4]
            0xB0, 0xF5, 0xA0, 0x6F, // cmp.w r0, #0x500
            0xF1, 0xD1, // bne #0x08000442
        ];

        let mut cpu = build_cpu_with_rcc();
        cpu.load_code_bytes(0x0800_042C, &code);
        cpu.load_code_bytes(0x0800_0504, &0x4002_1000u32.to_le_bytes());
        cpu.write_sp(0x2000_0110);
        cpu.write_reg(2, 0x2222_2222);
        cpu.write_reg(3, 0x3333_3333);
        cpu.write_reg(14, 0xEEEE_EEEE);
        cpu.next_pc = 0x0800_042C;
        cpu.write_mem(0x4002_1000, 0x0000_0083);

        let mut builder = JitBlockTableBuilder::new();
        let added = builder
            .extend_from_pc(&cpu, 0x0800_042C, 0x0800_0600, 64)
            .expect("failed to extend jit table");
        assert!(added > 0);

        let orr = builder.get(0x0800_0438).expect("missing runtime-decoded orr");
        let and = builder.get(0x0800_0446).expect("missing runtime-decoded and");
        let ldr_counter = builder.get(0x0800_044C).expect("missing runtime-decoded counter load");
        let add_counter = builder.get(0x0800_044E).expect("missing runtime-decoded counter increment");
        let str_counter = builder.get(0x0800_0450).expect("missing runtime-decoded counter store");
        let ldr_flag = builder.get(0x0800_0452).expect("missing runtime-decoded flag reload");
        let cbnz = builder.get(0x0800_0454).expect("missing runtime-decoded cbnz");

        assert_eq!(orr.insn_id, crate::arch::ArmInsn::ARM_INS_ORR as u32);
        assert_eq!(and.insn_id, crate::arch::ArmInsn::ARM_INS_AND as u32);
        assert_eq!(ldr_counter.insn_id, crate::arch::ArmInsn::ARM_INS_LDR as u32);
        assert_eq!(add_counter.insn_id, crate::arch::ArmInsn::ARM_INS_ADD as u32);
        assert_eq!(str_counter.insn_id, crate::arch::ArmInsn::ARM_INS_STR as u32);
        assert_eq!(ldr_flag.insn_id, crate::arch::ArmInsn::ARM_INS_LDR as u32);
        assert_eq!(cbnz.insn_id, crate::arch::ArmInsn::ARM_INS_CBNZ as u32);

        match orr.data.arm_operands.op2.as_ref().map(|op| &op.op_type) {
            Some(crate::opcodes::decoded::DecodedOperandKind::Imm(imm)) => assert_eq!(*imm, 0x1_0000),
            _ => panic!("runtime-decoded orr immediate mismatch"),
        }
        match and.data.arm_operands.op2.as_ref().map(|op| &op.op_type) {
            Some(crate::opcodes::decoded::DecodedOperandKind::Imm(imm)) => assert_eq!(*imm, 0x2_0000),
            _ => panic!("runtime-decoded and immediate mismatch"),
        }

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        for _ in 0..32 {
            if cpu.next_pc == 0x0800_045E {
                break;
            }
            let current_pc = cpu.next_pc;
            if builder.block_containing(current_pc).is_none() {
                let added = builder
                    .extend_from_pc(&cpu, current_pc, 0x0800_0600, 64)
                    .expect("failed to extend runtime-builder block");
                assert!(added > 0, "runtime builder added no instructions at 0x{current_pc:08X}");
            }
            let block = builder
                .block_containing(current_pc)
                .expect("missing block during runtime-builder execution");
            engine
                .step_block_builder(&mut cpu, &builder, current_pc, &block)
                .expect("runtime-builder jit step failed");
        }

        assert_eq!(cpu.next_pc, 0x0800_045E);
        assert_eq!(cpu.read_sp(), 0x2000_0104);
        assert_eq!(cpu.read_mem(0x2000_0104), 0x0002_0000);
        assert_eq!(cpu.read_mem(0x2000_0108), 1);
    }

    #[test]
    fn jit_compile_table_precompiles_all_entries() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x08, 0x68, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
        .expect("failed to build jit table");

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let compiled = engine
            .compile_table(&table)
            .expect("failed to compile table");

        assert_eq!(compiled.len(), 1);
        assert!(engine.compiled_entry(0x0800_0000).is_some());
        assert!(engine.compiled_entry(0x0800_0002).is_none());
        assert!(engine.blocks.get(&0x0800_0000).is_some());
        assert!(engine.blocks.get(&0x0800_0002).is_none());
    }

    #[test]
    fn jit_step_block_recompiles_when_block_end_changes() {
        let mut cpu = build_cpu();
        cpu.load_code_bytes(0x0800_0000, &[0x00, 0xBF, 0x00, 0xBF]);
        cpu.next_pc = 0x0800_0000;

        let mut builder = JitBlockTableBuilder::new();
        builder
            .extend_from_pc(&cpu, 0x0800_0000, 0x0800_0002, 16)
            .expect("failed to build initial block");
        let first = builder.build_snapshot();
        let first_block = first
            .block_containing(0x0800_0000)
            .expect("missing initial block")
            .clone();

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let first_cycles = engine
            .step_block(&mut cpu, &first, 0x0800_0000, &first_block)
            .expect("initial step_block failed");

        assert_eq!(first_cycles, 1);
        assert_eq!(engine.blocks.get(&0x0800_0000).map(|block| block.end_pc), Some(0x0800_0000));

        cpu.next_pc = 0x0800_0000;
        builder
            .extend_from_pc(&cpu, 0x0800_0002, 0x0800_0004, 16)
            .expect("failed to extend block tail");
        let second = builder.build_snapshot();
        let second_block = second
            .block_containing(0x0800_0000)
            .expect("missing extended block")
            .clone();

        let second_cycles = engine
            .step_block(&mut cpu, &second, 0x0800_0000, &second_block)
            .expect("extended step_block failed");

        assert_eq!(second_cycles, 2);
        assert_eq!(cpu.next_pc, 0x0800_0004);
        assert_eq!(engine.compiled_block_count(), 1);
        assert_eq!(engine.blocks.get(&0x0800_0000).map(|block| block.end_pc), Some(0x0800_0002));
    }

    #[test]
    fn jit_try_step_cached_block_builder_hits_compiled_block_cache() {
        let mut cpu = build_cpu();
        cpu.load_code_bytes(0x0800_0000, &[0x00, 0xBF, 0x00, 0xBF]);
        cpu.next_pc = 0x0800_0000;

        let mut builder = JitBlockTableBuilder::new();
        builder
            .extend_from_pc(&cpu, 0x0800_0000, 0x0800_0004, 16)
            .expect("failed to build block");
        let block = builder
            .block_containing(0x0800_0000)
            .expect("missing block");

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let first_cycles = engine
            .step_block_builder(&mut cpu, &builder, 0x0800_0000, &block)
            .expect("initial step_block_builder failed");
        assert_eq!(first_cycles, 2);

        cpu.next_pc = 0x0800_0000;
        let before = engine.stats_snapshot();
        let cached = engine
            .try_step_cached_block_builder(&mut cpu, &mut builder, 0x0800_0000)
            .expect("cached block lookup failed");
        let delta = engine.stats_snapshot().delta_since(before);

        assert_eq!(cached, Some((2, 2)));
        assert_eq!(cpu.next_pc, 0x0800_0004);
        assert_eq!(delta.cache_hits, 1);
        assert_eq!(delta.cache_misses, 0);
    }

    #[test]
    fn jit_mid_block_entry_compiles_suffix_block() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x00, 0xBF, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let table = JitBlockTableBuilder::build_from_disassembly(&cs, insns.iter())
        .expect("failed to build jit table");

        let mut cpu = build_cpu();
        cpu.next_pc = 0x0800_0002;

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let cycles = engine.step(&mut cpu, &table).expect("jit suffix block step failed");

        assert_eq!(cycles, 1);
        assert_eq!(cpu.next_pc, 0x0800_0004);
        assert_eq!(engine.compiled_block_count(), 1);
        assert!(engine.blocks.get(&0x0800_0002).is_some());
        assert!(engine.blocks.get(&0x0800_0000).is_none());
    }
}
