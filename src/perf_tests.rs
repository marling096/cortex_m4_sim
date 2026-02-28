#[cfg(test)]
mod tests {
    use std::fs;
    use std::hint::black_box;
    use std::path::Path;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Instant;

    use crate::context::CpuContext;
    use crate::cpu::Cpu;
    use crate::opcodes::Instructions;
    use crate::peripheral::bus::Bus;
    use crate::peripheral::peripheral::Peripheral;
    use crate::peripheral::flash::Flash;
    use crate::peripheral::gpio::Gpio;
    use crate::peripheral::nvic::Nvic;
    use crate::peripheral::rcc::Rcc;
    use crate::peripheral::scb::Scb;
    use crate::peripheral::systick::SysTick;

    fn report_perf(name: &str, iterations: u64, elapsed: std::time::Duration) {
        let ns_per_op = elapsed.as_nanos() as f64 / iterations as f64;
        let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();
        println!(
            "[perf] {name}: iter={iterations}, total={:?}, {:.2} ns/op, {:.2} ops/s",
            elapsed, ns_per_op, ops_per_sec
        );
    }

    fn write_instruction_perf_csv(
        rows: &[(&'static str, u64, std::time::Duration, usize)],
    ) -> std::io::Result<()> {
        let dir = Path::new("perf_reports");
        fs::create_dir_all(dir)?;
        let path = dir.join("instruction_def_build_sorted.csv");

        let mut csv = String::from(
            "rank,instruction,total_ns,total_ms,ns_per_op,iterations,defs_built\n",
        );

        for (index, (name, iterations, elapsed, defs)) in rows.iter().enumerate() {
            let total_ns = elapsed.as_nanos() as f64;
            let total_ms = elapsed.as_secs_f64() * 1000.0;
            let ns_per_op = total_ns / (*iterations as f64);
            csv.push_str(&format!(
                "{},{},{:.0},{:.6},{:.4},{},{}\n",
                index + 1,
                name,
                total_ns,
                total_ms,
                ns_per_op,
                iterations,
                defs
            ));
        }

        fs::write(&path, csv)?;
        println!("[perf] csv report written: {}", path.display());
        Ok(())
    }

    fn measure_builder(
        name: &'static str,
        loops: u64,
        builder: fn() -> Vec<crate::opcodes::opcode::Opcode>,
    ) -> (&'static str, u64, std::time::Duration, usize) {
        let start = Instant::now();
        let mut defs = 0usize;
        for _ in 0..loops {
            defs += builder().len();
        }
        (name, loops, start.elapsed(), defs)
    }

    #[test]
    fn perf_instruction_definition_build() {
        let loops = 20_000u64;
        let mut rows = vec![
            measure_builder("adr", loops, Instructions::adr::add_adr_def),
            measure_builder("bitop", loops, Instructions::bitop::add_bitop_def),
            measure_builder("branch", loops, Instructions::branch::add_branch_def),
            measure_builder("breakpoint", loops, Instructions::breakpoint::add_breakpoint_def),
            measure_builder("calculate", loops, Instructions::calculate::add_calculate_def),
            measure_builder("cmp", loops, Instructions::cmp::add_cmp_def),
            measure_builder(
                "compare_branch",
                loops,
                Instructions::compare_branch::add_compare_branch_def,
            ),
            measure_builder("hint", loops, Instructions::hint::add_Hint_def),
            measure_builder("it", loops, Instructions::it::add_it_def),
            measure_builder("ldm", loops, Instructions::ldm::add_ldm_def),
            measure_builder("ldr", loops, Instructions::ldr::add_ldr_def),
            measure_builder("mov", loops, Instructions::mov::add_mov_def),
            measure_builder("movs", loops, Instructions::movs::add_movs_def),
            measure_builder("nop", loops, Instructions::nop::add_nop_def),
            measure_builder("shift", loops, Instructions::shift::addd_shift_def),
            measure_builder("stack", loops, Instructions::stack::add_stack_def),
            measure_builder("stm", loops, Instructions::stm::add_stm_def),
            measure_builder("str", loops, Instructions::str::add_str_def),
            measure_builder("tst", loops, Instructions::tst::add_tst_def),
        ];

        rows.sort_by(|a, b| b.2.cmp(&a.2));

        println!("[perf] instruction_def_build sorted by total elapsed (desc)");
        for (name, iter, elapsed, defs) in &rows {
            let ns_per_op = elapsed.as_nanos() as f64 / *iter as f64;
            println!(
                "  - {:<16} total={:?}, {:>8.2} ns/op, defs={}",
                name, elapsed, ns_per_op, defs
            );
        }

        write_instruction_perf_csv(&rows).expect("failed to write instruction perf csv");

        let total_defs: usize = rows.iter().map(|(_, _, _, defs)| *defs).sum();
        let elapsed: std::time::Duration = rows.iter().map(|(_, _, d, _)| *d).sum();
        let iterations = loops * (rows.len() as u64);
        black_box(total_defs);
        report_perf("instruction_def_build", iterations, elapsed);
        assert!(total_defs > 0);
    }

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
        cpu.set_profiling_enabled(false);
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

        cpu.set_profiling_enabled(true);
        let profile_loops = loops / 10;

        let profile_write_start = Instant::now();
        for i in 0..profile_loops {
            let addr = base.wrapping_add((i as u32) & 0x3FFC);
            cpu.write_mem(addr, i as u32);
        }
        let profile_write_elapsed = profile_write_start.elapsed();

        let profile_read_start = Instant::now();
        for i in 0..profile_loops {
            let addr = base.wrapping_add((i as u32) & 0x3FFC);
            checksum ^= cpu.read_mem(addr);
        }
        let profile_read_elapsed = profile_read_start.elapsed();

        report_perf("cpu_write_mem_ram_profiled", profile_loops, profile_write_elapsed);
        report_perf("cpu_read_mem_ram_profiled", profile_loops, profile_read_elapsed);

        let profile = cpu.take_exec_profile();
        let write_avg = if profile.mem_write_count > 0 {
            profile.mem_write_duration.as_nanos() as f64 / profile.mem_write_count as f64
        } else {
            0.0
        };
        let read_avg = if profile.mem_read_count > 0 {
            profile.mem_read_duration.as_nanos() as f64 / profile.mem_read_count as f64
        } else {
            0.0
        };

        println!(
            "[perf] cpu_mem_profile: reads={}, read_total={:?}, read_avg={:.2} ns/op; writes={}, write_total={:?}, write_avg={:.2} ns/op",
            profile.mem_read_count,
            profile.mem_read_duration,
            read_avg,
            profile.mem_write_count,
            profile.mem_write_duration,
            write_avg,
        );

        black_box(checksum);
        assert_eq!(profile.mem_read_count, profile_loops);
        assert_eq!(profile.mem_write_count, profile_loops);
    }
}
