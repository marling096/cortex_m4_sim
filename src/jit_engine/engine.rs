use std::fmt;
use std::thread;
use std::time::{Duration, Instant};

use capstone::arch::arm::{ArmCC, ArmInsn, ArmOperandType};
use cranelift::codegen::ir::FuncRef;
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module, default_libcall_names};
use rustc_hash::FxHashMap;

use crate::context::CpuContext;
use crate::cpu::Cpu;
use crate::jit_engine::clif::instructions as jit_instructions;
use crate::jit_engine::table::{JitInstruction, JitInstructionTable};
use crate::opcodes::opcode::{
    UpdateApsr_C, UpdateApsr_N, UpdateApsr_V, UpdateApsr_Z, check_condition,
    operand_resolver_multi_runtime, resolve_op2_runtime,
};

pub type JitBlockFn = unsafe extern "C" fn(*mut Cpu, *const ()) -> u32;

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
}

pub(crate) struct RuntimeFunctions {
    pub(crate) check_condition: FuncId,
    pub(crate) read_reg: FuncId,
    pub(crate) write_reg: FuncId,
    pub(crate) read_apsr: FuncId,
    pub(crate) read_u8: FuncId,
    pub(crate) read_u16: FuncId,
    pub(crate) read_u32: FuncId,
    pub(crate) write_u8: FuncId,
    pub(crate) write_u16: FuncId,
    pub(crate) write_u32: FuncId,
    pub(crate) update_apsr_n: FuncId,
    pub(crate) update_apsr_z: FuncId,
    pub(crate) update_apsr_c: FuncId,
    pub(crate) update_apsr_v: FuncId,
    pub(crate) try_exception_return: FuncId,
    pub(crate) resolve_op2_packed: FuncId,
    pub(crate) resolve_simple_op2_value: FuncId,
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
    pub ptr_ty: Type,
    pub cpu_ptr: Value,
    pub instr_ptr: Value,
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

    pub(crate) fn emit_fallback(&mut self) -> Value {
        self.call_value(self.helpers.fallback_exec, &[self.cpu_ptr, self.instr_ptr])
    }
}

pub struct JitEngine {
    module: JITModule,
    builder_ctx: FunctionBuilderContext,
    helpers: RuntimeFunctions,
    blocks: FxHashMap<u32, CompiledBlock>,
    next_function_index: u32,
}

impl JitEngine {
    pub fn new() -> Result<Self, JitError> {
        let mut builder = JITBuilder::new(default_libcall_names())
            .map_err(|err| JitError::Backend(err.to_string()))?;
        builder.symbol("jit_check_condition", jit_check_condition as *const u8);
        builder.symbol("jit_read_reg", jit_read_reg as *const u8);
        builder.symbol("jit_write_reg", jit_write_reg as *const u8);
        builder.symbol("jit_read_apsr", jit_read_apsr as *const u8);
        builder.symbol("jit_read_u8", jit_read_u8 as *const u8);
        builder.symbol("jit_read_u16", jit_read_u16 as *const u8);
        builder.symbol("jit_read_u32", jit_read_u32 as *const u8);
        builder.symbol("jit_write_u8", jit_write_u8 as *const u8);
        builder.symbol("jit_write_u16", jit_write_u16 as *const u8);
        builder.symbol("jit_write_u32", jit_write_u32 as *const u8);
        builder.symbol("jit_update_apsr_n", jit_update_apsr_n as *const u8);
        builder.symbol("jit_update_apsr_z", jit_update_apsr_z as *const u8);
        builder.symbol("jit_update_apsr_c", jit_update_apsr_c as *const u8);
        builder.symbol("jit_update_apsr_v", jit_update_apsr_v as *const u8);
        builder.symbol(
            "jit_try_exception_return",
            jit_try_exception_return as *const u8,
        );
        builder.symbol("jit_resolve_op2_packed", jit_resolve_op2_packed as *const u8);
        builder.symbol(
            "jit_resolve_simple_op2_value",
            jit_resolve_simple_op2_value as *const u8,
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
            next_function_index: 0,
        })
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

