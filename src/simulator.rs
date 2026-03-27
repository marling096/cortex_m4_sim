use crate::context::CpuContext;
use crate::cpu::Cpu;
use crate::jit_engine::engine::{JitEngine, JitError, JitStatsSnapshot};
use crate::jit_engine::table::{JitBlockStats, JitBlockTable};
use crate::opcodes::thumb_runtime;
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

pub struct Simulator {
    cpu: Cpu,
    system_cycles: u64,
}

impl Simulator {
    pub fn new(cpu: Cpu) -> Self {
        Self {
            cpu,
            system_cycles: 0,
        }
    }

    pub fn sim_reset<'a>(
        &mut self,
        code_segments: &[(u64, Vec<u8>)],
        dcw_data: BTreeMap<u32, u32>,
        initial_sp: u32,
        reset_handler_ptr: u32,
    ) {
        for (addr, bytes) in code_segments.iter() {
            self.cpu.load_code_bytes(*addr as u32, bytes);
        }
        for (addr, data) in dcw_data.iter() {
            self.cpu.load_bytes(*addr, &((*data as u16).to_le_bytes()));
        }
        self.cpu.write_sp(initial_sp);
        let reset_handler = reset_handler_ptr & !1; //确保最低位为0，表示Thumb指令集
        print!("Reset Handler at: 0x{:08X}\n", reset_handler);
        self.cpu.write_pc(reset_handler);
        self.cpu.next_pc = reset_handler;
        self.cpu.write_mem(0x40021000, 0x0000_0083); //rcc.cr初始值
        self.system_cycles = 0;
    }

    #[inline(always)]
    fn advance_system_cycles(&mut self, elapsed_cycles: u32) {
        self.system_cycles = self.system_cycles.saturating_add(elapsed_cycles as u64);
    }

    #[inline(always)]
    fn maybe_drive_peripherals(
        &mut self,
        pending_peripheral_cycles: &mut u32,
        max_lag_cycles: u32,
    ) -> bool {
        if self.cpu.take_and_clear_peripheral_schedule_dirty() {
            self
                .cpu
                .refresh_peripheral_due_cycle(self.system_cycles, max_lag_cycles);
        }

        if *pending_peripheral_cycles == 0 {
            return false;
        }

        if self.system_cycles < self.cpu.peripheral_due_cycle() {
            return false;
        }

        let cycles = *pending_peripheral_cycles;
        *pending_peripheral_cycles = 0;
        self.cpu.peripheral_step_n(cycles);
        self
            .cpu
            .refresh_peripheral_due_cycle(self.system_cycles, max_lag_cycles);
        true
    }

    pub fn sim_loop_interpreter(&mut self) -> Result<(), String> {
        const DEFAULT_REPORT_WINDOW: u32 = 10000;

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
            "Simulator mode: FAST | throttle: {} | periph batch: {}",
            if no_throttle { "OFF" } else { "ON" },
            peripheral_tick_batch
        );

        self.sim_loop_fast(no_throttle, peripheral_tick_batch, report_window)
    }

    pub fn sim_loop_jit(
        &mut self,
        jit_table: JitBlockTable,
    ) -> Result<(), JitError> {
        const DEFAULT_REPORT_WINDOW: u32 = 10000;

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

        self.sim_loop_fast_jit(jit_table, no_throttle, peripheral_tick_batch, report_window)
    }

    fn sim_loop_fast(
        &mut self,
        no_throttle: bool,
        peripheral_tick_batch: u32,
        report_window: u32,
    ) -> Result<(), String> {
        let mut fetch_count: u32 = 0;
        let mut window_start = Instant::now();
        let report_window_f64 = report_window as f64;
        let mut pending_peripheral_cycles = 0u32;
        self
            .cpu
            .refresh_peripheral_due_cycle(self.system_cycles, peripheral_tick_batch);
        self.cpu.take_and_clear_peripheral_schedule_dirty();
        let trace_insn = std::env::var("SIM_TRACE_INSN")
            .map(|v| v != "0")
            .unwrap_or(false);
        let trace_limit = std::env::var("SIM_TRACE_LIMIT")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);
        let mut trace_count: u64 = 0;
        let mut trace_limit_reached = false;

        let machine_cycle = self.cpu.machine_cycle as u32;
        loop {
            let loop_start = if no_throttle {
                None
            } else {
                Some(Instant::now())
            };

            let current_pc = self.cpu.next_pc;
            self.cpu.prefetch_next_pc(current_pc);
            let step_outcome = match thumb_runtime::step(&mut self.cpu, current_pc) {
                Ok(outcome) => outcome,
                Err(err) => {
                    eprintln!("Error: {err}. Simulation stopped.");
                    break;
                }
            };

            if trace_insn && !trace_limit_reached {
                if trace_limit == 0 || trace_count < trace_limit {
                    println!(
                        "[TRACE] PC=0x{:08X} {} {}",
                        current_pc,
                        step_outcome.mnemonic,
                        step_outcome.op_str
                    );
                    trace_count += 1;
                    if trace_limit != 0 && trace_count >= trace_limit {
                        println!("[TRACE] limit reached: {}", trace_limit);
                        trace_limit_reached = true;
                    }
                }
            }

            let elapsed_cycles = step_outcome.cycles;

            self.advance_system_cycles(elapsed_cycles);

            pending_peripheral_cycles =
                pending_peripheral_cycles.saturating_add(elapsed_cycles);
            self.maybe_drive_peripherals(
                &mut pending_peripheral_cycles,
                peripheral_tick_batch,
            );

            fetch_count += 1;
            if fetch_count >= report_window {
                let elapsed_secs = window_start.elapsed().as_secs_f64();
                if elapsed_secs > 0.0 {
                    let actual_freq_hz = report_window_f64 / elapsed_secs;
                    println!(
                        "Actual Execution Frequency ({} ins): {:.6} MHz",
                        report_window,
                        actual_freq_hz / 1_000_000.0
                    );
                }
                fetch_count = 0;
                window_start = Instant::now();
            }

            if let Some(loop_start) = loop_start {
                let frequency = self.cpu.frequency.load(Ordering::Relaxed);
                let nanos_per_tick = 1_000_000_000 / (frequency * machine_cycle);
                let tick_duration = Duration::from_nanos(nanos_per_tick as u64);
                let elapsed = loop_start.elapsed();
                if elapsed < tick_duration {
                    thread::sleep(tick_duration - elapsed);
                }
            }
        }

        Ok(())
    }

    fn sim_loop_fast_jit(
        &mut self,
        jit_table: JitBlockTable,
        no_throttle: bool,
        peripheral_tick_batch: u32,
        report_window: u32,
    ) -> Result<(), JitError> {
        let mut engine = JitEngine::new()?;
        let report_jit_stats = std::env::var("SIM_JIT_STATS")
            .map(|v| v != "0")
            .unwrap_or(false);
        let table_block_stats = jit_table.block_stats();
        let mut last_jit_stats = engine.stats_snapshot();

        if report_jit_stats {
            Self::print_jit_table_stats(table_block_stats);
        }
        let mut fetch_count: u32 = 0;
        let mut window_start = Instant::now();
        let report_window_f64 = report_window as f64;
        let mut pending_peripheral_cycles = 0u32;
        self
            .cpu
            .refresh_peripheral_due_cycle(self.system_cycles, peripheral_tick_batch);
        self.cpu.take_and_clear_peripheral_schedule_dirty();
        let trace_insn = std::env::var("SIM_TRACE_INSN")
            .map(|v| v != "0")
            .unwrap_or(false);
        let trace_limit = std::env::var("SIM_TRACE_LIMIT")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);
        let mut trace_count: u64 = 0;
        let mut trace_limit_reached = false;

        macro_rules! trace_instruction {
            ($pc:expr, $ins:expr) => {
                if trace_insn && !trace_limit_reached {
                    if trace_limit == 0 || trace_count < trace_limit {
                        println!(
                            "[TRACE] PC=0x{:08X} {} {}",
                            $pc,
                            $ins.data.mnemonic(),
                            $ins.data.op_str()
                        );
                        trace_count += 1;
                        if trace_limit != 0 && trace_count >= trace_limit {
                            println!("[TRACE] limit reached: {}", trace_limit);
                            trace_limit_reached = true;
                        }
                    }
                }
            };
        }

        let machine_cycle = self.cpu.machine_cycle as u32;
        loop {
            let loop_start = if no_throttle {
                None
            } else {
                Some(Instant::now())
            };

            let current_pc = self.cpu.next_pc;
            if let Some(ins) = jit_table.get(current_pc) {
                trace_instruction!(current_pc, ins);
            }

            let elapsed_cycles = match engine.step(&mut self.cpu, &jit_table) {
                Ok(cycles) => cycles,
                Err(JitError::MissingInstruction { pc }) => {
                    eprintln!(
                        "Error: PC 0x{:X} is out of bounds. Simulation stopped.",
                        pc
                    );
                    break;
                }
                Err(err) => return Err(err),
            };
            self.advance_system_cycles(elapsed_cycles);

            pending_peripheral_cycles = pending_peripheral_cycles.saturating_add(elapsed_cycles);
            self.maybe_drive_peripherals(&mut pending_peripheral_cycles, peripheral_tick_batch);

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
                if report_jit_stats {
                    let current_stats = engine.stats_snapshot();
                    let delta = current_stats.delta_since(last_jit_stats);
                    Self::print_jit_runtime_stats(delta);
                    last_jit_stats = current_stats;
                }
                fetch_count = 0;
                window_start = Instant::now();
            }

            if let Some(loop_start) = loop_start {
                let frequency = self.cpu.frequency.load(Ordering::Relaxed);
                let nanos_per_tick = 1_000_000_000 / (frequency * machine_cycle);
                let tick_duration = Duration::from_nanos(nanos_per_tick as u64);
                let elapsed = loop_start.elapsed();
                if elapsed < tick_duration {
                    thread::sleep(tick_duration - elapsed);
                }
            }
        }

        Ok(())
    }

    fn print_jit_table_stats(stats: JitBlockStats) {
        println!(
            "JIT block table: blocks={} avg_len={:.2} terminators: branch={} target={} pc_write={} it_end={} exret={} gap={} eof={}",
            stats.block_count,
            stats.average_block_len(),
            stats.branch_blocks,
            stats.branch_target_blocks,
            stats.pc_write_blocks,
            stats.it_block_end_blocks,
            stats.exception_return_blocks,
            stats.gap_blocks,
            stats.end_of_table_blocks,
        );
    }

    fn print_jit_runtime_stats(stats: JitStatsSnapshot) {
        println!(
            concat!(
                "JIT stats: exec_blocks={} avg_exec_len={:.2} cache_hit={:.1}% ",
                "compiled={} suffix={} avg_compiled_len={:.2} fallback={} ",
                "helpers/ins={:.2} reg_r={} reg_w={} mem_r={} mem_w={} ",
                "op2={} mem_addr={} shift={} flags={} cc={} exret={}"
            ),
            stats.executed_blocks,
            stats.average_executed_block_len(),
            stats.cache_hit_rate() * 100.0,
            stats.compiled_blocks,
            stats.compiled_suffix_blocks,
            stats.average_compiled_block_len(),
            stats.fallback_calls,
            stats.helper_calls_per_guest_instruction(),
            stats.read_reg_calls,
            stats.write_reg_calls,
            stats.mem_read_calls,
            stats.mem_write_calls,
            stats.resolve_op2_calls,
            stats.resolve_mem_rt_addr_calls,
            stats.compute_shift_calls,
            stats.flag_update_calls,
            stats.check_condition_calls,
            stats.exception_return_calls,
        );
    }
}
