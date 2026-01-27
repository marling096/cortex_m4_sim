use crate::context::CpuContext;
use crate::opcodes::instruction::Cpu_Instruction;
pub struct Cpu {
    pub frequency: u32,
    pub machine_cycle: u8,
    pub Cycles: u64,

    pub Cpu_pipeline: Cpu_pipeline,

    pub flash: Vec<u8>, // 模拟 Flash (例如 512KB)
    pub ram: Vec<u8>,   // 模拟 SRAM (例如 128KB)
    pub registers: Registers,

    pub pc_coutnter: u32, // 用于跟踪程序计数器的变化,计数，并非地址
}
struct Registers {
    reg: [u32; 16], // R0 - R15    // 应用程序状态寄存器 (xPSR 的一部分)
    apsr: u32,
    is_msp: bool, // 当前使用的是否是 MSP
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
}

impl CpuContext for Cpu {
    fn read_mem(&self, addr: u32) -> u32 {
        // Cortex-M 默认是小端序 (Little-endian)
        // 从 addr 到 addr+3 依次读取并组合
        u32::from_le_bytes([
            self.read_byte(addr),
            self.read_byte(addr + 1),
            self.read_byte(addr + 2),
            self.read_byte(addr + 3),
        ])
    }

    fn write_mem(&mut self, addr: u32, val: u32) {
        let bytes = val.to_le_bytes();
        // 依次写入 4 个字节
        for i in 0..4 {
            self.write_byte(addr + i, bytes[i as usize]);
        }
    }

    fn read_reg(&self, r: u32) -> u32 {
        self.registers.reg[r as usize]
    }

    fn write_reg(&mut self, r: u32, v: u32) {
        self.registers.reg[r as usize] = v;
    }
    fn read_gpr(&self, r: u32) -> u32 {
        self.registers.reg[r as usize]
    }

    fn write_gpr(&mut self, r: u32, v: u32) {
        self.registers.reg[r as usize] = v;
    }

    fn read_msp(&self, _r: u32) -> u32 {
        if self.registers.is_msp {
            self.registers.reg[13]
        } else {
            0 // TODO: handle banked MSP
        }
    }
    fn write_msp(&mut self, v: u32) {
        if self.registers.is_msp {
            self.registers.reg[13] = v;
        } else {
            // TODO: handle banked MSP
        }
    }

    fn read_psp(&self, _r: u32) -> u32 {
        if !self.registers.is_msp {
            self.registers.reg[13]
        } else {
            0 // TODO: handle banked PSP
        }
    }

    fn write_psp(&mut self, v: u32) {
        if !self.registers.is_msp {
            self.registers.reg[13] = v;
        } else {
            // TODO: handle banked PSP
        }
    }

    fn read_lr(&self, _r: u32) -> u32 {
        self.registers.reg[14]
    }

    fn write_lr(&mut self, v: u32) {
        self.registers.reg[14] = v;
    }

    fn read_pc(&self) -> u32 {
        self.registers.reg[15]
    }

    fn write_pc(&mut self, pc: u32) {
        self.registers.reg[15] = pc;
        self.write_pc_counter(self.read_pc_counter() + 1); // 同步更新外部pc计数器
    }

    fn read_apsr(&self) -> u32 {
        self.registers.apsr
    }
    fn write_apsr(&mut self, v: u32) {
        self.registers.apsr = v;
    }

    fn read_pc_counter(&self) -> u32 {
        self.pc_coutnter
    }

    fn write_pc_counter(&mut self, v: u32) {
        self.pc_coutnter = v; //更新外部pc值
    }
}

