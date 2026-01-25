use crate::context::CpuContext;

pub struct Cpu {
    Frequency: u32,
    Machine_Cycle: u8,
    Cycles: u64,

    Cpu_pipeline: Cpu_pipeline,

    flash: Vec<u8>, // 模拟 Flash (例如 512KB)
    sram: Vec<u8>,  // 模拟 SRAM (例如 128KB)
    registers: Registers,
}
struct Registers {
    gpr: [u32; 13], // R0 - R12
    msp: u32,       // Main Stack Pointer
    psp: u32,       // Process Stack Pointer
    lr: u32,        // R14
    pc: u32,        // R15
    apsr: u32,      // 应用程序状态寄存器 (xPSR 的一部分)
    is_msp: bool,   // 当前使用的是否是 MSP
}

struct Cpu_pipeline {
    remain_cycles: u32,
    phase: Phase,
}

#[derive(Copy, Clone)]
enum Phase {
    Fetch,
    Decode,
    Execute,
}

impl Cpu_pipeline {
    fn new() -> Cpu_pipeline {
        Cpu_pipeline {
            remain_cycles: 0,
            phase: Phase::Fetch,
        }
    }

    fn fetch(&self, cpu: &Cpu) -> u32 {
        // Simulate fetching an instruction (placeholder implementation)
        0
    }

    // fn decode<D>(&self, instruction: Opcode<D>) {
    //     // Simulate decoding an instruction (placeholder implementation)
    // }

    // changed: make execute an associated function that only takes &mut Cpu
    fn execute(cpu: &mut Cpu) {
        // Simulate executing an instruction (placeholder implementation)
        cpu.Cycles += 1;
    }
}

impl CpuContext for Cpu {
    fn read_reg(&self, r: u32) -> u32 {
        // Placeholder implementation
        0
    }

    fn write_reg(&mut self, r: u32, v: u32) {
        // Placeholder implementation
    }

    fn read_mem(&self, addr: u32) -> u32 {
        // Placeholder implementation
        0
    }

    fn write_mem(&mut self, addr: u32, v: u32) {
        // Placeholder implementation
    }

    fn read_operand_value(&mut self, _imm: bool, _addr: u32) -> (u32, u8) {
        // Placeholder implementation: no shift -> carry = 0
        (0, 0)
    }

    fn read_gpr(&self, r: u32) -> u32 {
        self.registers.gpr[r as usize]
    }

    fn write_gpr(&mut self, r: u32, v: u32) {
        self.registers.gpr[r as usize] = v;
    }

    fn read_msp(&self, r: u32) -> u32 {
        self.registers.msp
    }
    fn write_msp(&mut self, v: u32) {
        self.registers.msp = v;
    }

    fn read_psp(&self, r: u32) -> u32 {
        self.registers.psp
    }

    fn write_psp(&mut self, v: u32) {
        self.registers.psp = v;
    }

    fn read_lr(&self, r: u32) -> u32 {
        self.registers.lr
    }

    fn write_lr(&mut self, v: u32) {
        self.registers.lr = v;
    }

    fn read_pc(&self) -> u32 {
        self.registers.pc as u32
    }

    fn write_pc(&mut self, pc: u32) {
        self.registers.pc = pc as u32;
    }

    fn read_apsr(&self) -> u32 {
        self.registers.apsr
    }
    fn write_apsr(&mut self, v: u32) {
        self.registers.apsr = v;
    }
}

impl Cpu {
    fn new(frequency: u32, machine_cycle: u8) -> Cpu {
        Cpu {
            Frequency: frequency,
            Machine_Cycle: machine_cycle,
            Cycles: 0,
            Cpu_pipeline: Cpu_pipeline::new(),
            flash: vec![0; 512 * 1024], // 512KB Flash
            sram: vec![0; 128 * 1024],  // 128
            registers: Registers {
                gpr: [0; 13],
                msp: 0,
                psp: 0,
                lr: 0,
                pc: 0,
                apsr: 0,
                is_msp: true,
            },
        }
    }

    // changed: make step take &mut self and avoid borrow conflicts
    fn step(&mut self) {
        if self.Cpu_pipeline.remain_cycles > 0 {
            self.Cpu_pipeline.remain_cycles -= 1;
            return;
        }

        let phase = self.Cpu_pipeline.phase;

        match phase {
            Phase::Fetch => { //skip fetch phase
                // // immutable borrows end at the end of the call, so subsequent mutations are allowed
                // let instruction = self.Cpu_pipeline.fetch(&*self);
                // self.Cpu_pipeline.phase = Phase::Decode;
                // // self.Cpu_pipeline.remain_cycles = 1;
            }
            Phase::Decode => {
                // let instruction = self.Cpu_pipeline.fetch(&*self); // Placeholder
                // self.Cpu_pipeline.decode(Opcode);
                // self.Cpu_pipeline.phase = Phase::Execute;
                // // self.Cpu_pipeline.remain_cycles = 2;
            }
            Phase::Execute => {
                Cpu_pipeline::execute(self);
                self.Cpu_pipeline.phase = Phase::Fetch;
                // self.Cpu_pipeline.remain_cycles = 0;
            }
        }
    }
}
