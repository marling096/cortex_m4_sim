use crate::peripheral::peripheral::Peripheral;
use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Default)]
pub struct Rcc {
    pub start: u32,
    pub end: u32,
    pub sys_clock: Arc<AtomicU32>,

    pub cr: u32,       // 时钟控制寄存器
    pub cfgr: u32,     // 时钟配置寄存器
    pub cir: u32,      // 时钟中断寄存器
    pub ahb1rstr: u32, // AHB1 外设复位寄存器
    pub apb1rstr: u32, // APB1 外设复位寄存器
    pub apb2rstr: u32, // APB2 外设复位寄存器
    pub ahb1enr: u32,  // AHB1 外设时钟使能寄存器
    pub apb1enr: u32,  // APB1 外设时钟使能寄存器
    pub apb2enr: u32,  // APB2 外设时钟使能寄存器
    pub bdcr: u32,     // 备份域控制寄存器
    pub csr: u32,      // 控制/状态寄存器
                       // 可根据需要继续添加其他 RCC 寄存器
}

impl Rcc {
    const HSI_FREQ: u32 = 16_000_000;
    const HSE_FREQ: u32 = 8_000_000;

    pub fn new(start: u32, end: u32, sys_clock: Arc<AtomicU32>) -> Self {
        Self {
            start,
            end,
            sys_clock,
            ..Default::default()
        }
    }

    fn is_hsi_ready(&self) -> bool {
        (self.cr & (1 << 0)) != 0 && (self.cr & (1 << 1)) != 0
    }

    fn is_hse_ready(&self) -> bool {
        (self.cr & (1 << 16)) != 0 && (self.cr & (1 << 17)) != 0
    }

    fn is_pll_ready(&self) -> bool {
        (self.cr & (1 << 24)) != 0 && (self.cr & (1 << 25)) != 0
    }

    fn pll_input_clock(&self) -> Option<u32> {
        let pll_src_hse = (self.cfgr & (1 << 16)) != 0;
        if pll_src_hse {
            if !self.is_hse_ready() {
                return None;
            }

            let hse_div2 = (self.cfgr & (1 << 17)) != 0;
            if hse_div2 {
                Some(Self::HSE_FREQ / 2)
            } else {
                Some(Self::HSE_FREQ)
            }
        } else if self.is_hsi_ready() {
            Some(Self::HSI_FREQ / 2)
        } else {
            None
        }
    }

    fn pll_multiplier(&self) -> u32 {
        let mul_bits = ((self.cfgr >> 18) & 0x0f) as u8;
        match mul_bits {
            0b0000..=0b1101 => (mul_bits as u32) + 2,
            0b1110 | 0b1111 => 16,
            _ => 2,
        }
    }

    fn pll_clock(&self) -> Option<u32> {
        if !self.is_pll_ready() {
            return None;
        }

        let pll_in = self.pll_input_clock()?;
        Some(pll_in.saturating_mul(self.pll_multiplier()))
    }

    fn compute_sys_clock(&self) -> u32 {
        let clock_src = (self.cfgr >> 2) & 0x03;
        match clock_src {
            0b00 => {
                if self.is_hsi_ready() {
                    Self::HSI_FREQ
                } else {
                    0
                }
            }
            0b01 => {
                if self.is_hse_ready() {
                    Self::HSE_FREQ
                } else if self.is_hsi_ready() {
                    Self::HSI_FREQ
                } else {
                    0
                }
            }
            0b10 => {
                if let Some(pll_clk) = self.pll_clock() {
                    pll_clk
                } else if self.is_hse_ready() {
                    Self::HSE_FREQ
                } else if self.is_hsi_ready() {
                    Self::HSI_FREQ
                } else {
                    0
                }
            }
            _ => {
                if self.is_hsi_ready() {
                    Self::HSI_FREQ
                } else {
                    0
                }
            }
        }
    }

    fn update_sys_clock(&self) {
        let freq = self.compute_sys_clock();
        self.sys_clock.store(freq, Ordering::Relaxed);
    }
}

