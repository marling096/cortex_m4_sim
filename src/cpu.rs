use std::vec;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;

use crate::context::CpuContext;
use crate::opcodes::instruction::Cpu_Instruction;
use crate::peripheral::bus::Bus;
use crate::peripheral::nvic::Nvic;
use crate::peripheral::systick::SysTick;

pub struct Cpu {
    pub frequency: Arc<AtomicU32>,
    pub machine_cycle: u8,
    pub Cycles: u64,

    pub Cpu_pipeline: Cpu_pipeline,

    pub flash: Vec<u8>, // 模拟 Flash (例如 512KB)
    pub ram: Vec<u8>,   // 模拟 SRAM (例如 128KB)
    pub registers: Registers,

    pub next_pc: u32, // 预取的下一条指令地址

    pub peripheral_bus: Bus,

    pub ppb: Bus,

    peripheral_tick_mask: u8,

    exception_stack: Vec<u32>,
    interrupt_check_hint: bool,
    interrupt_hint_source: u8,
    pending_ppb_event_drain: bool,
    peripheral_schedule_dirty: bool,
    peripheral_next_due_cycle: u64,
    ppb_nvic_index: Option<usize>,
    ppb_systick_index: Option<usize>,
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

    pub interrupt_check_calls: u64,
    pub interrupt_taken_count: u64,
    pub interrupt_check_duration: std::time::Duration,
    pub interrupt_hint_set_count: u64,
    pub interrupt_check_from_peripheral_count: u64,
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
    #[inline(always)]
    fn read_mem(&self, addr: u32) -> u32 {
        self.read32(addr)
    }
    #[inline(always)]
    fn write_mem(&mut self, addr: u32, val: u32) {
        self.write32(addr, val);
    }
    #[inline(always)]
    fn read_reg(&self, r: u32) -> u32 {
        // match r {
        //     13 => {
        //         // SP
        //         self.registers.reg[13]
        //     }
        //     14 => {
        //         // LR
        //         self.registers.reg[14]
        //     }
        //     15 => {
        //         // PC
        //         self.registers.reg[15]
        //     }
        //     _ => self.registers.reg[r as usize],
        // }
        self.registers.reg[r as usize]
    }
    #[inline(always)]
    fn write_reg(&mut self, r: u32, v: u32) {
        // match r {
        //     13 => {
        //         // SP
        //         self.registers.reg[13] = v;
        //     }
        //     14 => {
        //         // LR
        //         self.registers.reg[14] = v;
        //     }
        //     15 => {
        //         // PC
        //         self.registers.reg[15] = v;
        //     }
        //     _ => self.registers.reg[r as usize] = v,
        // }
        self.registers.reg[r as usize] = v;
    }
    #[inline(always)]
    fn read_gpr(&self, r: u32) -> u32 {
        self.registers.reg[r as usize]
    }
    #[inline(always)]
    fn write_gpr(&mut self, r: u32, v: u32) {
        self.registers.reg[r as usize] = v;
    }
    #[inline(always)]
    fn read_msp(&self, _r: u32) -> u32 {
        if self.registers.is_msp {
            self.registers.reg[13]
        } else {
            0 // TODO: handle banked MSP
        }
    }
    #[inline(always)]
    fn write_msp(&mut self, v: u32) {
        if self.registers.is_msp {
            self.registers.reg[13] = v;
        } else {
            // TODO: handle banked MSP
        }
    }
    #[inline(always)]
    fn read_psp(&self, _r: u32) -> u32 {
        if !self.registers.is_msp {
            self.registers.reg[13]
        } else {
            0 // TODO: handle banked PSP
        }
    }
    #[inline(always)]
    fn write_psp(&mut self, v: u32) {
        if !self.registers.is_msp {
            self.registers.reg[13] = v;
        } else {
            // TODO: handle banked PSP
        }
    }
    #[inline(always)]
    fn read_sp(&self) -> u32 {
        self.registers.reg[13]
    }
    #[inline(always)]
    fn write_sp(&mut self, v: u32) {
        self.registers.reg[13] = v;
    }
    #[inline(always)]
    fn read_lr(&self, _r: u32) -> u32 {
        self.registers.reg[14]
    }
    #[inline(always)]
    fn write_lr(&mut self, v: u32) {
        self.registers.reg[14] = v;
    }
    #[inline(always)]
    fn read_pc(&self) -> u32 {
        self.registers.reg[15]
    }
    #[inline(always)]
    fn write_pc(&mut self, pc: u32) {
        self.registers.reg[15] = pc;
    }
    #[inline(always)]
    fn read_apsr(&self) -> u32 {
        self.registers.apsr
    }
    #[inline(always)]
    fn write_apsr(&mut self, v: u32) {
        self.registers.apsr = v;
    }
    #[inline(always)]
    fn try_exception_return(&mut self, val: u32) -> bool {
        Cpu::try_exception_return(self, val)
    }
}