    pub fn compile_instruction_entries<'a, I>(
        &mut self,
        entries: I,
    ) -> Result<Vec<(u32, JitBlockFn)>, JitError>
    where
        I: IntoIterator<Item = (u32, &'a JitInstruction<'a>)>,
    {
        let mut entries: Vec<_> = entries.into_iter().collect();
        entries.sort_unstable_by_key(|(pc, _)| *pc);

        for (pc, ins) in entries {
            if self.blocks.contains_key(&pc) {
                continue;
            }

            let compiled = self.compile_instruction(pc, ins)?;
            self.blocks.insert(pc, compiled);
        }

        Ok(self.compiled_entries())
    }

    pub fn compile_table<'a>(
        &mut self,
        table: &JitInstructionTable<'a>,
    ) -> Result<Vec<(u32, JitBlockFn)>, JitError> {
        self.compile_instruction_entries(table.iter_entries())
    }

    pub fn step<'a>(&mut self, cpu: &mut Cpu, table: &JitInstructionTable<'a>) -> Result<u32, JitError> {
        if let Some(cycles) = cpu.begin_step() {
            return Ok(cycles);
        }

        let current_pc = cpu.next_pc;
        if table.block_starting_at(current_pc).is_some() {
            self.execute_block(cpu, table, current_pc)
        } else {
            self.execute_single(cpu, table, current_pc)
        }
    }

    pub fn run<'a>(&mut self, cpu: &mut Cpu, table: &JitInstructionTable<'a>) -> Result<(), JitError> {
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
        table: &JitInstructionTable<'a>,
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

    fn execute_single<'a>(
        &mut self,
        cpu: &mut Cpu,
        table: &JitInstructionTable<'a>,
        current_pc: u32,
    ) -> Result<u32, JitError> {
        let ins = table
            .get(current_pc)
            .ok_or(JitError::MissingInstruction { pc: current_pc })?;
        self.execute_instruction(cpu, current_pc, ins)
    }

    fn execute_block<'a>(
        &mut self,
        cpu: &mut Cpu,
        table: &JitInstructionTable<'a>,
        start_pc: u32,
    ) -> Result<u32, JitError> {
        let Some(block) = table.block_starting_at(start_pc) else {
            return self.execute_single(cpu, table, start_pc);
        };

        let mut total_cycles = 0u32;
        let mut current_pc = block.start_pc;

        loop {
            let ins = table
                .get(current_pc)
                .ok_or(JitError::MissingInstruction { pc: current_pc })?;
            let expected_next_pc = current_pc.wrapping_add(ins.data.size());

            total_cycles = total_cycles.saturating_add(self.execute_instruction(cpu, current_pc, ins)?);

            if current_pc == block.end_pc {
                break;
            }

            if cpu.next_pc != expected_next_pc {
                break;
            }

            while let Some(cycles) = cpu.begin_step() {
                total_cycles = total_cycles.saturating_add(cycles);
                if cpu.next_pc != expected_next_pc {
                    return Ok(total_cycles);
                }
            }

            if cpu.next_pc != expected_next_pc {
                break;
            }

            current_pc = cpu.next_pc;
        }

        Ok(total_cycles)
    }

    fn execute_instruction<'a>(
        &mut self,
        cpu: &mut Cpu,
        current_pc: u32,
        ins: &JitInstruction<'a>,
    ) -> Result<u32, JitError> {
        cpu.prefetch_next_pc(current_pc);

        let block = self.get_or_compile(current_pc, ins)?;
        let pc_update = unsafe { (block.entry)(cpu as *mut Cpu, ins as *const _ as *const ()) };

        Ok(cpu.finish_step_cycles(ins.op.cycles.execute_cycles, current_pc, pc_update))
    }

    fn get_or_compile<'a>(
        &mut self,
        pc: u32,
        ins: &JitInstruction<'a>,
    ) -> Result<&CompiledBlock, JitError> {
        if !self.blocks.contains_key(&pc) {
            let compiled = self.compile_instruction(pc, ins)?;
            self.blocks.insert(pc, compiled);
        }

        Ok(self
            .blocks
            .get(&pc)
            .expect("compiled block missing after insert"))
    }

    fn compile_instruction<'a>(
        &mut self,
        pc: u32,
        ins: &JitInstruction<'a>,
    ) -> Result<CompiledBlock, JitError> {
        let ptr_ty = self.module.target_config().pointer_type();
        let mut ctx = self.module.make_context();
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(ptr_ty));
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
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            let cpu_ptr = builder.block_params(entry_block)[0];
            let instr_ptr = builder.block_params(entry_block)[1];

            let mut lowering = LoweringContext {
                builder: &mut builder,
                module: &mut self.module,
                helpers: &self.helpers,
                ptr_ty,
                cpu_ptr,
                instr_ptr,
            };

            if !Self::try_lower_instruction(&mut lowering, ins) {
                let result = lowering.emit_fallback();
                lowering.builder.ins().return_(&[result]);
            }

            builder.finalize();
        }

        self.module.define_function(func_id, &mut ctx)?;
        self.module.clear_context(&mut ctx);
        self.module.finalize_definitions()?;

        let code = self.module.get_finalized_function(func_id);
        let entry = unsafe { std::mem::transmute::<*const u8, JitBlockFn>(code) };

        Ok(CompiledBlock { entry })
    }

    fn try_lower_instruction<'a>(
        lowering: &mut LoweringContext<'_, '_>,
        ins: &JitInstruction<'a>,
    ) -> bool {
        match ins.def.or_else(|| jit_instructions::find_def(ins.op.insnid)) {
            Some(def) if def.supports(ins) => {
                def.execute(lowering, ins);
                true
            }
            _ => false,
        }
    }
}

