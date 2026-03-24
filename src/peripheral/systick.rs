use super::peripheral::Peripheral;
use std::cell::Cell;
use std::any::Any;

const SYST_CSR_ENABLE: u32 = 1 << 0;
const SYST_CSR_TICKINT: u32 = 1 << 1;
const SYST_CSR_CLKSOURCE: u32 = 1 << 2;
const SYST_CSR_COUNTFLAG: u32 = 1 << 16;

#[derive(Default)]
pub struct SysTick {
    pub start: u32,
    pub end: u32,

    pub csr: Cell<u32>,   // 控制和状态寄存器
    pub rvr: Cell<u32>,   // 重载值寄存器
    pub cvr: Cell<u32>,   // 当前值寄存器
    pub calib: Cell<u32>, // 校准值寄存器
    pub interrupt_pending: Cell<bool>,
    pub interrupt_event: Cell<bool>,
    next_tick_cycle: Cell<Option<u32>>,
}

impl SysTick {
    pub fn new(start: u32, end: u32) -> Self {
        Self {
            start,
            end,
            csr: Cell::new(4), // 默认使用处理器时钟
            rvr: Cell::new(0),
            cvr: Cell::new(0),
            calib: Cell::new(0),
            interrupt_pending: Cell::new(false),
            interrupt_event: Cell::new(false),
            next_tick_cycle: Cell::new(None),
        }
    }

    #[inline(always)]
    fn refresh_next_tick_cycle(&self) {
        let next_tick_cycle = if !self.sys_enable_flag() {
            None
        } else {
            let cvr = self.cvr.get();
            let reload = self.rvr.get();
            Some(if cvr == 0 {
                reload.wrapping_add(1).max(1)
            } else {
                cvr.max(1)
            })
        };

        self.next_tick_cycle.set(next_tick_cycle);
    }

    fn sys_enable_flag(&self) -> bool {
        self.csr.get() & SYST_CSR_ENABLE != 0
    }

    #[inline(always)]
    fn on_wrap_event(&self) {
        let csr_now = self.csr.get();
        if (csr_now & SYST_CSR_COUNTFLAG) == 0 {
            self.csr.set(csr_now | SYST_CSR_COUNTFLAG);
        }
        if (csr_now & SYST_CSR_TICKINT) != 0 && !self.interrupt_pending.get() {
            self.interrupt_pending.set(true);
            self.interrupt_event.set(true);
        }
    }

    pub fn is_interrupt_pending(&self) -> bool {
        self.interrupt_pending.get()
    }

    pub fn clear_interrupt_pending(&self) {
        if self.interrupt_pending.get() {
            self.interrupt_pending.set(false);
            self.interrupt_event.set(true);
        }
    }

    pub fn take_interrupt_event(&self) -> bool {
        let event = self.interrupt_event.get();
        self.interrupt_event.set(false);
        event
    }
}

impl Peripheral for SysTick {
    fn start(&self) -> u32 {
        self.start
    }
    fn end(&self) -> u32 {
        self.end
    }

    fn read(&self, addr: u32) -> u32 {
        let offset = addr & 0x0F; // Assuming minimal offset space needed
        match offset {
            0x00 => {
                let val = self.csr.get();
                self.csr.set(val & !SYST_CSR_COUNTFLAG); // 读操作清除 COUNTFLAG
                val
            }
            0x04 => self.rvr.get(),
            0x08 => self.cvr.get(),
            0x0C => self.calib.get(),
            _ => 0,
        }
    }

    fn write(&mut self, addr: u32, val: u32) {
        let offset = addr & 0x0F;
        // println!("Writing to SysTick at offset 0x{:02X} value 0x{:08X}", offset, val);
        match offset {
            0x00 => {
                let prev_csr = self.csr.get();
                let writable = SYST_CSR_ENABLE | SYST_CSR_TICKINT | SYST_CSR_CLKSOURCE;
                let next_csr = (prev_csr & SYST_CSR_COUNTFLAG) | (val & writable);
                self.csr.set(next_csr);

                let was_enabled = (prev_csr & SYST_CSR_ENABLE) != 0;
                let is_enabled = (next_csr & SYST_CSR_ENABLE) != 0;
                if !was_enabled && is_enabled && self.cvr.get() == 0 {
                    self.cvr.set(self.rvr.get()); // 启用时 CVR 自动加载
                }
                self.refresh_next_tick_cycle();
            }
            0x04 => {
                self.rvr.set(val & 0xFFFFFF); // 24位
                self.refresh_next_tick_cycle();
            }
            0x08 => {
                // 写入任意值都会清除当前值和 COUNTFLAG
                // println!("Writing to CVR clears it and COUNTFLAG");
                self.cvr.set(0);
                self.csr.set(self.csr.get() & !SYST_CSR_COUNTFLAG);
                if self.interrupt_pending.get() {
                    self.interrupt_pending.set(false);
                    self.interrupt_event.set(true);
                }
                self.refresh_next_tick_cycle();
            }
            _ => {}
        }
    }
    fn tick(&mut self) {
        self.tick_n(1);
    }

