// mod cpu;
use crate::context::CpuContext;
use crate::cpu::Cpu;
use crate::opcodes::instruction::{Cpu_InstrTable, Cpu_Instruction};
use std::thread;
use std::time::{Duration, Instant};

pub struct Simulator {
    cpu: Cpu,
    now_ns: u64,
}

impl Simulator {
    pub fn new(cpu: Cpu) -> Self {
        Self { cpu, now_ns: 0 }
    }

    fn tick<'a>(&mut self, ins: &Cpu_Instruction<'a>) {
        let nanos_per_tick = 1_000_000_000 / (self.cpu.frequency * self.cpu.machine_cycle as u32);

        self.cpu.step(&ins);
        self.now_ns += nanos_per_tick as u64;
    }

    pub fn sim_loop<'a>(&mut self, ins_table: Cpu_InstrTable<'a>) {
        let start_time = Instant::now();
        let nanos_per_tick = 1_000_000_000 / (self.cpu.frequency * self.cpu.machine_cycle as u32);
        let tick_duration = Duration::from_nanos(nanos_per_tick as u64);

        loop {
            let loop_start = Instant::now();

            let pc = self.cpu.read_pc_counter() as usize;
            match ins_table.table.get(pc) {
                Some(ins) => self.tick(ins),
                None => {
                    eprintln!("Error: PC {} is out of bounds. Simulation stopped.", pc);
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