impl Cpu {
    const FLASH_ALIAS_BASE: u32 = 0x0000_0000;
    const FLASH_ALIAS_LAST: u32 = 0x0007_FFFF;
    const FLASH_BASE: u32 = 0x0800_0000;
    const FLASH_LAST: u32 = 0x0807_FFFF;
    const RAM_BASE: u32 = 0x2000_0000;
    const RAM_LAST: u32 = 0x3FFF_FFFF;
    const PERIPH_BASE: u32 = 0x4000_0000;
    const PERIPH_LAST: u32 = 0x5FFF_FFFF;
    const PPB_BASE: u32 = 0xE000_0000;
    const PPB_LAST: u32 = 0xE00F_FFFF;
    const SCB_VTOR_ADDR: u32 = 0xE000_ED08;
    const SCB_SHPR3_ADDR: u32 = 0xE000_ED20;
    const NVIC_BASE: u32 = 0xE000_E100;
    const NVIC_IABR_BASE: u32 = 0xE000_E300;
    const NVIC_IPR_BASE: u32 = 0xE000_E400;
    const NVIC_END: u32 = 0xE000_E4EF;
    const SYSTICK_BASE: u32 = 0xE000_E010;
    const SYSTICK_END: u32 = 0xE000_E01F;
    const NVIC_REG_COUNT: usize = 8;
    const SYSTICK_EXCEPTION_NUM: u32 = 15;
    const EXC_RETURN_HANDLER_MSP: u32 = 0xFFFF_FFF1;
    const EXC_RETURN_THREAD_MSP: u32 = 0xFFFF_FFF9;
    const HINT_SRC_NONE: u8 = 0;
    const HINT_SRC_OTHER: u8 = 1;
    const HINT_SRC_PERIPHERAL: u8 = 2;

    #[inline(always)]
    fn set_interrupt_check_hint(&mut self, source: u8) {
        self.interrupt_check_hint = true;

        if source == Self::HINT_SRC_PERIPHERAL || self.interrupt_hint_source == Self::HINT_SRC_NONE {
            self.interrupt_hint_source = source;
        }
    }

    #[inline(always)]
    fn clear_interrupt_check_hint(&mut self) {
        self.interrupt_check_hint = false;
        self.interrupt_hint_source = Self::HINT_SRC_NONE;
    }

    #[inline(always)]
    fn is_interrupt_related_ppb_write(addr: u32) -> bool {
        (Self::NVIC_BASE..=Self::NVIC_END).contains(&addr)
            || (Self::SYSTICK_BASE..=Self::SYSTICK_END).contains(&addr)
            || addr == Self::SCB_SHPR3_ADDR
    }

    #[inline(always)]
    fn drain_interrupt_events(&mut self) {
        let mut has_event = false;

        if let Some(index) = self.ppb_nvic_index {
            if let Some(nvic) = self.ppb.get_peripheral_mut_by_index::<Nvic>(index) {
                has_event |= nvic.take_interrupt_event();
            }
        }
        if let Some(index) = self.ppb_systick_index {
            if let Some(systick) = self.ppb.get_peripheral_mut_by_index::<SysTick>(index) {
                has_event |= systick.take_interrupt_event();
            }
        }

        if has_event {
            self.set_interrupt_check_hint(Self::HINT_SRC_PERIPHERAL);
        }
    }