impl Peripheral for Rcc {
    fn start(&self) -> u32 {
        self.start
    }

    fn end(&self) -> u32 {
        self.end
    }

    fn read(&self, addr: u32) -> u32 {
        let offset = addr & 0xFF;
        match offset {
            0x00 => self.cr,
            0x04 => self.cfgr,
            0x08 => self.cir,
            0x0C => self.apb2rstr,
            0x10 => self.ahb1rstr,
            0x14 => self.ahb1enr,
            0x18 => self.apb2enr,
            0x1c => self.apb1enr,
            0x20 => self.bdcr,
            0x24 => self.csr,
            // 可根据需要继续添加
            _ => 0,
        }
    }

    fn write(&mut self, addr: u32, val: u32) {
        let offset = addr & 0xFF;
        match offset {
            0x00 => {
                let mut new_val = val;
                // HSI ON(Bit 0) -> HSI RDY(Bit 1)
                if (new_val & (1 << 0)) != 0 {
                    new_val |= 1 << 1;
                } else {
                    new_val &= !(1 << 1);
                }

                // HSE ON(Bit 16) -> HSE RDY(Bit 17)
                if (new_val & (1 << 16)) != 0 {
                    new_val |= 1 << 17;
                } else {
                    new_val &= !(1 << 17);
                }

                // PLL ON(Bit 24) -> PLL RDY(Bit 25)
                if (new_val & (1 << 24)) != 0 {
                    new_val |= 1 << 25;
                } else {
                    new_val &= !(1 << 25);
                }
                self.cr = new_val;
                self.update_sys_clock();
            }
            0x04 => {
                let mut new_val = val;
                let sw = new_val & 0x03;
                new_val &= !(0x03 << 2);
                new_val |= sw << 2;
                self.cfgr = new_val;

                self.update_sys_clock();
            }
            0x08 => self.cir = val,
            0x0C => self.apb2rstr = val,
            0x10 => self.ahb1rstr = val,
            0x14 => self.ahb1enr = val,
            0x18 => self.apb2enr = val,
            0x1c => self.apb1enr = val,
            0x20 => self.bdcr = val,
            0x24 => self.csr = val,
            // 可根据需要继续添加
            _ => {}
        }
    }

    fn tick(&mut self) {
        // RCC 通常不需要周期性 tick 行为
    }

    #[inline(always)]
    fn needs_tick(&self) -> bool {
        false
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rcc_hsi_enable_sets_ready_and_clock() {
        let freq = Arc::new(AtomicU32::new(0));
        let mut rcc = Rcc::new(0x4002_0000, 0x4002_1024, freq.clone());

        rcc.write(0x4002_0000, 1 << 0);
        rcc.write(0x4002_0004, 0);

        assert_ne!(rcc.read(0x4002_0000) & (1 << 1), 0);
        assert_eq!(freq.load(Ordering::Relaxed), 16_000_000);
    }

    #[test]
    fn rcc_hse_enable_sets_ready_and_clock() {
        let freq = Arc::new(AtomicU32::new(0));
        let mut rcc = Rcc::new(0x4002_0000, 0x4002_1024, freq.clone());

        rcc.write(0x4002_0000, (1 << 0) | (1 << 16));
        rcc.write(0x4002_0004, 0b01);

        assert_ne!(rcc.read(0x4002_0000) & (1 << 17), 0);
        assert_eq!(freq.load(Ordering::Relaxed), 8_000_000);
    }

    #[test]
    fn rcc_pll_clock_selection_updates_frequency() {
        let freq = Arc::new(AtomicU32::new(0));
        let mut rcc = Rcc::new(0x4002_0000, 0x4002_1024, freq.clone());

        rcc.write(0x4002_0000, (1 << 0) | (1 << 24));
        let pll_x4_cfg = (0b10) | (0b0010 << 18);
        rcc.write(0x4002_0004, pll_x4_cfg);

        assert_ne!(rcc.read(0x4002_0000) & (1 << 25), 0);
        assert_eq!(freq.load(Ordering::Relaxed), 32_000_000);
    }
}
