// mod cpu;
use crate::context::CpuContext;
use crate::cpu::Cpu;
use crate::opcodes::instruction::{Cpu_InstrTable, Cpu_Instruction};
use std::collections::BTreeMap;
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

    pub fn sim_reset<'a>(&mut self, dcw_data: BTreeMap<u32, u32> , initial_sp: u32, reset_handler_ptr: u32) {
        for (addr, data) in dcw_data.iter() {
            self.cpu.write_mem(*addr, *data);
        }
        self.cpu.write_sp(initial_sp);
        let reset_handler = reset_handler_ptr & !1; //确保最低位为0，表示Thumb指令集
        print!("Reset Handler at: 0x{:08X}\n", reset_handler);
        self.cpu.write_pc(reset_handler);
        self.cpu.next_pc = reset_handler;
        self.cpu.write_mem(0x40021000, 0x0000_0083);//rcc.cr初始值
    }

    fn tick<'a>(&mut self, ins: &Cpu_Instruction<'a>, current_pc: u32) {
        let nanos_per_tick = 1_000_000_000 / (self.cpu.frequency * self.cpu.machine_cycle as u32);

        self.cpu.step(&ins, current_pc);
        self.cpu.peripheral_step();
        self.now_ns += nanos_per_tick as u64;
    }

    pub fn sim_loop<'a>(&mut self, ins_table: Cpu_InstrTable<'a>) {
        let start_time = Instant::now();
        let nanos_per_tick = 1_000_000_000 / (self.cpu.frequency * self.cpu.machine_cycle as u32);
        let tick_duration = Duration::from_nanos(nanos_per_tick as u64);

        // self.cpu
        //     .reset_handler(*ins_table.table.first_key_value().unwrap().0);

        loop {
            let loop_start = Instant::now();

            //if pc == 0 pc = 0x08000000
            self.cpu.current_pc = self.cpu.next_pc;
            let mut current_pc = self.cpu.current_pc;

            self.cpu.write_pc(current_pc.wrapping_add(4)); //预取下一条指令地址
            // println!(
            //     "Current ins: 0x{:X}  Pc: 0x{:X}",
            //     current_pc,
            //     self.cpu.read_pc()
            // );
            match ins_table.table.get(&current_pc) {
                Some(ins) => self.tick(ins, current_pc),
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
