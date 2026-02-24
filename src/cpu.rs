use std::cell::RefCell;
use std::collections::BTreeMap;
use std::vec;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

use crate::context::CpuContext;
use crate::opcodes::instruction::Cpu_Instruction;
use crate::peripheral::bus::Bus;

pub struct Cpu {
    pub frequency: Arc<AtomicU32>,
    pub machine_cycle: u8,
    pub Cycles: u64,

    pub Cpu_pipeline: Cpu_pipeline,

    pub flash: Vec<u8>, // 模拟 Flash (例如 512KB)
    pub ram: Vec<u8>,   // 模拟 SRAM (例如 128KB)
    pub registers: Registers,

    pub next_pc: u32, // 预取的下一条指令地址

    pub peripheral_bus: RefCell<Bus>,

    pub ppb: RefCell<Bus>,

    exec_profile: RefCell<CpuExecProfile>,
    exec_op_stats: RefCell<BTreeMap<String, OpExecStat>>,
    profiling_enabled: bool,
}

#[derive(Default, Clone, Copy)]
pub struct CpuExecProfile {
    pub step_calls: u64,
    pub execute_calls: u64,
    pub pipeline_stall_count: u64,

    pub op_exec_duration: std::time::Duration,
    pub update_pc_duration: std::time::Duration,

    pub mem_read_count: u64,
    pub mem_write_count: u64,
    pub mem_read_duration: std::time::Duration,
    pub mem_write_duration: std::time::Duration,
}

#[derive(Default, Clone, Copy)]
pub struct OpExecStat {
    pub calls: u64,
    pub total_duration: std::time::Duration,
    pub max_duration: std::time::Duration,
}
struct Registers {
    reg: [u32; 16], // R0 - R15    // 应用程序状态寄存器 (xPSR 的一部分)
    apsr: u32,
    is_msp: bool, // 当前使用的是否是 MSP
}

struct Cpu_pipeline {
    remain_cycles: u32,
}

impl Cpu_pipeline {
    fn new() -> Cpu_pipeline {
        Cpu_pipeline {
            remain_cycles: 0,
        }
    }
}

impl CpuContext for Cpu {
    fn read_mem(&self, addr: u32) -> u32 {
        self.read32(addr)
    }

    fn write_mem(&mut self, addr: u32, val: u32) {
        self.write32(addr, val);
    }

    fn read_reg(&self, r: u32) -> u32 {
        match r {
            13 => {
                // SP
                self.registers.reg[13]
            }
            14 => {
                // LR
                self.registers.reg[14]
            }
            15 => {
                // PC
                self.registers.reg[15]
            }
            _ => self.registers.reg[r as usize],
        }
    }

    fn write_reg(&mut self, r: u32, v: u32) {
        match r {
            13 => {
                // SP
                self.registers.reg[13] = v;
            }
            14 => {
                // LR
                self.registers.reg[14] = v;
            }
            15 => {
                // PC
                self.registers.reg[15] = v;
            }
            _ => self.registers.reg[r as usize] = v,
        }
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
    pub fn new(frequency: Arc<AtomicU32>, machine_cycle: u8, peripheral_bus: Bus, ppb: Bus) -> Cpu {
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
            next_pc: 0,
            peripheral_bus: RefCell::new(peripheral_bus),
            ppb: RefCell::new(ppb),
            exec_profile: RefCell::new(CpuExecProfile::default()),
            exec_op_stats: RefCell::new(BTreeMap::new()),
            profiling_enabled: true,
        }
    }

    pub fn set_profiling_enabled(&mut self, enabled: bool) {
        self.profiling_enabled = enabled;
        if !enabled {
            *self.exec_profile.borrow_mut() = CpuExecProfile::default();
            self.exec_op_stats.borrow_mut().clear();
        }
    }

    pub fn is_profiling_enabled(&self) -> bool {
        self.profiling_enabled
    }

    pub fn take_exec_profile(&mut self) -> CpuExecProfile {
        let snapshot = *self.exec_profile.borrow();
        *self.exec_profile.borrow_mut() = CpuExecProfile::default();
        snapshot
    }

