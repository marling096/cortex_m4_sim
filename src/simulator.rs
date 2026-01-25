mod cpu;
use crate::cpu::Cpu;
use std::time::{Duration, Instant};

pub struct Simulator {
    cpu: Cpu,
    now_ns: u64,
}

impl Simulator {
    pub fn new(cpu: Cpu) -> Self {
        Self { cpu, now_ns: 0 }
    }

    fn tick(&mut self) {
        let nanos_per_tick = 1_000_000_000 / (self.cpu.frequency * self.cpu.machine_cycle);

        (self.cpu.step)(&mut self.cpu);
        self.now_ns += nanos_per_tick;
    }

    pub fn sim_loop(&mut self) {
        let start_time = Instant::now();
        let nanos_per_tick = 1_000_000_000 / (self.cpu.frequency * self.cpu.machine_cycle);
        let tick_duration = Duration::from_nanos(nanos_per_tick);

        loop {
            let loop_start = Instant::now();

            self.tick();

            let elapsed = loop_start.elapsed();
            if elapsed < tick_duration {
                thread::sleep(tick_duration - elapsed);
            }
        }
    }
}
