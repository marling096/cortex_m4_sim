#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::hint::black_box;
    use std::path::Path;
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Instant;

    use crate::context::CpuContext;
    use crate::cpu::{Cpu, OpExecStat};
    use crate::disassembler::disassemble_from_reset_handler;
    use crate::opcodes::Instructions;
    use crate::opcodes::instruction::{Cpu_InstrTable, Cpu_Instruction, OpcodeTable};
    use crate::opcodes::opcode::ArmOpcode;
    use crate::peripheral::afio::Afio;
    use crate::peripheral::bus::Bus;
    use crate::peripheral::timer::GeneralTimer;
    use crate::peripheral::peripheral::Peripheral;
    use crate::peripheral::flash::Flash;
    use crate::peripheral::gpio::Gpio;
    use crate::peripheral::nvic::Nvic;
    use crate::peripheral::rcc::Rcc;
    use crate::peripheral::scb::Scb;
    use crate::peripheral::systick::SysTick;
    use crate::peripheral::uart::Uart;

    #[derive(Clone)]
    struct InstructionPerfRow {
        group: String,
        definition: String,
        exec_calls: u64,
        elapsed: std::time::Duration,
        matched_mnemonics: Vec<String>,
    }

    static INSTRUCTION_EXEC_STATS_CACHE: OnceLock<Result<HashMap<String, OpExecStat>, String>> = OnceLock::new();
    static INSTRUCTION_DEFINITION_PERF_CACHE: OnceLock<Result<Vec<InstructionPerfRow>, String>> = OnceLock::new();
    const INSTRUCTION_PERF_REPEAT_ROUNDS: usize = 5;

    fn report_perf(name: &str, iterations: u64, elapsed: std::time::Duration) {
        let ns_per_op = elapsed.as_nanos() as f64 / iterations as f64;
        let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();
        println!(
            "[perf] {name}: iter={iterations}, total={:?}, {:.2} ns/op, {:.2} ops/s",
            elapsed, ns_per_op, ops_per_sec
        );
    }

    fn estimate_exec_timer_overhead(exec_calls: u64) -> std::time::Duration {
        if exec_calls == 0 {
            return std::time::Duration::ZERO;
        }

        // Mirror the profiling timing envelope used in cpu.step:
        // Instant::now(); <exec body>; elapsed().
        // Here we measure an empty body and use it as sampling overhead calibration.
        let mut measured = std::time::Duration::ZERO;
        for _ in 0..exec_calls {
            let op_start = Instant::now();
            black_box(());
            measured += op_start.elapsed();
        }
        measured
    }

    fn write_instruction_exec_perf_csv(
        rows: &[(String, u64, std::time::Duration)],
        repeat_rounds: usize,
    ) -> std::io::Result<()> {
        let dir = Path::new("perf_reports");
        fs::create_dir_all(dir)?;
        let path = dir.join("instruction_exec_mnemonic_sorted.csv");

        let mut csv = String::from("rank,mnemonic,total_ns,total_ms,avg_ns_per_exec,exec_calls,repeat_rounds\n");

        for (index, (mnemonic, exec_calls, elapsed)) in rows.iter().enumerate() {
            let total_ns = elapsed.as_nanos() as f64;
            let total_ms = elapsed.as_secs_f64() * 1000.0;
            let ns_per_op = if *exec_calls == 0 {
                0.0
            } else {
                total_ns / (*exec_calls as f64)
            };
            csv.push_str(&format!(
                "{},{},{:.0},{:.6},{:.4},{},{}\n",
                index + 1,
                mnemonic,
                total_ns,
                total_ms,
                ns_per_op,
                exec_calls,
                repeat_rounds
            ));
        }

        fs::write(&path, csv)?;
        println!("[perf] csv report written: {}", path.display());
        Ok(())
    }

    fn write_instruction_definition_exec_perf_csv(
        rows: &[InstructionPerfRow],
        repeat_rounds: usize,
    ) -> std::io::Result<()> {
        let dir = Path::new("perf_reports");
        fs::create_dir_all(dir)?;
        let path = dir.join("instruction_exec_definition_sorted.csv");

        let mut csv = String::from(
            "rank,instruction_group,definition,total_ns,total_ms,avg_ns_per_exec,exec_calls,repeat_rounds,matched_mnemonics\n",
        );

        for (index, row) in rows.iter().enumerate() {
            let total_ns = row.elapsed.as_nanos() as f64;
            let total_ms = row.elapsed.as_secs_f64() * 1000.0;
            let ns_per_op = if row.exec_calls == 0 {
                0.0
            } else {
                total_ns / row.exec_calls as f64
            };
            let matched = if row.matched_mnemonics.is_empty() {
                "-".to_string()
            } else {
                row.matched_mnemonics.join("|")
            };
            csv.push_str(&format!(
                "{},{},{},{:.0},{:.6},{:.4},{},{},{}\n",
                index + 1,
                row.group,
                row.definition,
                total_ns,
                total_ms,
                ns_per_op,
                row.exec_calls,
                repeat_rounds,
                matched
            ));
        }

        fs::write(&path, csv)?;
        println!("[perf] csv report written: {}", path.display());
        Ok(())
    }

    fn collect_instruction_exec_stats() -> Result<HashMap<String, OpExecStat>, String> {
        let step_budget_per_axf = 150_000u64;
        let axf_candidates = [
            "uart_loop.axf",
            "uart_helloworld.axf",
            "timer.axf",
            "io_toggle.axf",
        ];

        let mut rows_map: HashMap<String, OpExecStat> = HashMap::new();
        let mut found_axf = false;

        for _ in 0..INSTRUCTION_PERF_REPEAT_ROUNDS {
            for axf in axf_candidates {
                if !Path::new(axf).exists() {
                    continue;
                }

                let stats = profile_instruction_exec_on_axf(axf, step_budget_per_axf)
                    .map_err(|e| format!("failed to profile {axf}: {e}"))?;

                found_axf = true;

                for (mnemonic, stat) in stats {
                    let key = mnemonic.to_ascii_lowercase();
                    let entry = rows_map.entry(key).or_default();
                    entry.calls += stat.calls;
                    entry.total_duration += stat.total_duration;
                    if stat.max_duration > entry.max_duration {
                        entry.max_duration = stat.max_duration;
                    }
                }
            }
        }

        if !found_axf {
            return Err("no AXF file found for execution perf test".to_string());
        }

        Ok(rows_map)
    }

    fn get_instruction_exec_stats() -> Result<&'static HashMap<String, OpExecStat>, String> {
        let cache = INSTRUCTION_EXEC_STATS_CACHE.get_or_init(collect_instruction_exec_stats);
        match cache {
            Ok(stats) => Ok(stats),
            Err(err) => Err(err.clone()),
        }
    }

    fn all_instruction_groups_and_defs() -> Vec<(String, Vec<crate::opcodes::opcode::Opcode>)> {
        vec![
            ("adr".to_string(), Instructions::adr::add_adr_def()),
            ("bit_field".to_string(), Instructions::bit_field::add_bit_field_def()),
            ("bitop".to_string(), Instructions::bitop::add_bitop_def()),
            ("branch".to_string(), Instructions::branch::add_branch_def()),
            ("breakpoint".to_string(), Instructions::breakpoint::add_breakpoint_def()),
            ("calculate".to_string(), Instructions::calculate::add_calculate_def()),
            ("cmp".to_string(), Instructions::cmp::add_cmp_def()),
            ("compare_branch".to_string(), Instructions::compare_branch::add_compare_branch_def()),
            ("extend".to_string(), Instructions::extend::add_extend_def()),
            ("hint".to_string(), Instructions::hint::add_Hint_def()),
            ("it".to_string(), Instructions::it::add_it_def()),
            ("ldm".to_string(), Instructions::ldm::add_ldm_def()),
            ("ldr".to_string(), Instructions::ldr::add_ldr_def()),
            ("mov".to_string(), Instructions::mov::add_mov_def()),
            ("movs".to_string(), Instructions::movs::add_movs_def()),
            ("nop".to_string(), Instructions::nop::add_nop_def()),
            ("shift".to_string(), Instructions::shift::addd_shift_def()),
            ("stack".to_string(), Instructions::stack::add_stack_def()),
            ("stm".to_string(), Instructions::stm::add_stm_def()),
            ("str".to_string(), Instructions::str::add_str_def()),
            ("tst".to_string(), Instructions::tst::add_tst_def()),
        ]
    }

    fn build_mnemonic_aliases(definition: &str) -> Vec<String> {
        let def = definition.to_ascii_lowercase();
        let mut aliases = vec![def.clone(), format!("{def}.w")];

        match def.as_str() {
            "add" | "sub" | "and" | "orr" | "bic" | "lsl" | "lsr" | "asr" | "mul" | "mvn" => {
                aliases.push(format!("{def}s"));
            }
            "b" => {
                aliases.extend(
                    ["beq", "bne", "bhs", "blo", "bmi", "bpl", "bvs", "bvc", "bhi", "bls", "bge", "blt", "bgt", "ble"]
                        .iter()
                        .map(|v| v.to_string()),
                );
            }
            "hint" => {
                aliases.extend(
                    ["nop", "yield", "wfe", "wfi", "sev", "sevl"]
                        .iter()
                        .map(|v| v.to_string()),
                );
            }
            _ => {}
        }

        aliases.sort();
        aliases.dedup();
        aliases
    }

    fn build_instruction_definition_perf_rows() -> Result<Vec<InstructionPerfRow>, String> {
        let stats = get_instruction_exec_stats()?;

        let mut rows = Vec::new();
        for (group, defs) in all_instruction_groups_and_defs() {
            for def in defs {
                let aliases = build_mnemonic_aliases(&def.name);
                let mut matched_mnemonics = Vec::new();
                let mut exec_calls = 0u64;
                let mut elapsed = std::time::Duration::ZERO;

                for alias in aliases {
                    if let Some(stat) = stats.get(&alias) {
                        matched_mnemonics.push(alias);
                        exec_calls += stat.calls;
                        elapsed += stat.total_duration;
                    }
                }

                rows.push(InstructionPerfRow {
                    group: group.clone(),
                    definition: def.name.to_ascii_lowercase(),
                    exec_calls,
                    elapsed,
                    matched_mnemonics,
                });
            }
        }

        rows.sort_by(|a, b| b.elapsed.cmp(&a.elapsed));

        write_instruction_definition_exec_perf_csv(&rows, INSTRUCTION_PERF_REPEAT_ROUNDS)
            .map_err(|e| format!("failed to write instruction definition perf csv: {e}"))?;

        Ok(rows)
    }

    fn get_instruction_definition_perf_rows() -> Result<&'static Vec<InstructionPerfRow>, String> {
        let cache = INSTRUCTION_DEFINITION_PERF_CACHE.get_or_init(build_instruction_definition_perf_rows);
        match cache {
            Ok(rows) => Ok(rows),
            Err(err) => Err(err.clone()),
        }
    }

    fn assert_instruction_group_perf(group: &str) {
        let rows = get_instruction_definition_perf_rows()
            .unwrap_or_else(|e| panic!("failed to build instruction definition perf rows: {e}"));

        let group_rows: Vec<&InstructionPerfRow> = rows.iter().filter(|r| r.group == group).collect();
        assert!(
            !group_rows.is_empty(),
            "instruction group {group} has no definition row"
        );

        let group_exec_calls: u64 = group_rows.iter().map(|r| r.exec_calls).sum();
        let group_elapsed: std::time::Duration = group_rows.iter().map(|r| r.elapsed).sum();
        let group_avg_ns_raw = if group_exec_calls == 0 {
            0.0
        } else {
            group_elapsed.as_nanos() as f64 / group_exec_calls as f64
        };

        let timer_overhead = estimate_exec_timer_overhead(group_exec_calls);
        let corrected_elapsed = group_elapsed.saturating_sub(timer_overhead);
        let timer_overhead_avg_ns = if group_exec_calls == 0 {
            0.0
        } else {
            timer_overhead.as_nanos() as f64 / group_exec_calls as f64
        };
        let group_avg_ns_corrected = if group_exec_calls == 0 {
            0.0
        } else {
            corrected_elapsed.as_nanos() as f64 / group_exec_calls as f64
        };

        println!(
            "[perf] instruction_group={group}, definitions={}, exec_calls={}, total_raw={:?}, avg_ns_per_exec_raw={:.2}, timer_overhead_total={:?}, timer_overhead_ns_per_exec={:.2}, total_real={:?}, avg_ns_per_exec_real={:.2}, repeat_rounds={}",
            group_rows.len(),
            group_exec_calls,
            group_elapsed,
            group_avg_ns_raw,
            timer_overhead,
            timer_overhead_avg_ns,
            corrected_elapsed,
            group_avg_ns_corrected,
            INSTRUCTION_PERF_REPEAT_ROUNDS
        );
    }

    fn build_cpu_for_instruction_exec(freq: Arc<AtomicU32>) -> Cpu {
        let gpioa = Gpio::new(0x4001_0800, 0x4001_0BFF);
        let gpiob = Gpio::new(0x4001_0C00, 0x4001_0FFF);
        let gpioc = Gpio::new(0x4001_1000, 0x4001_13FF);
        let afio = Afio::new(0x4001_0000, 0x4001_03FF);
        let usart1 = Uart::new(0x4001_3800, 0x4001_3BFF);
        let rcc = Rcc::new(0x4002_0000, 0x4002_1024, freq.clone());
        let flash_interface = Flash::new(0x4002_2000, 0x4002_201C);
        let tim2 = GeneralTimer::new(0x4000_0000, 0x4000_03FF, 28);
        let tim3 = GeneralTimer::new(0x4000_0400, 0x4000_07FF, 29);
        let tim4 = GeneralTimer::new(0x4000_0800, 0x4000_0BFF, 30);
        let tim5 = GeneralTimer::new(0x4000_0C00, 0x4000_0FFF, 50);

        let mut bus = Bus::new();
        bus.register_peripheral(Box::new(afio));
        bus.register_peripheral(Box::new(gpioa));
        bus.register_peripheral(Box::new(gpiob));
        bus.register_peripheral(Box::new(gpioc));
        bus.register_peripheral(Box::new(usart1));
        bus.register_peripheral(Box::new(flash_interface));
        bus.register_peripheral(Box::new(rcc));
        bus.register_peripheral(Box::new(tim2));
        bus.register_irq_peripheral(0x4000_0000);
        bus.register_peripheral(Box::new(tim3));
        bus.register_irq_peripheral(0x4000_0400);
        bus.register_peripheral(Box::new(tim4));
        bus.register_irq_peripheral(0x4000_0800);
        bus.register_peripheral(Box::new(tim5));
        bus.register_irq_peripheral(0x4000_0C00);

        let mut ppb = Bus::new();
        ppb.register_peripheral(Box::new(SysTick::new(0xE000_E010, 0xE000_E01F)));
        ppb.register_peripheral(Box::new(Nvic::new(0xE000_E100, 0xE000_E4EF)));
        ppb.register_peripheral(Box::new(Scb::new(0xE000_ED00, 0xE000_ED3C)));

        Cpu::new(freq, 1, bus, ppb)
    }

    fn profile_instruction_exec_on_axf(
        axf_path: &str,
        step_budget: u64,
    ) -> Result<Vec<(String, OpExecStat)>, String> {
        let disassembly_output = "target/perf_disassembly_detail.asm";
        let (_result, cs, code_segments, dcw_data, initial_sp, reset_handler_ptr, _reset_handler_addr) =
            disassemble_from_reset_handler(axf_path, disassembly_output).map_err(|e| e.to_string())?;

        let opcode_table = OpcodeTable::new();
        let table = opcode_table.get_table();

        let mut all_insns_storage = Vec::new();
        for (addr, bytes) in &code_segments {
            let insns = cs.disasm_all(bytes, *addr).map_err(|e| e.to_string())?;
            all_insns_storage.push(insns);
        }

        let mut instr_table = Cpu_InstrTable::new();
        for insns in &all_insns_storage {
            for i in insns.iter() {
                let key = i.id().0 as u16;
                if let Some(instructions) = table.get(&key) {
                    for instruction in instructions {
                        if let Some(arm_opcode) = ArmOpcode::new(&cs, &i) {
                            let cpu_instruction = Cpu_Instruction::new(instruction.clone(), arm_opcode);
                            instr_table.add_instruction(cpu_instruction);
                        }
                    }
                }
            }
        }
        instr_table.optimize();

        let freq = Arc::new(AtomicU32::new(8_000_000));
        let mut cpu = build_cpu_for_instruction_exec(freq);
        for (addr, data) in dcw_data {
            cpu.write_mem(addr, data);
        }

        cpu.write_sp(initial_sp);
        let reset_handler = reset_handler_ptr & !1;
        cpu.write_pc(reset_handler);
        cpu.next_pc = reset_handler;
        cpu.write_mem(0x4002_1000, 0x0000_0083);

        for _ in 0..step_budget {
            let current_pc = cpu.next_pc;
            cpu.prefetch_next_pc(current_pc);
            if let Some(ins) = instr_table.get(current_pc) {
                cpu.step(ins, current_pc);
            } else {
                break;
            }
        }

        Ok(cpu.take_exec_op_stats())
    }

    #[test]
    #[ignore = "profiling removed; fast mode only"]
    fn perf_instruction_definition_build() {
        let rows_map = get_instruction_exec_stats()
            .unwrap_or_else(|e| panic!("failed to collect instruction exec stats: {e}"));

        let mut rows: Vec<(String, u64, std::time::Duration)> = rows_map
            .iter()
            .map(|(mnemonic, stat)| (mnemonic.clone(), stat.calls, stat.total_duration))
            .filter(|(_, exec_calls, _)| *exec_calls > 0)
            .collect();

        rows.sort_by(|a, b| b.2.cmp(&a.2));

        println!(
            "[perf] instruction_exec (per mnemonic) sorted by total elapsed (desc), repeat_rounds={}",
            INSTRUCTION_PERF_REPEAT_ROUNDS
        );
        for (mnemonic, exec_calls, elapsed) in &rows {
            let ns_per_op = if *exec_calls == 0 {
                0.0
            } else {
                elapsed.as_nanos() as f64 / *exec_calls as f64
            };
            println!(
                "  - {:<12} total={:?}, {:>8.2} ns/exec(avg), exec_calls={}",
                mnemonic, elapsed, ns_per_op, exec_calls
            );
        }

        write_instruction_exec_perf_csv(&rows, INSTRUCTION_PERF_REPEAT_ROUNDS)
            .expect("failed to write instruction execution perf csv");

        let total_exec_calls: u64 = rows.iter().map(|(_, calls, _)| *calls).sum();
        let elapsed: std::time::Duration = rows.iter().map(|(_, _, d)| *d).sum();
        black_box(total_exec_calls);
        report_perf("instruction_exec_avg", total_exec_calls, elapsed);
        assert!(total_exec_calls > 0);
    }

    #[test]
    #[ignore = "profiling removed; fast mode only"]
    fn perf_instruction_exec_per_definition() {
        let rows = get_instruction_definition_perf_rows()
            .unwrap_or_else(|e| panic!("failed to collect instruction definition perf rows: {e}"));

        let covered = rows.iter().filter(|r| r.exec_calls > 0).count();
        let total = rows.len();
        println!(
            "[perf] instruction definition coverage: covered={covered}, total={total}, repeat_rounds={}",
            INSTRUCTION_PERF_REPEAT_ROUNDS
        );

        assert!(total > 0, "no instruction definitions found");
    }

    macro_rules! instruction_group_perf_test {
        ($test_name:ident, $group:literal) => {
            #[test]
            #[ignore = "profiling removed; fast mode only"]
            fn $test_name() {
                assert_instruction_group_perf($group);
            }
        };
    }

    instruction_group_perf_test!(perf_instruction_group_adr, "adr");
    instruction_group_perf_test!(perf_instruction_group_bit_field, "bit_field");
    instruction_group_perf_test!(perf_instruction_group_bitop, "bitop");
    instruction_group_perf_test!(perf_instruction_group_branch, "branch");
    instruction_group_perf_test!(perf_instruction_group_breakpoint, "breakpoint");
    instruction_group_perf_test!(perf_instruction_group_calculate, "calculate");
    instruction_group_perf_test!(perf_instruction_group_cmp, "cmp");
    instruction_group_perf_test!(perf_instruction_group_compare_branch, "compare_branch");
    instruction_group_perf_test!(perf_instruction_group_extend, "extend");
    instruction_group_perf_test!(perf_instruction_group_hint, "hint");
    instruction_group_perf_test!(perf_instruction_group_it, "it");
    instruction_group_perf_test!(perf_instruction_group_ldm, "ldm");
    instruction_group_perf_test!(perf_instruction_group_ldr, "ldr");
    instruction_group_perf_test!(perf_instruction_group_mov, "mov");
    instruction_group_perf_test!(perf_instruction_group_movs, "movs");
    instruction_group_perf_test!(perf_instruction_group_nop, "nop");
    instruction_group_perf_test!(perf_instruction_group_shift, "shift");
    instruction_group_perf_test!(perf_instruction_group_stack, "stack");
    instruction_group_perf_test!(perf_instruction_group_stm, "stm");
    instruction_group_perf_test!(perf_instruction_group_str, "str");
    instruction_group_perf_test!(perf_instruction_group_tst, "tst");

    #[test]
    fn perf_peripheral_io_and_tick() {
        let loops = 200_000u64;

        let mut gpio = Gpio::new(0x4001_1000, 0x4001_13FF);
        let mut flash = Flash::new(0x4002_2000, 0x4002_201C);
        let mut systick = SysTick::new(0xE000_E010, 0xE000_E01F);
        let mut scb = Scb::new(0xE000_ED00, 0xE000_ED3C);
        let freq = Arc::new(AtomicU32::new(8_000_000));
        let mut rcc = Rcc::new(0x4002_0000, 0x4002_1024, freq.clone());

        systick.write(0xE000_E014, 1000);
        systick.write(0xE000_E010, 1);

        let start = Instant::now();
        for i in 0..loops {
            let v = i as u32;

            gpio.write(0x4001_100C, v);
            black_box(gpio.read(0x4001_100C));
            gpio.tick();

            flash.write(0x4002_2004, 0x4567_0123);
            flash.write(0x4002_2004, 0xCDEF_89AB);
            flash.write(0x4002_2010, v);
            black_box(flash.read(0x4002_2010));
            flash.tick();

            rcc.write(0x4002_0000, (1 << 0) | (((v & 1) as u32) << 16));
            rcc.write(0x4002_0004, v & 0x3);
            black_box(rcc.read(0x4002_0004));
            rcc.tick();

            scb.write(0xE000_ED10, v);
            black_box(scb.read(0xE000_ED10));
            scb.tick();

            systick.tick();
            black_box(systick.read(0xE000_E010));
        }

        let elapsed = start.elapsed();
        let iterations = loops * 5;
        black_box(freq.load(Ordering::Relaxed));
        report_perf("peripheral_io_tick", iterations, elapsed);
        assert!(elapsed.as_nanos() > 0);
    }

    #[test]
    fn perf_cpu_peripheral_step() {
        let loops = 1_000_000u64;
        let freq = Arc::new(AtomicU32::new(8_000_000));

        let mut bus = Bus::new();
        bus.register_peripheral(Box::new(Gpio::new(0x4001_1000, 0x4001_13FF)));
        bus.register_peripheral(Box::new(Flash::new(0x4002_2000, 0x4002_201C)));
        bus.register_peripheral(Box::new(Rcc::new(0x4002_0000, 0x4002_1024, freq.clone())));

        let mut ppb = Bus::new();
        ppb.register_peripheral(Box::new(SysTick::new(0xE000_E010, 0xE000_E01F)));
        ppb.register_peripheral(Box::new(Nvic::new(0xE000_E100, 0xE000_E4EF)));
        ppb.register_peripheral(Box::new(Scb::new(0xE000_ED00, 0xE000_ED3C)));

        let mut cpu = Cpu::new(freq.clone(), 1, bus, ppb);

        let start = Instant::now();
        for _ in 0..loops {
            cpu.peripheral_step();
        }
        let elapsed = start.elapsed();

        black_box(freq.load(Ordering::Relaxed));
        report_perf("cpu_peripheral_step", loops, elapsed);
        assert!(elapsed.as_nanos() > 0);
    }

    #[test]
    fn perf_cpu_read_mem_write_mem() {
        let loops = 1_000_000u64;
        let mut cpu = Cpu::new(
            Arc::new(AtomicU32::new(8_000_000)),
            1,
            Bus::new(),
            Bus::new(),
        );
        let base = 0x2000_0000u32;

        let write_start = Instant::now();
        for i in 0..loops {
            let addr = base.wrapping_add(((i as u32) & 0x3FFC) as u32);
            cpu.write_mem(addr, i as u32);
        }
        let write_elapsed = write_start.elapsed();

        let read_start = Instant::now();
        let mut checksum = 0u32;
        for i in 0..loops {
            let addr = base.wrapping_add(((i as u32) & 0x3FFC) as u32);
            checksum ^= cpu.read_mem(addr);
        }
        let read_elapsed = read_start.elapsed();

        report_perf("cpu_write_mem_ram", loops, write_elapsed);
        report_perf("cpu_read_mem_ram", loops, read_elapsed);

        black_box(checksum);
        assert!(write_elapsed.as_nanos() > 0);
        assert!(read_elapsed.as_nanos() > 0);
    }
}