    pub fn take_exec_op_stats(&mut self) -> Vec<(String, OpExecStat)> {
        let mut stats = self.exec_op_stats.borrow_mut();
        let snapshot = stats.iter().map(|(mnemonic, stat)| (mnemonic.clone(), *stat)).collect();
        stats.clear();
        snapshot
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

    #[inline(always)]
    fn read32(&self, addr: u32) -> u32 {
        let start = if self.profiling_enabled {
            Some(Instant::now())
        } else {
            None
        };
        let value = match addr {
            // 1. 别名区域 (Aliased region)
            // 将 0x0000_0000 起始的访问重定向到 Flash
            0x0000_0000..=0x0007_FFFF => {
                let offset = addr as usize;
                let bytes = &self.flash[offset..offset + 4];
                u32::from_le_bytes(bytes.try_into().unwrap())
            }

            // 2. 物理 Flash 区域 (Physical Flash)
            // 假设 Flash 基地址是 0x0800_0000
            0x0800_0000..=0x0807_FFFF => {
                let offset = (addr - 0x0800_0000) as usize;
                let bytes = &self.flash[offset..offset + 4];
                u32::from_le_bytes(bytes.try_into().unwrap())
            }
            // RAM 区域 (例如 0x2000_0000 - 0x3FFFFFFF)
            0x2000_0000..=0x3FFFFFFF => {
                let offset = (addr - 0x2000_0000) as usize;
                let bytes = &self.ram[offset..offset + 4];
                u32::from_le_bytes(bytes.try_into().unwrap())
            }

            0x40000000..=0x5FFFFFFF => self.peripheral_bus.borrow_mut().read32(addr),
            0xE000_0000..=0xE00F_FFFF => {
                // 这里可以后续实现System 控制寄存器寄存器
                self.ppb.borrow_mut().read32(addr)
            }
            _ => {
                // 触发仿真异常：访问了未映射的地址
                panic!("Memory Read Error: Unmapped address 0x{:08X}", addr);
            }
        };

        if let Some(start) = start {
            let elapsed = start.elapsed();
            let mut profile = self.exec_profile.borrow_mut();
            profile.mem_read_count += 1;
            profile.mem_read_duration += elapsed;
        }

        value
    }

    /// 核心写入函数：处理不同区域的写入权限
    #[inline(always)]
    fn write32(&mut self, addr: u32, val: u32) {
        let start = if self.profiling_enabled {
            Some(Instant::now())
        } else {
            None
        };
        match addr {
            // RAM 区域
            0x2000_0000..=0x3FFFFFFF => {
                let offset = (addr - 0x2000_0000) as usize;
                let bytes = val.to_le_bytes();
                self.ram[offset..offset + 4].copy_from_slice(&bytes);
            }
            // Flash 区域：通常只读，直接写入可能在仿真中报错或模拟 Flash 控制器
            0x0000_0000..=0x0007_FFFF => {
                // eprintln!("Warning: Attempted to write to Flash at 0x{:08X}", addr);
            }
            0x0800_0000..=0x0807_FFFF => {
                // eprintln!("Warning: Attempted to write to Flash at 0x{:08X}", addr);
                let offset = (addr - 0x0800_0000) as usize;
                let bytes = val.to_le_bytes();
                self.flash[offset..offset + 4].copy_from_slice(&bytes);
            }
            0x40000000..=0x5FFFFFFF => {
                // 这里可以后续实现外设寄存器写入
                // print!("Peripheral Write at 0x{:08X} Value: 0x{:08X}\n", addr, val);
                self.peripheral_bus.borrow_mut().write32(addr, val);
            }

            0xE000_0000..=0xE00F_FFFF => {
                // println!("PPB Write at 0x{:08X} Value: 0x{:08X}", addr, val);
                self.ppb.borrow_mut().write32(addr, val);
            }
            _ => {
                panic!("Memory Write Error: Unmapped address 0x{:08X}", addr);
            }
        }

        if let Some(start) = start {
            let elapsed = start.elapsed();
            let mut profile = self.exec_profile.borrow_mut();
            profile.mem_write_count += 1;
            profile.mem_write_duration += elapsed;
        }
    }

    // changed: make step take &mut self and avoid borrow conflicts
    #[inline(always)]
    pub fn step<'a>(&mut self, ins: &Cpu_Instruction<'a>, current_pc: u32) {
        if !self.profiling_enabled {
            if self.Cpu_pipeline.remain_cycles > 0 {
                self.Cpu_pipeline.remain_cycles -= 1;
                return;
            }

            let pc_update = ins.op.exec.execute(&mut *self, &ins.data);
            self.update_pc_with_current(current_pc, pc_update);
            self.Cpu_pipeline.remain_cycles = ins.op.cycles.execute_cycles.saturating_sub(1);
            return;
        }

        self.exec_profile.borrow_mut().step_calls += 1;

        if self.Cpu_pipeline.remain_cycles > 0 {
            self.Cpu_pipeline.remain_cycles -= 1;
            self.exec_profile.borrow_mut().pipeline_stall_count += 1;
            return;
        }
        self.exec_profile.borrow_mut().execute_calls += 1;

        let op_start = Instant::now();
        let pc_update = ins.op.exec.execute(&mut *self, &ins.data);
        let op_elapsed = op_start.elapsed();
        self.exec_profile.borrow_mut().op_exec_duration += op_elapsed;
        {
            let mnemonic = ins.data.mnemonic().to_string();
            let mut stats = self.exec_op_stats.borrow_mut();
            let entry = stats.entry(mnemonic).or_default();
            entry.calls += 1;
            entry.total_duration += op_elapsed;
            entry.max_duration = entry.max_duration.max(op_elapsed);
        }

        let update_start = Instant::now();
        self.update_pc_with_current(current_pc, pc_update);
        let update_elapsed = update_start.elapsed();
        self.exec_profile.borrow_mut().update_pc_duration += update_elapsed;

        self.Cpu_pipeline.remain_cycles = ins.op.cycles.execute_cycles.saturating_sub(1);
    }

    #[inline(always)]
    pub fn peripheral_step(&mut self) {
        self.peripheral_bus.borrow_mut().tick();
        self.ppb.borrow_mut().tick();
    }

    #[inline(always)]
    pub fn update_pc<'a>(&mut self, update: u32) {
        if update == 0 {
            self.next_pc = self.read_pc();
        } else {
            self.next_pc = self.read_pc().wrapping_sub(4).wrapping_add(update);
        }
    }

    #[inline(always)]
    fn update_pc_with_current(&mut self, current_pc: u32, update: u32) {
        if update == 0 {
            self.next_pc = self.read_pc();
        } else {
            self.next_pc = current_pc.wrapping_add(update);
        }
    }

    #[inline(always)]
    pub fn prefetch_next_pc(&mut self, current_pc: u32) {
        self.registers.reg[15] = current_pc.wrapping_add(4);
    }
}
