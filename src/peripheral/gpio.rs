use crate::peripheral::peripheral::Peripheral;
use std::any::Any;

#[derive(Default)]
pub struct Gpio {
    pub start: u32,
    pub end: u32,

    pub crl: u32,
    pub crh: u32,
    pub idr: u32,
    pub odr: u32,
    pub bsrr: u32,
    pub brr: u32,
    pub lckr: u32,

    // ---- GPIO 引脚变化观测 ----
    /// 上一次读到的 ODR 值（用于边沿检测）
    pub odr_shadow: u32,
    /// 已发生的翻转次数
    pub toggle_count: u64,
    /// 自上次读取后发生变化的引脚位（事件锁存）
    pub changed_pins_latch: u32,
}

// 端口,起始地址 (Base),结束地址 (Boundary),范围大小,挂载总线
// GPIOA,0x4002 0000,0x4002 03FF,1 KB,AHB1
// GPIOB,0x4002 0400,0x4002 07FF,1 KB,AHB1
// GPIOC,0x4002 0800,0x4002 0BFF,1 KB,AHB1
// GPIOD,0x4002 0C00,0x4002 0FFF,1 KB,AHB1

impl Gpio {
    pub fn new(start: u32, end: u32) -> Self {
        Self {
            start,
            end,
            ..Default::default()
        }
    }

    /// 每次 odr 被改写后调用：检测指定引脚是否发生边沿跳变，若有则打印。
    /// `pin_mask`：关注的引脚位掩码（可同时监视多个引脚）
    #[inline(always)]
    fn notify_odr_change(&mut self, pin_mask: u32) {
        let changed = (self.odr ^ self.odr_shadow) & pin_mask;
        if changed == 0 {
            return;
        }
        self.changed_pins_latch |= changed;
        // 对每个变化的引脚逐位报告
        let mut bits = changed;
        while bits != 0 {
            let pin = bits.trailing_zeros();
            let new_level = (self.odr >> pin) & 1;
            self.toggle_count += 1;
            println!(
                "[GPIO 0x{:08X}] pin {:>2} -> {} (toggle #{})",
                self.start,
                pin,
                if new_level == 1 { "HIGH" } else { "LOW " },
                self.toggle_count,
            );
            bits &= bits - 1;
        }
        self.odr_shadow = self.odr;
    }

    #[inline(always)]
    pub fn take_changed_pins(&mut self) -> u32 {
        let changed = self.changed_pins_latch;
        self.changed_pins_latch = 0;
        changed
    }

    #[inline(always)]
    pub fn odr_value(&self) -> u32 {
        self.odr
    }

    #[inline(always)]
    pub fn read_odr_pin(&self, pin: u8) -> bool {
        ((self.odr >> pin) & 1) != 0
    }

    #[inline(always)]
    pub fn read_idr_pin(&self, pin: u8) -> bool {
        ((self.idr >> pin) & 1) != 0
    }

    #[inline(always)]
    fn pin_cfg_nibble(&self, pin: u8) -> u32 {
        if pin < 8 {
            (self.crl >> (pin * 4)) & 0xF
        } else {
            (self.crh >> ((pin - 8) * 4)) & 0xF
        }
    }

    /// STM32F1: MODE=00 代表输入模式
    #[inline(always)]
    pub fn pin_is_input_mode(&self, pin: u8) -> bool {
        (self.pin_cfg_nibble(pin) & 0b0011) == 0
    }

    /// STM32F1: MODE!=00 代表输出模式
    #[inline(always)]
    pub fn pin_is_output_mode(&self, pin: u8) -> bool {
        (self.pin_cfg_nibble(pin) & 0b0011) != 0
    }

    /// STM32F1: MODE!=00 且 CNF=10 代表复用推挽输出（常用于 USART TX）
    #[inline(always)]
    pub fn pin_is_alt_push_pull(&self, pin: u8) -> bool {
        let nibble = self.pin_cfg_nibble(pin);
        let mode = nibble & 0b0011;
        let cnf = (nibble >> 2) & 0b0011;
        mode != 0 && cnf == 0b10
    }

    #[inline(always)]
    pub fn set_odr_pin(&mut self, pin: u8, level: bool) {
        let mask = 1u32 << pin;
        if level {
            self.odr |= mask;
        } else {
            self.odr &= !mask;
        }
    }

    #[inline(always)]
    pub fn set_idr_pin(&mut self, pin: u8, level: bool) {
        let mask = 1u32 << pin;
        if level {
            self.idr |= mask;
        } else {
            self.idr &= !mask;
        }
    }
}

impl Peripheral for Gpio {
    fn start(&self) -> u32 {
        self.start
    }

    fn end(&self) -> u32 {
        self.end
    }
    fn read(&self, addr: u32) -> u32 {
        let offset = addr & 0xFF;
        match offset {
            0x00 => self.crl,
            0x04 => self.crh,
            0x08 => self.idr,
            0x0C => self.odr,
            0x10 => 0, // BSRR is write-only
            0x14 => 0, // BRR is write-only
            0x18 => self.lckr,

            _ => 0,
        }
    }

    fn write(&mut self, addr: u32, val: u32) {
        let offset = addr & 0xFF;
        match offset {
            0x00 => self.crl = val,
            0x04 => self.crh = val,
            0x08 => { /* IDR is read-only */ }
            0x0C => {
                self.odr = val;
                // self.notify_odr_change(1 << 13);
            }
            0x10 => {
                self.bsrr = val;
                // Bits 0-15: set; Bits 16-31: reset
                let set = val & 0xFFFF;
                let reset = (val >> 16) & 0xFFFF;
                self.odr = (self.odr & !reset) | set;
                // self.notify_odr_change(1 << 13);
            }
            0x14 => {
                self.brr = val;
                // Bits 0-15: reset
                let reset = val & 0xFFFF;
                self.odr &= !reset;
                // self.notify_odr_change(1 << 13);
            }
            0x18 => {
                self.lckr = val;
            }
            _ => {}
        }
    }

    fn tick(&mut self) {
        // GPIO 本身无需时钟驱动，状态变化由 write() 中的边沿检测实时上报
    }

    #[inline(always)]
    fn needs_tick(&self) -> bool {
        false // GPIO 不需要参与每周期 tick，减少仿真开销
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpio_basic_register_read_write() {
        let mut gpio = Gpio::new(0x4001_1000, 0x4001_13FF);

        gpio.write(0x4001_1000, 0x1122_3344);
        gpio.write(0x4001_1004, 0x5566_7788);
        gpio.write(0x4001_100C, 0x0000_00AA);

        assert_eq!(gpio.read(0x4001_1000), 0x1122_3344);
        assert_eq!(gpio.read(0x4001_1004), 0x5566_7788);
        assert_eq!(gpio.read(0x4001_100C), 0x0000_00AA);
    }

    #[test]
    fn gpio_bsrr_and_brr_update_odr() {
        let mut gpio = Gpio::new(0x4001_1000, 0x4001_13FF);

        gpio.write(0x4001_100C, 0x0000_0000);
        gpio.write(0x4001_1010, 0x0000_0003);
        assert_eq!(gpio.odr & 0x3, 0x3);

        gpio.write(0x4001_1010, 0x0003_0000);
        assert_eq!(gpio.odr & 0x3, 0x0);

        gpio.write(0x4001_1014, 0x0000_0002);
        assert_eq!(gpio.odr & 0x2, 0x0);
    }
}