impl RuntimeFunctions {
    fn declare(module: &mut JITModule) -> Result<Self, JitError> {
        let ptr_ty = module.target_config().pointer_type();

        let check_condition = declare_import(module, "jit_check_condition", |sig| {
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

        let update_apsr_n = declare_import(module, "jit_update_apsr_n", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(types::I32));
        })?;

        let update_apsr_z = declare_import(module, "jit_update_apsr_z", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(types::I32));
        })?;

        let update_apsr_c = declare_import(module, "jit_update_apsr_c", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(types::I32));
        })?;

        let update_apsr_v = declare_import(module, "jit_update_apsr_v", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(types::I32));
        })?;

        let try_exception_return = declare_import(module, "jit_try_exception_return", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(types::I32));
            sig.returns.push(AbiParam::new(types::I32));
        })?;

        let resolve_op2_packed = declare_import(module, "jit_resolve_op2_packed", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(ptr_ty));
            sig.returns.push(AbiParam::new(types::I64));
        })?;

        let resolve_simple_op2_value = declare_import(module, "jit_resolve_simple_op2_value", |sig| {
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(ptr_ty));
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
            check_condition,
            read_reg,
            write_reg,
            read_apsr,
            read_u8,
            read_u16,
            read_u32,
            write_u8,
            write_u16,
            write_u32,
            update_apsr_n,
            update_apsr_z,
            update_apsr_c,
            update_apsr_v,
            try_exception_return,
            resolve_op2_packed,
            resolve_simple_op2_value,
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

fn decode_condition(cc: u32) -> ArmCC {
    match cc {
        x if x == ArmCC::ARM_CC_EQ as u32 => ArmCC::ARM_CC_EQ,
        x if x == ArmCC::ARM_CC_NE as u32 => ArmCC::ARM_CC_NE,
        x if x == ArmCC::ARM_CC_HS as u32 => ArmCC::ARM_CC_HS,
        x if x == ArmCC::ARM_CC_LO as u32 => ArmCC::ARM_CC_LO,
        x if x == ArmCC::ARM_CC_MI as u32 => ArmCC::ARM_CC_MI,
        x if x == ArmCC::ARM_CC_PL as u32 => ArmCC::ARM_CC_PL,
        x if x == ArmCC::ARM_CC_VS as u32 => ArmCC::ARM_CC_VS,
        x if x == ArmCC::ARM_CC_VC as u32 => ArmCC::ARM_CC_VC,
        x if x == ArmCC::ARM_CC_HI as u32 => ArmCC::ARM_CC_HI,
        x if x == ArmCC::ARM_CC_LS as u32 => ArmCC::ARM_CC_LS,
        x if x == ArmCC::ARM_CC_GE as u32 => ArmCC::ARM_CC_GE,
        x if x == ArmCC::ARM_CC_LT as u32 => ArmCC::ARM_CC_LT,
        x if x == ArmCC::ARM_CC_GT as u32 => ArmCC::ARM_CC_GT,
        x if x == ArmCC::ARM_CC_LE as u32 => ArmCC::ARM_CC_LE,
        _ => ArmCC::ARM_CC_AL,
    }
}

extern "C" fn jit_check_condition(cpu: *mut Cpu, cc: u32) -> u32 {
    let cpu = unsafe { &*cpu };
    u32::from(check_condition(cpu, decode_condition(cc)))
}

extern "C" fn jit_read_reg(cpu: *mut Cpu, reg: u32) -> u32 {
    let cpu = unsafe { &*cpu };
    cpu.read_reg(reg)
}

extern "C" fn jit_write_reg(cpu: *mut Cpu, reg: u32, value: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.write_reg(reg, value);
}

extern "C" fn jit_read_apsr(cpu: *mut Cpu) -> u32 {
    let cpu = unsafe { &*cpu };
    cpu.read_apsr()
}

extern "C" fn jit_read_u32(cpu: *mut Cpu, addr: u32) -> u32 {
    let cpu = unsafe { &*cpu };
    cpu.read_mem(addr)
}

extern "C" fn jit_read_u8(cpu: *mut Cpu, addr: u32) -> u32 {
    let cpu = unsafe { &*cpu };
    let word = cpu.read_mem(addr & !3);
    let shift = (addr & 3) * 8;
    (word >> shift) & 0xFF
}

extern "C" fn jit_read_u16(cpu: *mut Cpu, addr: u32) -> u32 {
    let cpu = unsafe { &*cpu };
    let word = cpu.read_mem(addr & !3);
    let shift = (addr & 2) * 8;
    (word >> shift) & 0xFFFF
}

extern "C" fn jit_write_u32(cpu: *mut Cpu, addr: u32, value: u32) {
    let cpu = unsafe { &mut *cpu };
    cpu.write_mem(addr, value);
}

extern "C" fn jit_write_u8(cpu: *mut Cpu, addr: u32, value: u32) {
    let cpu = unsafe { &mut *cpu };
    let aligned_addr = addr & !3;
    let shift = (addr & 3) * 8;
    let mask = !(0xFF << shift);
    let current = cpu.read_mem(aligned_addr);
    let new_value = (current & mask) | ((value & 0xFF) << shift);
    cpu.write_mem(aligned_addr, new_value);
}

extern "C" fn jit_write_u16(cpu: *mut Cpu, addr: u32, value: u32) {
    let cpu = unsafe { &mut *cpu };
    let aligned_addr = addr & !3;
    let shift = (addr & 2) * 8;
    let mask = !(0xFFFF << shift);
    let current = cpu.read_mem(aligned_addr);
    let new_value = (current & mask) | ((value & 0xFFFF) << shift);
    cpu.write_mem(aligned_addr, new_value);
}

extern "C" fn jit_update_apsr_n(cpu: *mut Cpu, value: u32) {
    let cpu = unsafe { &mut *cpu };
    UpdateApsr_N(cpu, value);
}

extern "C" fn jit_update_apsr_z(cpu: *mut Cpu, value: u32) {
    let cpu = unsafe { &mut *cpu };
    UpdateApsr_Z(cpu, value);
}

extern "C" fn jit_update_apsr_c(cpu: *mut Cpu, value: u32) {
    let cpu = unsafe { &mut *cpu };
    UpdateApsr_C(cpu, value as u8);
}

extern "C" fn jit_update_apsr_v(cpu: *mut Cpu, value: u32) {
    let cpu = unsafe { &mut *cpu };
    UpdateApsr_V(cpu, value as u8);
}

extern "C" fn jit_try_exception_return(cpu: *mut Cpu, value: u32) -> u32 {
    let cpu = unsafe { &mut *cpu };
    u32::from(cpu.try_exception_return(value))
}

extern "C" fn jit_resolve_op2_packed(cpu: *mut Cpu, instr: *const ()) -> u64 {
    let cpu = unsafe { &mut *cpu };
    let instr = unsafe { &*(instr as *const JitInstruction<'static>) };
    let (value, carry) = resolve_op2_runtime(cpu, &instr.data);
    ((carry as u64) << 32) | u64::from(value)
}

extern "C" fn jit_resolve_simple_op2_value(cpu: *mut Cpu, instr: *const ()) -> u32 {
    let cpu = unsafe { &mut *cpu };
    let instr = unsafe { &*(instr as *const JitInstruction<'static>) };

    match &instr.data.arm_operands.op2 {
        Some(op) => match op.op_type {
            ArmOperandType::Imm(imm) => imm as u32,
            ArmOperandType::Reg(reg) => cpu.read_reg(instr.data.resolve_reg(reg)),
            _ => 0,
        },
        None => 0,
    }
}

extern "C" fn jit_resolve_mem_rt_addr(cpu: *mut Cpu, instr: *const ()) -> u64 {
    let cpu = unsafe { &mut *cpu };
    let instr = unsafe { &*(instr as *const JitInstruction<'static>) };
    let (rt, addr) = operand_resolver_multi_runtime(cpu, &instr.data);
    (u64::from(addr) << 32) | u64::from(rt)
}

extern "C" fn jit_compute_shift_packed(cpu: *mut Cpu, instr: *const ()) -> u64 {
    let cpu = unsafe { &mut *cpu };
    let instr = unsafe { &*(instr as *const JitInstruction<'static>) };
    let rm = instr.data.arm_operands.rn;
    let rm_val = cpu.read_reg(rm);
    let current_c = ((cpu.read_apsr() >> 29) & 1) as u8;
    let shift_amount = match &instr.data.arm_operands.op2 {
        Some(op) => match op.op_type {
            ArmOperandType::Imm(imm) => (imm as u32) & 0xFF,
            ArmOperandType::Reg(reg) => cpu.read_reg(instr.data.resolve_reg(reg)) & 0xFF,
            _ => 0,
        },
        None => 0,
    };

    let (result, carry) = match instr.op.insnid {
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
    let cpu = unsafe { &mut *cpu };
    let instr = unsafe { &*(instr as *const JitInstruction<'static>) };

    let imm = match &instr.data.arm_operands.op2 {
        Some(op) => match op.op_type {
            ArmOperandType::Imm(imm) => imm as u32,
            ArmOperandType::Reg(reg) => cpu.read_reg(instr.data.resolve_reg(reg)),
            _ => 0,
        },
        None => 0,
    };

    println!("BKPT #{}", imm);
}

extern "C" fn jit_udiv_or_zero(lhs: u32, rhs: u32) -> u32 {
    if rhs == 0 { 0 } else { lhs / rhs }
}

extern "C" fn jit_execute_fallback(cpu: *mut Cpu, instr: *const ()) -> u32 {
    let cpu = unsafe { &mut *cpu };
    let instr = unsafe { &*(instr as *const JitInstruction<'static>) };
    (instr.op.exec)(cpu, &instr.data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use capstone::arch;
    use capstone::prelude::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicU32;

    use crate::cpu::Cpu;
    use crate::jit_engine::table::JitInstructionTableBuilder;
    use crate::opcodes::instruction::OpcodeTable;
    use crate::peripheral::bus::Bus;
    use crate::peripheral::nvic::Nvic;
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


    #[test]
    fn jit_falls_back_for_unsupported_instruction() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let opcode_table = OpcodeTable::new();
        let table = JitInstructionTableBuilder::build_from_disassembly(
            &opcode_table,
            &cs,
            insns.iter(),
        )
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
        let opcode_table = OpcodeTable::new();
        let table = JitInstructionTableBuilder::build_from_disassembly(
            &opcode_table,
            &cs,
            insns.iter(),
        )
        .expect("failed to build jit table");

        let mut cpu = build_cpu();
        cpu.next_pc = 0x0800_0000;

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let cycles = engine.step(&mut cpu, &table).expect("jit block step failed");

        assert_eq!(cycles, 2);
        assert_eq!(cpu.next_pc, 0x0800_0004);
        assert_eq!(engine.compiled_block_count(), 2);
        assert!(engine.blocks.get(&0x0800_0000).is_some());
        assert!(engine.blocks.get(&0x0800_0002).is_some());
    }

    #[test]
    fn jit_compile_table_precompiles_all_entries() {
        let cs = build_thumb_capstone();
        let insns = cs
            .disasm_all(&[0x08, 0x68, 0x00, 0xBF], 0x0800_0000)
            .expect("failed to disassemble");
        let opcode_table = OpcodeTable::new();
        let table = JitInstructionTableBuilder::build_from_disassembly(
            &opcode_table,
            &cs,
            insns.iter(),
        )
        .expect("failed to build jit table");

        let mut engine = JitEngine::new().expect("failed to create jit engine");
        let compiled = engine
            .compile_table(&table)
            .expect("failed to compile table");

        assert_eq!(compiled.len(), 2);
        assert!(engine.compiled_entry(0x0800_0000).is_some());
        assert!(engine.compiled_entry(0x0800_0002).is_some());
        assert!(engine.blocks.get(&0x0800_0000).is_some());
        assert!(engine.blocks.get(&0x0800_0002).is_some());
    }
}
