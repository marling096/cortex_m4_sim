use std::cell::RefCell;
use std::vec;

use crate::context::CpuContext;
use crate::opcodes::instruction::Cpu_Instruction;
use crate::peripheral::bus::Bus;

pub struct Cpu {
    pub frequency: u32,
    pub machine_cycle: u8,
    pub Cycles: u64,

    pub Cpu_pipeline: Cpu_pipeline,

    pub flash: Vec<u8>, // 模拟 Flash (例如 512KB)
    pub ram: Vec<u8>,   // 模拟 SRAM (例如 128KB)
    pub registers: Registers,

    pub current_pc: u32, // 当前 指令地址

    pub next_pc: u32, // 预取的下一条指令地址

    pub peripheral_bus: RefCell<Bus>,

    pub ppb: RefCell<Bus>,
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

    fn read_sp(&self) -> u32 {
        self.registers.reg[13]
    }

    fn write_sp(&mut self, v: u32) {
        self.registers.reg[13] = v;
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
    }

    fn read_apsr(&self) -> u32 {
        self.registers.apsr
    }
    fn write_apsr(&mut self, v: u32) {
        self.registers.apsr = v;
    }
}

impl Cpu {
    pub fn new(frequency: u32, machine_cycle: u8, peripheral_bus: Bus, ppb: Bus) -> Cpu {
        Cpu {
            frequency,
            machine_cycle,
            Cycles: 0,
            Cpu_pipeline: Cpu_pipeline::new(),
            flash: vec![0; 512 * 1024], // 512KB Flash
            ram: vec![0; 128 * 1024],   // 128KB RAM
            registers: Registers {
                reg: [0; 16],
                apsr: 0,
                is_msp: true,
            },
            current_pc: 0,
            next_pc: 0,
            peripheral_bus: RefCell::new(peripheral_bus),
            ppb: RefCell::new(ppb),
        }
    }

    pub fn reset_handler(&mut self, reset_vector: u32) {
        // 复位时，PC 设置为复位向量地址
        self.write_pc(reset_vector);
        self.next_pc = reset_vector;
        // 其他寄存器初始化
        self.registers.reg = [0; 16];
        self.registers.apsr = 0;
        self.registers.is_msp = true;
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
            // RAM 区域 (例如 0x2000_0000 - 0x3FFFFFFF)
            0x2000_0000..=0x3FFFFFFF => {
                let offset = (addr - 0x2000_0000) as usize;
                self.ram[offset]
            }

            0x40000000..=0x5FFFFFFF => self.peripheral_bus.borrow_mut().read8(addr),
            0xE000_0000..=0xE00F_FFFF => {
                // 这里可以后续实现System 控制寄存器寄存器
                self.ppb.borrow_mut().read8(addr)
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
            0x2000_0000..=0x3FFFFFFF => {
                let offset = (addr - 0x2000_0000) as usize;
                self.ram[offset] = val;
            }
            // Flash 区域：通常只读，直接写入可能在仿真中报错或模拟 Flash 控制器
            0x0000_0000..=0x0007_FFFF => {
                // eprintln!("Warning: Attempted to write to Flash at 0x{:08X}", addr);
            }
            0x0800_0000..=0x0807_FFFF => {
                // eprintln!("Warning: Attempted to write to Flash at 0x{:08X}", addr);
                self.flash[(addr - 0x0800_0000) as usize] = val;
            }
            0x40000000..=0x5FFFFFFF => {
                // 这里可以后续实现外设寄存器写入
                print!("Peripheral Write at 0x{:08X} Value: 0x{:02X}\n", addr, val);
                self.peripheral_bus.borrow_mut().write8(addr, val);
            }

            0xE000_0000..=0xE00F_FFFF => {
                self.ppb.borrow_mut().write8(addr, val);
            }
            _ => {
                panic!("Memory Write Error: Unmapped address 0x{:08X}", addr);
            }
        }
    }

    // changed: make step take &mut self and avoid borrow conflicts
    pub fn step<'a>(&mut self, ins: &Cpu_Instruction<'a>, current_pc: u32) {
        if self.Cpu_pipeline.remain_cycles > 0 {
            self.Cpu_pipeline.remain_cycles -= 1;
            return;
        }
        self.Cpu_pipeline.phase = Phase::Execute;
        let phase = self.Cpu_pipeline.phase;

        match phase {
            Phase::Fetch => {
                //skip fetch phase
                self.Cpu_pipeline.phase = Phase::Execute;

                self.Cpu_pipeline.remain_cycles = ins.op.cycles.fetch_cycles.saturating_sub(1);
            }
            Phase::Decode => {
                // let instruction = self.Cpu_pipeline.fetch(&*self); // Placeholder
                // self.Cpu_pipeline.decode(Opcode);
                self.Cpu_pipeline.phase = Phase::Execute;
                self.Cpu_pipeline.remain_cycles = ins.op.cycles.decode_cycles.saturating_sub(1);
            }
            Phase::Execute => {
                self.Cpu_pipeline.phase = Phase::Execute;
                self.current_pc = current_pc;
                let pc_update = ins.op.exec.execute(&mut *self, &ins.data);
                print!(
                    "Executed instruction {} at 0x{:08X}, PC update: 0x{:X}\n",
                    ins.data.mnemonic(),
                    current_pc,
                    pc_update
                );
                self.update_pc(pc_update);
                self.Cpu_pipeline.remain_cycles = ins.op.cycles.execute_cycles.saturating_sub(1);
            }
        }
    }

    pub fn peripheral_step(&mut self) {
        self.peripheral_bus.borrow_mut().tick();
        self.ppb.borrow_mut().tick();
    }

    pub fn update_pc<'a>(&mut self, update: u32) {
        if update == 0 {
            self.next_pc = self.read_pc();
        } else {
            self.next_pc = self.current_pc.wrapping_add(update);
        }
    }
}
