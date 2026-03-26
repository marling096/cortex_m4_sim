#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Instant;

    use crate::context::CpuContext;
    use crate::cpu::Cpu;
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