    fn tick_n(&mut self, cycles: u32) {
        if cycles == 0 {
            return;
        }

        if !self.sys_enable_flag() {
            self.refresh_next_tick_cycle();
            return;
        }

        let mut remain = cycles;
        while remain > 0 {
            let cvr = self.cvr.get();
            if cvr == 0 {
                let reload = self.rvr.get();
                if reload == 0 {
                    self.on_wrap_event();
                } else {
                    self.cvr.set(reload);
                }
                remain -= 1;
                continue;
            }

            if remain < cvr {
                self.cvr.set(cvr - remain);
                self.refresh_next_tick_cycle();
                return;
            }

            remain -= cvr;
            self.cvr.set(0);
            self.on_wrap_event();
        }

        self.refresh_next_tick_cycle();
    }

    #[inline(always)]
    fn needs_tick(&self) -> bool {
        true
    }

    #[inline(always)]
    fn is_tick_active(&self) -> bool {
        self.sys_enable_flag()
    }

    #[inline(always)]
    fn interrupt_event_pending(&self) -> bool {
        self.interrupt_event.get()
    }

    #[inline(always)]
    fn next_event_in_cycles(&self) -> Option<u32> {
        self.next_tick_cycle.get()
    }

    #[inline(always)]
    fn next_tick_cycle(&self) -> Option<u32> {
        self.next_tick_cycle.get()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn systick_enable_loads_cvr_from_rvr() {
        let mut systick = SysTick::new(0xE000_E010, 0xE000_E01F);

        systick.write(0xE000_E014, 10);
        systick.write(0xE000_E010, 1);

        assert_eq!(systick.cvr.get(), 10);
    }

    #[test]
    fn systick_tick_sets_countflag_and_reloads() {
        let mut systick = SysTick::new(0xE000_E010, 0xE000_E01F);
        systick.write(0xE000_E014, 3);
        systick.write(0xE000_E010, 1);

        systick.tick();
        systick.tick();
        systick.tick();

        let csr = systick.read(0xE000_E010);
        assert_ne!(csr & (1 << 16), 0);
        assert_eq!(systick.cvr.get(), 0);

        systick.tick();
        assert_eq!(systick.cvr.get(), 3);
    }

    #[test]
    fn systick_read_csr_clears_countflag() {
        let systick = SysTick::new(0xE000_E010, 0xE000_E01F);
        systick.csr.set(systick.csr.get() | (1 << 16));

        let first = systick.read(0xE000_E010);
        let second = systick.read(0xE000_E010);

        assert_ne!(first & (1 << 16), 0);
        assert_eq!(second & (1 << 16), 0);
    }

    #[test]
    fn systick_rvr_zero_still_sets_countflag() {
        let mut systick = SysTick::new(0xE000_E010, 0xE000_E01F);
        systick.write(0xE000_E014, 0);
        systick.write(0xE000_E010, 1);

        systick.tick();
        let csr = systick.read(0xE000_E010);
        assert_ne!(csr & (1 << 16), 0);
    }

    #[test]
    fn systick_firmware_style_polling_loop_exits() {
        let mut systick = SysTick::new(0xE000_E010, 0xE000_E01F);

        systick.write(0xE000_E014, 16);
        systick.write(0xE000_E018, 0);
        systick.write(0xE000_E010, 0x5);

        let mut exited = false;
        for _ in 0..512 {
            systick.tick();
            if (systick.read(0xE000_E010) & 0x0001_0000) != 0 {
                exited = true;
                break;
            }
        }

        assert!(exited, "firmware polling loop should observe COUNTFLAG");
    }
}