    pub fn new(frequency: Arc<AtomicU32>, machine_cycle: u8, peripheral_bus: Bus, mut ppb: Bus) -> Cpu {
        let mut peripheral_tick_mask = 0u8;
        if peripheral_bus.has_tickables() {
            peripheral_tick_mask |= 1;
        }
        if ppb.has_tickables() {
            peripheral_tick_mask |= 2;
        }

        let ppb_nvic_index = ppb.find_peripheral_index(Self::NVIC_BASE);
        let ppb_systick_index = ppb.find_peripheral_index(Self::SYSTICK_BASE);

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
            peripheral_bus,
            ppb,
            peripheral_tick_mask,
            exception_stack: Vec::new(),
            interrupt_check_hint: true,
            interrupt_hint_source: Self::HINT_SRC_OTHER,
            pending_ppb_event_drain: false,
            peripheral_schedule_dirty: true,
            peripheral_next_due_cycle: 0,
            ppb_nvic_index,
            ppb_systick_index,
        }
    }

    pub fn set_profiling_enabled(&mut self, _enabled: bool) {
    }

    pub fn is_profiling_enabled(&self) -> bool {
        false
    }

    pub fn take_exec_profile(&mut self) -> CpuExecProfile {
        CpuExecProfile::default()
    }

    pub fn take_exec_op_stats(&mut self) -> Vec<(String, OpExecStat)> {
        Vec::new()
    }

    pub fn reset_handler(&mut self, reset_vector: u32) {
        // 复位时，PC 设置为复位向量地址
        self.write_pc(reset_vector);
        self.next_pc = reset_vector;
        // 其他寄存器初始化
        self.registers.reg = [0; 16];
        self.registers.apsr = 0;
        self.registers.is_msp = true;
        self.exception_stack.clear();
        self.interrupt_check_hint = true;
        self.interrupt_hint_source = Self::HINT_SRC_OTHER;
        self.pending_ppb_event_drain = false;
        self.peripheral_schedule_dirty = true;
        self.peripheral_next_due_cycle = 0;
    }

    #[inline(always)]
    fn is_exc_return(value: u32) -> bool {
        if (value & 0xFF00_0000) != 0xFF00_0000 {
            return false;
        }

        matches!(value & 0xF, 0x1 | 0x9 | 0xD)
    }

    #[inline(always)]
    fn push_stack_word(&mut self, value: u32) {
        let new_sp = self.registers.reg[13].wrapping_sub(4);
        self.registers.reg[13] = new_sp;
        if Self::in_range(new_sp, Self::RAM_BASE, Self::RAM_LAST) {
            let offset = (new_sp - Self::RAM_BASE) as usize;
            Self::write_u32_le_unchecked(&mut self.ram, offset, value);
            return;
        }
        self.write32(new_sp, value);
    }

    #[inline(always)]
    fn push_exception_frame_fast(
        &mut self,
        r0: u32,
        r1: u32,
        r2: u32,
        r3: u32,
        r12: u32,
        lr: u32,
        pc: u32,
        xpsr: u32,
    ) -> bool {
        let sp = self.registers.reg[13];
        let new_sp = sp.wrapping_sub(32);
        if !Self::in_range(new_sp, Self::RAM_BASE, Self::RAM_LAST) {
            return false;
        }

        let offset = (new_sp - Self::RAM_BASE) as usize;
        if offset + 32 > self.ram.len() {
            return false;
        }

        Self::write_u32_le_unchecked(&mut self.ram, offset, r0);
        Self::write_u32_le_unchecked(&mut self.ram, offset + 4, r1);
        Self::write_u32_le_unchecked(&mut self.ram, offset + 8, r2);
        Self::write_u32_le_unchecked(&mut self.ram, offset + 12, r3);
        Self::write_u32_le_unchecked(&mut self.ram, offset + 16, r12);
        Self::write_u32_le_unchecked(&mut self.ram, offset + 20, lr);
        Self::write_u32_le_unchecked(&mut self.ram, offset + 24, pc);
        Self::write_u32_le_unchecked(&mut self.ram, offset + 28, xpsr);
        self.registers.reg[13] = new_sp;
        true
    }

    #[inline(always)]
    fn pop_stack_word(&mut self) -> u32 {
        let sp = self.registers.reg[13];
        let value = if Self::in_range(sp, Self::RAM_BASE, Self::RAM_LAST) {
            let offset = (sp - Self::RAM_BASE) as usize;
            Self::read_u32_le_unchecked(&self.ram, offset)
        } else {
            self.read32(sp)
        };
        self.registers.reg[13] = sp.wrapping_add(4);
        value
    }

    #[inline(always)]
    fn pop_exception_frame_fast(&mut self) -> Option<(u32, u32, u32, u32, u32, u32, u32, u32)> {
        let sp = self.registers.reg[13];
        if !Self::in_range(sp, Self::RAM_BASE, Self::RAM_LAST) {
            return None;
        }

        let offset = (sp - Self::RAM_BASE) as usize;
        if offset + 32 > self.ram.len() {
            return None;
        }

        let r0 = Self::read_u32_le_unchecked(&self.ram, offset);
        let r1 = Self::read_u32_le_unchecked(&self.ram, offset + 4);
        let r2 = Self::read_u32_le_unchecked(&self.ram, offset + 8);
        let r3 = Self::read_u32_le_unchecked(&self.ram, offset + 12);
        let r12 = Self::read_u32_le_unchecked(&self.ram, offset + 16);
        let lr = Self::read_u32_le_unchecked(&self.ram, offset + 20);
        let pc = Self::read_u32_le_unchecked(&self.ram, offset + 24);
        let xpsr = Self::read_u32_le_unchecked(&self.ram, offset + 28);
        self.registers.reg[13] = sp.wrapping_add(32);
        Some((r0, r1, r2, r3, r12, lr, pc, xpsr))
    }

    #[inline(always)]
    fn read_exception_priority_fast(exception_num: u32, systick_priority: u8, nvic_ipr: &[u8; 240]) -> u8 {
        if exception_num == Self::SYSTICK_EXCEPTION_NUM {
            return systick_priority;
        }

        if exception_num >= 16 {
            let irq = (exception_num - 16) as usize;
            if irq < nvic_ipr.len() {
                return nvic_ipr[irq];
            }
        }

        0
    }

    fn pick_preemptable_exception(&mut self) -> Option<u32> {
        let mut in_service_mask = [0u32; Self::NVIC_REG_COUNT];
        for &exception_num in &self.exception_stack {
            if exception_num >= 16 {
                let irq = exception_num - 16;
                let index = (irq / 32) as usize;
                if index < Self::NVIC_REG_COUNT {
                    in_service_mask[index] |= 1u32 << (irq % 32);
                }
            }
        }

        let systick_priority = ((self.ppb.read32(Self::SCB_SHPR3_ADDR) >> 24) & 0xFF) as u8;

        let (nvic_active_word_bitmap, nvic_active_words, nvic_ipr) =
            if let Some(index) = self.ppb_nvic_index {
                if let Some(nvic) = self.ppb.get_peripheral_mut_by_index::<Nvic>(index) {
                    (nvic.active_word_bitmap, nvic.iabr, nvic.ipr)
                } else {
                    (0, [0; Self::NVIC_REG_COUNT], [0; 240])
                }
            } else {
                (0, [0; Self::NVIC_REG_COUNT], [0; 240])
            };

        let current_priority = self
            .exception_stack
            .last()
            .map(|exception_num| {
                Self::read_exception_priority_fast(*exception_num, systick_priority, &nvic_ipr)
            })
            .unwrap_or(u8::MAX);

        let mut best_exception: Option<u32> = None;
        let mut best_priority = u8::MAX;

        let systick_pending = if let Some(index) = self.ppb_systick_index {
            if let Some(systick) = self.ppb.get_peripheral_mut_by_index::<SysTick>(index) {
                systick.is_interrupt_pending()
            } else {
                false
            }
        } else {
            false
        };

        if systick_pending {
            let exception_num = Self::SYSTICK_EXCEPTION_NUM;
            let priority = systick_priority;
            let preemptable = self.exception_stack.is_empty() || priority < current_priority;
            if preemptable {
                best_exception = Some(exception_num);
                best_priority = priority;
            }
        }

        let mut active_words = nvic_active_word_bitmap;
        while active_words != 0 {
            let index = active_words.trailing_zeros() as usize;
            let mut candidates = nvic_active_words[index] & !in_service_mask[index];

            while candidates != 0 {
                let bit = candidates.trailing_zeros();
                let irq_num = (index as u32) * 32 + bit;
                let exception_num = 16 + irq_num;
                let priority = Self::read_exception_priority_fast(exception_num, systick_priority, &nvic_ipr);

                let preemptable = self.exception_stack.is_empty() || priority < current_priority;
                if preemptable
                    && (best_exception.is_none()
                        || priority < best_priority
                        || (priority == best_priority
                            && exception_num < best_exception.unwrap_or(u32::MAX)))
                {
                    best_exception = Some(exception_num);
                    best_priority = priority;
                }

                candidates &= candidates - 1;
            }

            active_words &= active_words - 1;
        }

        best_exception
    }

    fn enter_exception(&mut self, exception_num: u32) {
        let return_pc = self.next_pc | 1;
        let return_lr = self.read_lr(14);
        let xpsr = (self.read_apsr() & 0xF000_0000) | 0x0100_0000 | (exception_num & 0x1FF);

        if !self.push_exception_frame_fast(
            self.registers.reg[0],
            self.registers.reg[1],
            self.registers.reg[2],
            self.registers.reg[3],
            self.registers.reg[12],
            return_lr,
            return_pc,
            xpsr,
        ) {
            self.push_stack_word(xpsr);
            self.push_stack_word(return_pc);
            self.push_stack_word(return_lr);
            self.push_stack_word(self.read_gpr(12));
            self.push_stack_word(self.read_gpr(3));
            self.push_stack_word(self.read_gpr(2));
            self.push_stack_word(self.read_gpr(1));
            self.push_stack_word(self.read_gpr(0));
        }

        let exc_return = if self.exception_stack.is_empty() {
            Self::EXC_RETURN_THREAD_MSP
        } else {
            Self::EXC_RETURN_HANDLER_MSP
        };

        self.write_lr(exc_return);

        let vector_base = self.ppb.read32(Self::SCB_VTOR_ADDR);
        let vector_addr = vector_base.wrapping_add(exception_num * 4);
        let handler = self.read32(vector_addr) & !1;

        self.write_pc(handler);
        self.next_pc = handler;
        self.exception_stack.push(exception_num);

        if exception_num == Self::SYSTICK_EXCEPTION_NUM {
            if let Some(index) = self.ppb_systick_index {
                if let Some(systick) = self.ppb.get_peripheral_mut_by_index::<SysTick>(index) {
                    systick.clear_interrupt_pending();
                }
            }
        }
    }

    fn try_take_interrupt_inner(&mut self) -> bool {
        if !self.interrupt_check_hint {
            return false;
        }

        if let Some(exception_num) = self.pick_preemptable_exception() {
            self.enter_exception(exception_num);
            self.clear_interrupt_check_hint();
            return true;
        }

        self.clear_interrupt_check_hint();
        false
    }

    fn try_take_interrupt(&mut self) -> bool {
        if !self.interrupt_check_hint {
            return false;
        }

        self.try_take_interrupt_inner()

    }

    pub fn try_exception_return(&mut self, exc_return: u32) -> bool {
        if !Self::is_exc_return(exc_return) || self.exception_stack.is_empty() {
            return false;
        }

        let (r0, r1, r2, r3, r12, lr, pc, xpsr) = if let Some(frame) = self.pop_exception_frame_fast() {
            frame
        } else {
            (
                self.pop_stack_word(),
                self.pop_stack_word(),
                self.pop_stack_word(),
                self.pop_stack_word(),
                self.pop_stack_word(),
                self.pop_stack_word(),
                self.pop_stack_word(),
                self.pop_stack_word(),
            )
        };

        self.registers.reg[0] = r0;
        self.registers.reg[1] = r1;
        self.registers.reg[2] = r2;
        self.registers.reg[3] = r3;
        self.registers.reg[12] = r12;
        self.registers.reg[14] = lr;
        self.registers.apsr = xpsr & 0xF000_0000;

        let target_pc = pc & !1;
        self.registers.reg[15] = target_pc;
        self.next_pc = target_pc;

        if let Some(exception_num) = self.exception_stack.pop() {
            if exception_num >= 16 {
                if let Some(index) = self.ppb_nvic_index {
                    if let Some(nvic) = self.ppb.get_peripheral_mut_by_index::<Nvic>(index) {
                        nvic.clear_active_irq(exception_num - 16);
                    }
                }
            }
        }

        self.drain_interrupt_events();
        self.set_interrupt_check_hint(Self::HINT_SRC_OTHER);

        true
    }

    #[inline(always)]
    fn read32(&self, addr: u32) -> u32 {
        self.read32_inner(addr)
    }

    /// 核心写入函数：处理不同区域的写入权限
    #[inline(always)]
    fn write32(&mut self, addr: u32, val: u32) {
        self.write32_inner(addr, val);
    }

    #[inline(always)]
    fn in_range(addr: u32, base: u32, last: u32) -> bool {
        addr.wrapping_sub(base) <= (last - base)
    }

    #[inline(always)]
    fn read_u32_le_unchecked(buf: &[u8], offset: usize) -> u32 {
        debug_assert!(offset + 4 <= buf.len());
        unsafe {
            let p = buf.as_ptr().add(offset) as *const u32;
            u32::from_le(std::ptr::read_unaligned(p))
        }
    }

    #[inline(always)]
    fn write_u32_le_unchecked(buf: &mut [u8], offset: usize, val: u32) {
        debug_assert!(offset + 4 <= buf.len());
        unsafe {
            let p = buf.as_mut_ptr().add(offset) as *mut u32;
            std::ptr::write_unaligned(p, val.to_le());
        }
    }

    #[inline(always)]
    fn read32_inner(&self, addr: u32) -> u32 {
        if Self::in_range(addr, Self::RAM_BASE, Self::RAM_LAST) {
            let offset = (addr - Self::RAM_BASE) as usize;
            return Self::read_u32_le_unchecked(&self.ram, offset);
        }

        if Self::in_range(addr, Self::FLASH_BASE, Self::FLASH_LAST) {
            let offset = (addr - Self::FLASH_BASE) as usize;
            return Self::read_u32_le_unchecked(&self.flash, offset);
        }

        if Self::in_range(addr, Self::FLASH_ALIAS_BASE, Self::FLASH_ALIAS_LAST) {
            return Self::read_u32_le_unchecked(&self.flash, addr as usize);
        }

        if Self::in_range(addr, Self::PERIPH_BASE, Self::PERIPH_LAST) {
            return self.peripheral_bus.read32(addr);
        }

        if Self::in_range(addr, Self::PPB_BASE, Self::PPB_LAST) {
            return self.ppb.read32(addr);
        }

        panic!("Memory Read Error: Unmapped address 0x{:08X}", addr);
    }

    #[inline(always)]
    fn write32_inner(&mut self, addr: u32, val: u32) {
        if Self::in_range(addr, Self::RAM_BASE, Self::RAM_LAST) {
            let offset = (addr - Self::RAM_BASE) as usize;
            Self::write_u32_le_unchecked(&mut self.ram, offset, val);
            return;
        }

        if Self::in_range(addr, Self::FLASH_BASE, Self::FLASH_LAST) {
            let offset = (addr - Self::FLASH_BASE) as usize;
            Self::write_u32_le_unchecked(&mut self.flash, offset, val);
            return;
        }

        if Self::in_range(addr, Self::FLASH_ALIAS_BASE, Self::FLASH_ALIAS_LAST) {
            return;
        }

        if Self::in_range(addr, Self::PERIPH_BASE, Self::PERIPH_LAST) {
            let schedule_changed = self.peripheral_bus.write32(addr, val);
            if schedule_changed {
                self.peripheral_schedule_dirty = true;
            }
            return;
        }

        if Self::in_range(addr, Self::PPB_BASE, Self::PPB_LAST) {
            let schedule_changed = self.ppb.write32(addr, val);
            if schedule_changed {
                self.peripheral_schedule_dirty = true;
            }
            if Self::is_interrupt_related_ppb_write(addr) {
                self.set_interrupt_check_hint(Self::HINT_SRC_OTHER);
            }
            return;
        }

        panic!("Memory Write Error: Unmapped address 0x{:08X}", addr);
    }

    #[inline(always)]
    pub fn begin_step(&mut self) -> Option<u32> {
        if self.pending_ppb_event_drain {
            self.drain_interrupt_events();
            self.pending_ppb_event_drain = false;
        }
        if self.interrupt_check_hint && self.try_take_interrupt() {
            return Some(1);
        }

        if self.Cpu_pipeline.remain_cycles > 0 {
            self.Cpu_pipeline.remain_cycles -= 1;
            return Some(1);
        }

        None
    }

    #[inline(always)]
    pub fn finish_step_cycles(
        &mut self,
        execute_cycles: u32,
        current_pc: u32,
        pc_update: u32,
    ) -> u32 {
        self.update_pc_with_current(current_pc, pc_update);

        self.Cpu_pipeline.remain_cycles = execute_cycles.saturating_sub(1);
        if self.interrupt_check_hint {
            self.try_take_interrupt();
        }
        1
    }

    #[inline(always)]
    pub fn finish_step<'a>(
        &mut self,
        ins: &Cpu_Instruction<'a>,
        current_pc: u32,
        pc_update: u32,
    ) -> u32 {
        self.finish_step_cycles(ins.op.cycles.execute_cycles, current_pc, pc_update)
    }

    // changed: make step take &mut self and avoid borrow conflicts
    #[inline(always)]
    pub fn step<'a>(&mut self, ins: &Cpu_Instruction<'a>, current_pc: u32) -> u32 {
        if let Some(cycles) = self.begin_step() {
            return cycles;
        }

        let pc_update = (ins.op.exec)(&mut *self, &ins.data);
        self.finish_step(ins, current_pc, pc_update)
    }

    #[inline(always)]
    pub fn peripheral_step(&mut self) {
        self.peripheral_step_n(1);
    }

    #[inline(always)]
    pub fn peripheral_step_n(&mut self, cycles: u32) {
        if cycles == 0 {
            return;
        }
        let tick_mask = self.peripheral_tick_mask;
        if tick_mask == 0 {
            return;
        }

        let mut has_peripheral_event = false;
        let mut has_ppb_event = false;

        if (tick_mask & 1) != 0 {
            has_peripheral_event = self.peripheral_bus.tick_n(cycles);
            if has_peripheral_event {
                self.drain_peripheral_bus_irqs();
            }
        }
        if (tick_mask & 2) != 0 {
            has_ppb_event = self.ppb.tick_n(cycles);
            if has_ppb_event {
                self.pending_ppb_event_drain = true;
            }
        }

        if has_peripheral_event || has_ppb_event {
            self.set_interrupt_check_hint(Self::HINT_SRC_PERIPHERAL);
        }
    }

    #[inline(always)]
    pub fn next_peripheral_event_in_cycles(&self) -> Option<u32> {
        let tick_mask = self.peripheral_tick_mask;
        if tick_mask == 0 {
            return None;
        }

        let peripheral_next = if (tick_mask & 1) != 0 {
            self.peripheral_bus.next_event_in_cycles()
        } else {
            None
        };

        let ppb_next = if (tick_mask & 2) != 0 {
            self.ppb.next_event_in_cycles()
        } else {
            None
        };

        match (peripheral_next, ppb_next) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }

    #[inline(always)]
    pub fn take_and_clear_peripheral_schedule_dirty(&mut self) -> bool {
        let dirty = self.peripheral_schedule_dirty;
        self.peripheral_schedule_dirty = false;
        dirty
    }

    #[inline(always)]
    pub fn refresh_peripheral_due_cycle(&mut self, system_cycles: u64, max_lag_cycles: u32) {
        let max_lag = max_lag_cycles.max(1);
        let next_delta = self
            .next_peripheral_event_in_cycles()
            .unwrap_or(max_lag)
            .max(1)
            .min(max_lag);
        self.peripheral_next_due_cycle = system_cycles.saturating_add(next_delta as u64);
    }

    #[inline(always)]
    pub fn peripheral_due_cycle(&self) -> u64 {
        self.peripheral_next_due_cycle
    }

    /// 收集 peripheral_bus 上各外设挂起的 IRQ，通过 NVIC ISPR 寄存器触发中断。
    #[inline(always)]
    fn drain_peripheral_bus_irqs(&mut self) {
        if !self.peripheral_bus.has_irq_sources() {
            return;
        }

        // 先把 IRQ 号收集到栈上小数组，避免同时持有对 peripheral_bus 和 ppb 的可变引用
        let mut pending: [u32; 8] = [u32::MAX; 8];
        let mut count = 0usize;

        self.peripheral_bus.drain_pending_irqs(|irq| {
            if count < pending.len() {
                pending[count] = irq;
                count += 1;
            }
        });

        if count == 0 {
            return;
        }

        // 通过写 NVIC ISPR 寄存器，将中断置为 pending 状态
        // NVIC_BASE = 0xE000_E100，ISPR 偏移 0x100：ISPR[0] = 0xE000_E200
        for &irq in &pending[..count] {
            let index = irq / 32;
            let bit   = irq % 32;
            let ispr_addr = Self::NVIC_BASE + 0x100 + index * 4;
            self.ppb.write32(ispr_addr, 1u32 << bit);
        }

        self.set_interrupt_check_hint(Self::HINT_SRC_PERIPHERAL);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peripheral::nvic::Nvic;
    use crate::peripheral::scb::Scb;
    use crate::peripheral::systick::SysTick;
    use std::hint::black_box;
    use std::sync::Arc;
    use std::sync::atomic::AtomicU32;
    use std::time::Instant;

    fn report_perf(name: &str, iterations: u64, elapsed: std::time::Duration) {
        let total_ns = elapsed.as_nanos() as f64;
        let ns_per_op = if iterations == 0 {
            0.0
        } else {
            total_ns / iterations as f64
        };
        let mops = if elapsed.as_secs_f64() > 0.0 {
            iterations as f64 / elapsed.as_secs_f64() / 1_000_000.0
        } else {
            0.0
        };

        println!(
            "CPU Perf [{}]: iters={} total={:.3}ms ns/op={:.3} throughput={:.3} Mops/s",
            name,
            iterations,
            elapsed.as_secs_f64() * 1000.0,
            ns_per_op,
            mops
        );
    }

    fn build_cpu_with_ppb() -> Cpu {
        let mut ppb = Bus::new();
        ppb.register_peripheral(Box::new(SysTick::new(0xE000_E010, 0xE000_E01F)));
        ppb.register_peripheral(Box::new(Nvic::new(0xE000_E100, 0xE000_E4EF)));
        ppb.register_peripheral(Box::new(Scb::new(0xE000_ED00, 0xE000_ED3C)));

        Cpu::new(Arc::new(AtomicU32::new(8_000_000)), 1, Bus::new(), ppb)
    }

    #[test]
    fn cpu_takes_systick_exception_and_clears_pending() {
        let mut cpu = build_cpu_with_ppb();
        let initial_sp = 0x2000_1000;
        let initial_pc = 0x0800_0100;
        let handler = 0x0800_1234;

        cpu.write_sp(initial_sp);
        cpu.write_pc(initial_pc);
        cpu.next_pc = initial_pc;
        cpu.write_mem(0x0800_003C, handler | 1);

        let systick = cpu
            .ppb
            .get_peripheral_mut::<SysTick>(0xE000_E010)
            .expect("SysTick peripheral should exist");
        systick.interrupt_pending.set(true);

        assert!(cpu.try_take_interrupt());
        assert_eq!(cpu.read_pc(), handler & !1);
        assert_eq!(cpu.read_lr(14), Cpu::EXC_RETURN_THREAD_MSP);
        assert_eq!(cpu.exception_stack.last().copied(), Some(15));

        let systick = cpu
            .ppb
            .get_peripheral_mut::<SysTick>(0xE000_E010)
            .expect("SysTick peripheral should exist");
        assert!(!systick.is_interrupt_pending());
    }

    #[test]
    fn cpu_exception_return_clears_nvic_active() {
        let mut cpu = build_cpu_with_ppb();
        let initial_sp = 0x2000_1000;
        let initial_pc = 0x0800_0200;
        let irq_num = 5u32;
        let exception_num = 16 + irq_num;
        let vector_addr = 0x0800_0000 + exception_num * 4;
        let handler = 0x0800_2340;

        cpu.write_sp(initial_sp);
        cpu.write_pc(initial_pc);
        cpu.next_pc = initial_pc;
        cpu.write_mem(vector_addr, handler | 1);

        cpu.write_mem(0xE000_E100, 1 << irq_num);
        cpu.write_mem(0xE000_E200, 1 << irq_num);

        assert!(cpu.try_take_interrupt());
        assert_eq!(cpu.exception_stack.last().copied(), Some(exception_num));
        assert_ne!(cpu.read_mem(0xE000_E300) & (1 << irq_num), 0);

        let exc_return = cpu.read_lr(14);
        assert!(cpu.try_exception_return(exc_return));

        assert_eq!(cpu.read_pc(), initial_pc);
        assert_eq!(cpu.read_sp(), initial_sp);
        assert!(cpu.exception_stack.is_empty());
        assert_eq!(cpu.read_mem(0xE000_E300) & (1 << irq_num), 0);
    }

    #[test]
    fn perf_cpu_try_take_interrupt_fastpath_no_hint() {
        let loops = 5_000_000u64;
        let mut cpu = build_cpu_with_ppb();
        cpu.interrupt_check_hint = false;
        cpu.interrupt_hint_source = Cpu::HINT_SRC_NONE;

        let start = Instant::now();
        for _ in 0..loops {
            black_box(cpu.try_take_interrupt());
        }
        let elapsed = start.elapsed();

        report_perf("try_take_interrupt_fastpath_no_hint", loops, elapsed);
        assert!(elapsed.as_nanos() > 0);
    }

    #[test]
    fn perf_cpu_try_take_interrupt_event_path() {
        let loops = 500_000u64;
        let irq_num = 5u32;
        let exception_num = 16 + irq_num;

        let mut cpu = build_cpu_with_ppb();
        cpu.write_sp(0x2000_2000);
        cpu.write_pc(0x0800_0200);
        cpu.next_pc = 0x0800_0200;
        cpu.write_mem(0x0800_0000 + exception_num * 4, 0x0800_2341);
        cpu.write_mem(0xE000_E100, 1 << irq_num);

        let start = Instant::now();
        for _ in 0..loops {
            cpu.write_mem(0xE000_E200, 1 << irq_num);
            black_box(cpu.try_take_interrupt());
            let exc_return = cpu.read_lr(14);
            black_box(cpu.try_exception_return(exc_return));
        }
        let elapsed = start.elapsed();

        report_perf("try_take_interrupt_event_path", loops * 3, elapsed);
        assert!(elapsed.as_nanos() > 0);
    }

    fn prepare_irq_cpu(irq_num: u32) -> Cpu {
        let mut cpu = build_cpu_with_ppb();
        let exception_num = 16 + irq_num;
        cpu.write_sp(0x2000_2000);
        cpu.write_pc(0x0800_0200);
        cpu.next_pc = 0x0800_0200;
        cpu.write_mem(0x0800_0000 + exception_num * 4, 0x0800_2341);
        cpu.write_mem(0xE000_E100, 1 << irq_num);
        cpu
    }

    #[test]
    fn perf_cpu_irq_set_pending_only() {
        let loops = 1_000_000u64;
        let irq_num = 5u32;
        let mut cpu = prepare_irq_cpu(irq_num);

        let start = Instant::now();
        for _ in 0..loops {
            cpu.write_mem(0xE000_E200, 1 << irq_num);
            black_box(cpu.interrupt_check_hint);
        }
        let elapsed = start.elapsed();

        report_perf("irq_set_pending_only", loops, elapsed);
        assert!(elapsed.as_nanos() > 0);
    }

    #[test]
    fn perf_cpu_irq_take_only() {
        let loops = 500_000u64;
        let irq_num = 5u32;
        let mut cpu = prepare_irq_cpu(irq_num);

        let start = Instant::now();
        for _ in 0..loops {
            cpu.write_mem(0xE000_E200, 1 << irq_num);
            black_box(cpu.try_take_interrupt());
            let exc_return = cpu.read_lr(14);
            black_box(cpu.try_exception_return(exc_return));
        }
        let elapsed = start.elapsed();

        report_perf("irq_take_only", loops, elapsed);
        assert!(elapsed.as_nanos() > 0);
    }

    #[test]
    fn perf_cpu_irq_return_only() {
        let loops = 500_000u64;
        let irq_num = 5u32;
        let mut cpu = prepare_irq_cpu(irq_num);

        let start = Instant::now();
        for _ in 0..loops {
            cpu.write_mem(0xE000_E200, 1 << irq_num);
            let taken = cpu.try_take_interrupt();
            black_box(taken);
            let exc_return = cpu.read_lr(14);
            black_box(cpu.try_exception_return(exc_return));
        }
        let elapsed = start.elapsed();

        report_perf("irq_return_only", loops, elapsed);
        assert!(elapsed.as_nanos() > 0);
    }
}
