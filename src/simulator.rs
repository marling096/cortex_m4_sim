// mod cpu;
use crate::context::CpuContext;
use crate::cpu::{Cpu, CpuExecProfile, OpExecStat};
use crate::opcodes::instruction::{Cpu_InstrTable, Cpu_Instruction};
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

pub struct Simulator {
    cpu: Cpu,
}

impl Simulator {
    pub fn new(cpu: Cpu) -> Self {
        Self { cpu }
    }

    pub fn sim_reset<'a>(
        &mut self,
        dcw_data: BTreeMap<u32, u32>,
        initial_sp: u32,
        reset_handler_ptr: u32,
    ) {
        for (addr, data) in dcw_data.iter() {
            self.cpu.write_mem(*addr, *data);
        }
        self.cpu.write_sp(initial_sp);
        let reset_handler = reset_handler_ptr & !1; //确保最低位为0，表示Thumb指令集
        print!("Reset Handler at: 0x{:08X}\n", reset_handler);
        self.cpu.write_pc(reset_handler);
        self.cpu.next_pc = reset_handler;
        self.cpu.write_mem(0x40021000, 0x0000_0083); //rcc.cr初始值
    }

    fn tick<'a>(
        &mut self,
        ins: &Cpu_Instruction<'a>,
        current_pc: u32,
        profile_enabled: bool,
    ) -> (Duration, u32) {
        let exec_start = if profile_enabled {
            Some(Instant::now())
        } else {
            None
        };
        let elapsed_cycles = self.cpu.step(&ins, current_pc);
        let exec_duration = if let Some(exec_start) = exec_start {
            exec_start.elapsed()
        } else {
            Duration::ZERO
        };

        (exec_duration, elapsed_cycles)
    }

    pub fn sim_loop<'a>(&mut self, ins_table: Cpu_InstrTable<'a>) {
        const DEFAULT_REPORT_WINDOW: u32 = 10000;
        const MIN_CALLS_FOR_TOP: u64 = 20;

        let report_window = std::env::var("SIM_REPORT_WINDOW")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(DEFAULT_REPORT_WINDOW);

        let fast_mode = std::env::var("SIM_FAST_MODE")
            .map(|v| v != "0")
            .unwrap_or(!cfg!(debug_assertions));
        let no_throttle = std::env::var("SIM_NO_THROTTLE")
            .map(|v| v != "0")
            .unwrap_or(false);
        let peripheral_tick_batch = std::env::var("SIM_PERIPH_TICK_BATCH")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(1);
        let profile_enabled = !fast_mode;
        self.cpu.set_profiling_enabled(profile_enabled);
        println!(
            "Simulator mode: {} | throttle: {} | periph batch: {}",
            if fast_mode { "FAST (profiling off)" } else { "PROFILE" }
            ,if no_throttle { "OFF" } else { "ON" }
            ,peripheral_tick_batch
        );

        if profile_enabled {
            self.sim_loop_profile(
                ins_table,
                no_throttle,
                peripheral_tick_batch,
                report_window,
                MIN_CALLS_FOR_TOP,
            );
        } else {
            self.sim_loop_fast(ins_table, no_throttle, peripheral_tick_batch, report_window);
        }
    }

    fn sim_loop_fast<'a>(
        &mut self,
        ins_table: Cpu_InstrTable<'a>,
        no_throttle: bool,
        peripheral_tick_batch: u32,
        report_window: u32,
    ) {
        let mut fetch_count: u32 = 0;
        let mut window_start = Instant::now();
        let report_window_f64 = report_window as f64;
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

        if no_throttle {
            if peripheral_tick_batch == 1 {
                loop {
                    let current_pc = self.cpu.next_pc;
                    self.cpu.prefetch_next_pc(current_pc);

                    match ins_table.get(current_pc) {
                        Some(ins) => {
                            trace_instruction!(current_pc, ins);
                            let elapsed_cycles = self.cpu.step(ins, current_pc);
                            // println!("current ins: {}  Pc: 0x{:X}", ins.op.name, current_pc);
                            self.cpu.peripheral_step_n(elapsed_cycles);

                            fetch_count += 1;
                            if fetch_count >= report_window {
                                let elapsed_secs = window_start.elapsed().as_secs_f64();
                                let _ = elapsed_secs;
                                fetch_count = 0;
                                window_start = Instant::now();
                            }
                        }
                        None => {
                            eprintln!(
                                "Error: PC 0x{:X} is out of bounds. Simulation stopped.",
                                current_pc
                            );
                            break;
                        }
                    }
                }
            } else if peripheral_tick_batch.is_power_of_two() {
                let mut pending_peripheral_cycles = 0u32;
                loop {
                    let current_pc = self.cpu.next_pc;
                    self.cpu.prefetch_next_pc(current_pc);

                    match ins_table.get(current_pc) {
                        Some(ins) => {
                            trace_instruction!(current_pc, ins);
                            let elapsed_cycles = self.cpu.step(ins, current_pc);

                            pending_peripheral_cycles = pending_peripheral_cycles.saturating_add(elapsed_cycles);
                            if pending_peripheral_cycles >= peripheral_tick_batch {
                                self.cpu.peripheral_step_n(pending_peripheral_cycles);
                                pending_peripheral_cycles = 0;
                            }

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
                        }
                        None => {
                            eprintln!(
                                "Error: PC 0x{:X} is out of bounds. Simulation stopped.",
                                current_pc
                            );
                            break;
                        }
                    }
                }
            } else {
                let mut pending_peripheral_cycles = 0u32;
                loop {
                    let current_pc = self.cpu.next_pc;
                    self.cpu.prefetch_next_pc(current_pc);

                    match ins_table.get(current_pc) {
                        Some(ins) => {
                            trace_instruction!(current_pc, ins);
                            let elapsed_cycles = self.cpu.step(ins, current_pc);

                            pending_peripheral_cycles = pending_peripheral_cycles.saturating_add(elapsed_cycles);
                            if pending_peripheral_cycles >= peripheral_tick_batch {
                                self.cpu.peripheral_step_n(pending_peripheral_cycles);
                                pending_peripheral_cycles = 0;
                            }

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
                        }
                        None => {
                            eprintln!(
                                "Error: PC 0x{:X} is out of bounds. Simulation stopped.",
                                current_pc
                            );
                            break;
                        }
                    }
                }
            }

            return;
        }

        let machine_cycle = self.cpu.machine_cycle as u32;
        if peripheral_tick_batch == 1 {
            loop {
                let frequency = self.cpu.frequency.load(Ordering::Relaxed);
                let nanos_per_tick = 1_000_000_000 / (frequency * machine_cycle);
                let tick_duration = Duration::from_nanos(nanos_per_tick as u64);
                let loop_start = Instant::now();

                let current_pc = self.cpu.next_pc;
                self.cpu.prefetch_next_pc(current_pc);

                match ins_table.get(current_pc) {
                    Some(ins) => {
                        trace_instruction!(current_pc, ins);
                        let elapsed_cycles = self.cpu.step(ins, current_pc);
                        self.cpu.peripheral_step_n(elapsed_cycles);

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
                    }
                    None => {
                        eprintln!(
                            "Error: PC 0x{:X} is out of bounds. Simulation stopped.",
                            current_pc
                        );
                        break;
                    }
                }

                let elapsed = loop_start.elapsed();
                if elapsed < tick_duration {
                    thread::sleep(tick_duration - elapsed);
                }
            }
        } else if peripheral_tick_batch.is_power_of_two() {
            let mut pending_peripheral_cycles = 0u32;
            loop {
                let frequency = self.cpu.frequency.load(Ordering::Relaxed);
                let nanos_per_tick = 1_000_000_000 / (frequency * machine_cycle);
                let tick_duration = Duration::from_nanos(nanos_per_tick as u64);
                let loop_start = Instant::now();

                let current_pc = self.cpu.next_pc;
                self.cpu.prefetch_next_pc(current_pc);

                match ins_table.get(current_pc) {
                    Some(ins) => {
                        trace_instruction!(current_pc, ins);
                        let elapsed_cycles = self.cpu.step(ins, current_pc);

                        pending_peripheral_cycles = pending_peripheral_cycles.saturating_add(elapsed_cycles);
                        if pending_peripheral_cycles >= peripheral_tick_batch {
                            self.cpu.peripheral_step_n(pending_peripheral_cycles);
                            pending_peripheral_cycles = 0;
                        }

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
                    }
                    None => {
                        eprintln!(
                            "Error: PC 0x{:X} is out of bounds. Simulation stopped.",
                            current_pc
                        );
                        break;
                    }
                }

                let elapsed = loop_start.elapsed();
                if elapsed < tick_duration {
                    thread::sleep(tick_duration - elapsed);
                }
            }
        } else {
            let mut pending_peripheral_cycles = 0u32;
            loop {
                let frequency = self.cpu.frequency.load(Ordering::Relaxed);
                let nanos_per_tick = 1_000_000_000 / (frequency * machine_cycle);
                let tick_duration = Duration::from_nanos(nanos_per_tick as u64);
                let loop_start = Instant::now();

                let current_pc = self.cpu.next_pc;
                self.cpu.prefetch_next_pc(current_pc);

                match ins_table.get(current_pc) {
                    Some(ins) => {
                        trace_instruction!(current_pc, ins);
                        let elapsed_cycles = self.cpu.step(ins, current_pc);

                        pending_peripheral_cycles = pending_peripheral_cycles.saturating_add(elapsed_cycles);
                        if pending_peripheral_cycles >= peripheral_tick_batch {
                            self.cpu.peripheral_step_n(pending_peripheral_cycles);
                            pending_peripheral_cycles = 0;
                        }

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
                    }
                    None => {
                        eprintln!(
                            "Error: PC 0x{:X} is out of bounds. Simulation stopped.",
                            current_pc
                        );
                        break;
                    }
                }

                let elapsed = loop_start.elapsed();
                if elapsed < tick_duration {
                    thread::sleep(tick_duration - elapsed);
                }
            }
        }
    }

    fn sim_loop_profile<'a>(
        &mut self,
        ins_table: Cpu_InstrTable<'a>,
        no_throttle: bool,
        peripheral_tick_batch: u32,
        report_window: u32,
        min_calls_for_top: u64,
    ) {

        fn avg_us(duration: Duration, n: u64) -> f64 {
            if n == 0 {
                0.0
            } else {
                duration.as_secs_f64() * 1_000_000.0 / n as f64
            }
        }

        let mut fetch_count: u32 = 0;
        let mut fetch_duration = Duration::new(0, 0);
        let mut loop_log_count = 0u32;

        let mut lookup_duration = Duration::ZERO;
        let mut prefetch_duration = Duration::ZERO;
        let mut exec_duration = Duration::ZERO;
        let mut peripheral_duration = Duration::ZERO;
        let mut sleep_duration = Duration::ZERO;

        let mut max_lookup = Duration::ZERO;
        let mut max_prefetch = Duration::ZERO;
        let mut max_exec = Duration::ZERO;
        let mut max_peripheral = Duration::ZERO;
        let mut pending_peripheral_cycles = 0u32;
        let machine_cycle = self.cpu.machine_cycle as u32;

        loop {
            let tick_duration = if no_throttle {
                Duration::ZERO
            } else {
                let frequency = self.cpu.frequency.load(Ordering::Relaxed);
                let nanos_per_tick = 1_000_000_000 / (frequency * machine_cycle);
                Duration::from_nanos(nanos_per_tick as u64)
            };

            let loop_start = if !no_throttle {
                Some(Instant::now())
            } else {
                None
            };
            let current_pc = self.cpu.next_pc;

            let prefetch_start = Instant::now();
            self.cpu.prefetch_next_pc(current_pc); //预取下一条指令地址
            let prefetch_elapsed = prefetch_start.elapsed();
            prefetch_duration += prefetch_elapsed;
            max_prefetch = max_prefetch.max(prefetch_elapsed);
            // println!(
            //     "Current ins: 0x{:X}  Pc: 0x{:X}",
            //     current_pc,
            //     self.cpu.read_pc()
            // );
            let test_start = Instant::now();
            let lookup_start = Instant::now();
            let fetched_ins = ins_table.get(current_pc);
            let lookup_elapsed = lookup_start.elapsed();
            lookup_duration += lookup_elapsed;
            max_lookup = max_lookup.max(lookup_elapsed);

            match fetched_ins {
                Some(ins) => {
                    let (exec_elapsed, elapsed_cycles) = self.tick(ins, current_pc, true);
                    pending_peripheral_cycles = pending_peripheral_cycles.saturating_add(elapsed_cycles);

                    let peripheral_elapsed = if pending_peripheral_cycles >= peripheral_tick_batch {
                        let peripheral_start = Instant::now();
                        self.cpu.peripheral_step_n(pending_peripheral_cycles);
                        pending_peripheral_cycles = 0;
                        peripheral_start.elapsed()
                    } else {
                        Duration::ZERO
                    };
                    exec_duration += exec_elapsed;
                    peripheral_duration += peripheral_elapsed;
                    max_exec = max_exec.max(exec_elapsed);
                    max_peripheral = max_peripheral.max(peripheral_elapsed);
                    fetch_duration += test_start.elapsed();

                    fetch_count += 1;
                    if fetch_count >= report_window {
                        let elapsed_secs = fetch_duration.as_secs_f64();

                        if elapsed_secs > 0.0 {
                            let actual_freq_hz = report_window as f64 / elapsed_secs;
                            println!(
                                "Actual Execution Frequency ({} ins): {:.6} MHz",
                                report_window,
                                actual_freq_hz / 1_000_000.0
                            );
                        }

                        let overhead_duration = fetch_duration
                            .saturating_sub(prefetch_duration)
                            .saturating_sub(lookup_duration)
                            .saturating_sub(exec_duration)
                            .saturating_sub(peripheral_duration);

                        let active_total_secs = fetch_duration.as_secs_f64();
                        let lookup_pct = if active_total_secs > 0.0 {
                            lookup_duration.as_secs_f64() * 100.0 / active_total_secs
                        } else {
                            0.0
                        };
                        let prefetch_pct = if active_total_secs > 0.0 {
                            prefetch_duration.as_secs_f64() * 100.0 / active_total_secs
                        } else {
                            0.0
                        };
                        let exec_pct = if active_total_secs > 0.0 {
                            exec_duration.as_secs_f64() * 100.0 / active_total_secs
                        } else {
                            0.0
                        };
                        let peripheral_pct = if active_total_secs > 0.0 {
                            peripheral_duration.as_secs_f64() * 100.0 / active_total_secs
                        } else {
                            0.0
                        };
                        let overhead_pct = if active_total_secs > 0.0 {
                            overhead_duration.as_secs_f64() * 100.0 / active_total_secs
                        } else {
                            0.0
                        };

                        println!(
                            "Profile ({} ins): prefetch avg {:.3}us (max {:.3}us, {:.1}%), lookup avg {:.3}us (max {:.3}us, {:.1}%), exec avg {:.3}us (max {:.3}us, {:.1}%), peripheral avg {:.3}us (max {:.3}us, {:.1}%), overhead avg {:.3}us ({:.1}%), sleep avg {:.3}us",
                            fetch_count,
                            prefetch_duration.as_secs_f64() * 1_000_000.0 / fetch_count as f64,
                            max_prefetch.as_secs_f64() * 1_000_000.0,
                            prefetch_pct,
                            lookup_duration.as_secs_f64() * 1_000_000.0 / fetch_count as f64,
                            max_lookup.as_secs_f64() * 1_000_000.0,
                            lookup_pct,
                            exec_duration.as_secs_f64() * 1_000_000.0 / fetch_count as f64,
                            max_exec.as_secs_f64() * 1_000_000.0,
                            exec_pct,
                            peripheral_duration.as_secs_f64() * 1_000_000.0 / fetch_count as f64,
                            max_peripheral.as_secs_f64() * 1_000_000.0,
                            peripheral_pct,
                            overhead_duration.as_secs_f64() * 1_000_000.0 / fetch_count as f64,
                            overhead_pct,
                            sleep_duration.as_secs_f64() * 1_000_000.0 / fetch_count as f64
                        );

                        let cpu_profile: CpuExecProfile = self.cpu.take_exec_profile();
                        let exec_calls = cpu_profile.execute_calls;
                        let step_calls = cpu_profile.step_calls;
                        let stall_calls = cpu_profile.pipeline_stall_count;

                        println!(
                            "ExecDetail: step {} exec {} stall {} | op.exec avg {:.3}us | update_pc avg {:.3}us | memR {} avg {:.3}us | memW {} avg {:.3}us | irq.check {} avg {:.3}us taken {} from_periph {} | hint.set {}",
                            step_calls,
                            exec_calls,
                            stall_calls,
                            avg_us(cpu_profile.op_exec_duration, exec_calls),
                            avg_us(cpu_profile.update_pc_duration, exec_calls),
                            cpu_profile.mem_read_count,
                            avg_us(cpu_profile.mem_read_duration, cpu_profile.mem_read_count),
                            cpu_profile.mem_write_count,
                            avg_us(cpu_profile.mem_write_duration, cpu_profile.mem_write_count),
                            cpu_profile.interrupt_check_calls,
                            avg_us(cpu_profile.interrupt_check_duration, cpu_profile.interrupt_check_calls),
                            cpu_profile.interrupt_taken_count,
                            cpu_profile.interrupt_check_from_peripheral_count,
                            cpu_profile.interrupt_hint_set_count,
                        );

                        let mut op_stats: Vec<(String, OpExecStat)> = self
                            .cpu
                            .take_exec_op_stats()
                            .into_iter()
                            .filter(|(_, stat)| stat.calls >= min_calls_for_top)
                            .collect();

                        op_stats.sort_by(|a, b| {
                            let total_cmp = b.1.total_duration.cmp(&a.1.total_duration);
                            if total_cmp == std::cmp::Ordering::Equal {
                                let a_avg = if a.1.calls == 0 {
                                    0.0
                                } else {
                                    a.1.total_duration.as_secs_f64() / a.1.calls as f64
                                };
                                let b_avg = if b.1.calls == 0 {
                                    0.0
                                } else {
                                    b.1.total_duration.as_secs_f64() / b.1.calls as f64
                                };
                                b_avg
                                    .partial_cmp(&a_avg)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            } else {
                                total_cmp
                            }
                        });

                        if !op_stats.is_empty() {
                            let top_n = op_stats.len().min(3);
                            println!("ExecTop{} Slow Mnemonic:", top_n);
                            for (idx, (mnemonic, stat)) in op_stats.iter().take(top_n).enumerate() {
                                let avg = avg_us(stat.total_duration, stat.calls);
                                let max = stat.max_duration.as_secs_f64() * 1_000_000.0;
                                let total = stat.total_duration.as_secs_f64() * 1_000_000.0;
                                println!(
                                    "  {}. {} calls {} total {:.3}us avg {:.3}us max {:.3}us",
                                    idx + 1,
                                    mnemonic,
                                    stat.calls,
                                    total,
                                    avg,
                                    max
                                );
                            }
                        } else {
                            println!(
                                "ExecTop: no mnemonic reached min samples (calls >= {})",
                                min_calls_for_top
                            );
                        }

                        fetch_count = 0;
                        fetch_duration = Duration::ZERO;
                        lookup_duration = Duration::ZERO;
                        prefetch_duration = Duration::ZERO;
                        exec_duration = Duration::ZERO;
                        peripheral_duration = Duration::ZERO;
                        sleep_duration = Duration::ZERO;
                        max_lookup = Duration::ZERO;
                        max_prefetch = Duration::ZERO;
                        max_exec = Duration::ZERO;
                        max_peripheral = Duration::ZERO;
                    }
                    // println!("tick");
                }
                None => {
                    eprintln!(
                        "Error: PC 0x{:X} is out of bounds. Simulation stopped.",
                        current_pc
                    );
                    break;
                }
            }
            // monitor.update();

            let elapsed = if let Some(loop_start) = loop_start {
                loop_start.elapsed()
            } else {
                Duration::ZERO
            };
            if !no_throttle {
                loop_log_count += 1;
                if loop_log_count >= report_window {
                    println!(
                        "Loop elapsed: {:.6} us  tick_duration: {:.6} us",
                        elapsed.as_secs_f64() * 1_000_000.0,
                        tick_duration.as_secs_f64() * 1_000_000.0
                    );
                    loop_log_count = 0;
                }
            }
            if !no_throttle && elapsed < tick_duration {
                let sleep_start = Instant::now();
                thread::sleep(tick_duration - elapsed);
                sleep_duration += sleep_start.elapsed();
            }
        }
    }
}