impl Cpu {
    pub fn new(frequency: u32, machine_cycle: u8) -> Cpu {
        Cpu {
            frequency,
            machine_cycle,
            Cycles: 0,
            Cpu_pipeline: Cpu_pipeline::new(),
            flash: vec![0; 512 * 1024], // 512KB Flash
            ram: vec![0; 128 * 1024],   // 128
            registers: Registers {
                reg: [0; 16],
                apsr: 0,
                is_msp: true,
            },
            pc_coutnter: 0,
        }
    }
    fn read_byte(&self, addr: u32) -> u8 {
        match addr {
            // 1. 别名区域 (Aliased region)
            // 将 0x0000_0000 起始的访问重定向到 Flash
            0x0000_0000..=0x0007_FFFF => self.flash[addr as usize],

            // 2. 物理 Flash 区域 (Physical Flash)
            // 假设 Flash 基地址是 0x0800_0000
            0x0800_0000..=0x0807_FFFF => {
                let offset = (addr - 0x0800_0000) as usize;
                self.flash[offset]
            }
            // RAM 区域 (例如 0x2000_0000 - 0x2001_FFFF)
            0x2000_0000..=0x2001_FFFF => {
                let offset = (addr - 0x2000_0000) as usize;
                self.ram[offset]
            }

            // 内部私有外设总线 (PPB, 如 NVIC, SysTick)
            0x40000000..=0x500607FF => {
                // 这里可以后续实现外设寄存器
                0
            }
            0xE000_0000..=0xE00F_FFFF => {
                // 这里可以后续实现System 控制寄存器寄存器
                0
            }
            _ => {
                // 触发仿真异常：访问了未映射的地址
                panic!("Memory Read Error: Unmapped address 0x{:08X}", addr);
            }
        }
    }

    /// 核心写入函数：处理不同区域的写入权限
    fn write_byte(&mut self, addr: u32, val: u8) {
        match addr {
            // RAM 区域
            0x2000_0000..=0x2001_FFFF => {
                let offset = (addr - 0x2000_0000) as usize;
                self.ram[offset] = val;
            }
            // Flash 区域：通常只读，直接写入可能在仿真中报错或模拟 Flash 控制器
            0x0000_0000..=0x0007_FFFF => {
                eprintln!("Warning: Attempted to write to Flash at 0x{:08X}", addr);
            }
            0x40000000..=0x500607FF => {
                // 这里可以后续实现外设寄存器写入
            }

            0xE000_0000..=0xE00F_FFFF => {
                // 这里可以后续实现System 控制寄存器寄存器写入
            }
            _ => {
                panic!("Memory Write Error: Unmapped address 0x{:08X}", addr);
            }
        }
    }
    // changed: make step take &mut self and avoid borrow conflicts
    pub fn step<'a>(&mut self, ins: &Cpu_Instruction<'a>) {
        if self.Cpu_pipeline.remain_cycles > 0 {
            self.Cpu_pipeline.remain_cycles -= 1;
            return;
        }

        let phase = self.Cpu_pipeline.phase;

        match phase {
            Phase::Fetch => {
                //skip fetch phase
                // // immutable borrows end at the end of the call, so subsequent mutations are allowed
                // let instruction = self.Cpu_pipeline.fetch(&*self);
                self.Cpu_pipeline.phase = Phase::Decode;
                self.Cpu_pipeline.remain_cycles = ins.op.cycles.fetch_cycles.saturating_sub(1);
            }
            Phase::Decode => {
                // let instruction = self.Cpu_pipeline.fetch(&*self); // Placeholder
                // self.Cpu_pipeline.decode(Opcode);
                self.Cpu_pipeline.phase = Phase::Execute;
                self.Cpu_pipeline.remain_cycles = ins.op.cycles.decode_cycles.saturating_sub(1);
            }
            Phase::Execute => {
                self.Cpu_pipeline.phase = Phase::Fetch;
                ins.op.exec.execute(&mut *self, &ins.data);
                self.update_pc(&ins);
                self.Cpu_pipeline.remain_cycles = ins.op.cycles.execute_cycles.saturating_sub(1);
            }
        }
    }

    fn update_pc<'a>(&mut self, ins: &Cpu_Instruction<'a>) {
        self.write_pc(self.read_pc() + ins.op.length as u32);
    }
}
