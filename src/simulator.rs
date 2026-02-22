// mod cpu;
use crate::context::CpuContext;
use crate::cpu::Cpu;
use crate::opcodes::instruction::{Cpu_InstrTable, Cpu_Instruction};
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

struct Monitor {
    start: Instant,
    count: u64,
}

impl Monitor {
    fn new() -> Self {
        Self {
            start: Instant::now(),
            count: 0,
        }
    }

    fn set_start(&mut self, start: Instant) {
        self.start = start;
    }

    fn update(&mut self) {
        self.count += 1;
        if self.count % 1000 == 0 {
            let elapsed = self.start.elapsed();
            if elapsed >= Duration::from_secs(1) {
                let freq = self.count as f64 / elapsed.as_secs_f64();
                println!("Current Execution Frequency: {:.6} MHz", freq / 1_000_000.0);
                self.start = Instant::now();
                self.count = 0;
            }
        }
    }
}

pub struct Simulator {
    cpu: Cpu,
    now_ns: u64,
}

impl Simulator {
    pub fn new(cpu: Cpu) -> Self {
        Self { cpu, now_ns: 0 }
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

    fn tick<'a>(&mut self, ins: &Cpu_Instruction<'a>, current_pc: u32) {
        // println!("tick once");
        self.cpu.step(&ins, current_pc);
        self.cpu.peripheral_step();
    }

    pub fn sim_loop<'a>(&mut self, ins_table: Cpu_InstrTable<'a>) {
        // let nanos_per_tick = 1_000_000_000 / (self.cpu.frequency * self.cpu.machine_cycle as u32);
        // let tick_duration = Duration::from_nanos(nanos_per_tick as u64);

        let mut monitor = Monitor::new();

        println!("--- Benchmarking Fetch Speed (100M iters) ---");
        let bench_start = Instant::now();
        let mut bench_pc = self.cpu.next_pc;
        // 预热并测试纯 Fetch 性能
        for _ in 0..100_000_000 { // 1亿次
             // 模拟简单的 PC 递增，确保不下越界
             bench_pc = if bench_pc > 0x0800_1000 { 0x0800_0000 } else { bench_pc + 2 };
             let _ = ins_table.get(bench_pc);
             std::hint::black_box(&bench_pc); // 防止编译器过度优化
        }
        let bench_duration = bench_start.elapsed();
        println!(
            "Fetch Benchmark: {:.9} ns / op",
            (bench_duration.as_nanos() as f64) / 100_000_000.0
        );
        println!("--- End Benchmark ---");

        let mut fetch_count = 0;
        let mut fetch_duration = Duration::new(0, 0);

        let start_time = Instant::now();
        monitor.set_start(start_time);
        loop {
            let frequency = self.cpu.frequency.load(Ordering::Relaxed);
            // println!("actual freq: {} MHz", frequency as f64 / 1_000_000.0);
            let nanos_per_tick = 1_000_000_000 / (frequency * self.cpu.machine_cycle as u32);
            let tick_duration = Duration::from_nanos(nanos_per_tick as u64);
            let loop_start = Instant::now();
            self.cpu.current_pc = self.cpu.next_pc;
            let mut current_pc = self.cpu.current_pc;

            self.cpu.write_pc(current_pc.wrapping_add(4)); //预取下一条指令地址
            // println!(
            //     "Current ins: 0x{:X}  Pc: 0x{:X}",
            //     current_pc,
            //     self.cpu.read_pc()
            // );
            // let test_start = Instant::now(); // 移除高开销的计时
            match ins_table.get(current_pc) {
                Some(ins) => {
                    // fetch_duration += test_start.elapsed(); // 移除高开销的计时
                    // fetch_count += 1;
                    // if fetch_count >= 100 { ... }
                    self.tick(ins, current_pc);
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

            let elapsed = loop_start.elapsed();
            // println!("Loop elapsed: {:.6} ms  tick_duration: {:.6} ms", elapsed.as_secs_f64() * 1000.0, tick_duration.as_secs_f64() * 1000.0);
            if elapsed < tick_duration {
                thread::sleep(tick_duration - elapsed);
            }
        }
    }
}
