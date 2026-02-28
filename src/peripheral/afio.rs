use crate::peripheral::peripheral::Peripheral;
use std::any::Any;

const MAPR_USART1_REMAP: u32 = 1 << 2;
const GPIO_BRIDGE_EN: u32 = 1 << 0;

const GPIOA_BASE: u32 = 0x4001_0800;
const GPIOB_BASE: u32 = 0x4001_0C00;

#[derive(Clone, Copy)]
pub struct GpioBridgeConfig {
    pub src_gpio_base: u32,
    pub dst_gpio_base: u32,
    pub src_pin: u8,
    pub dst_pin: u8,
}

#[derive(Default)]
pub struct Afio {
    pub start: u32,
    pub end: u32,

    pub evcr: u32,
    pub mapr: u32,
    pub exticr1: u32,
    pub exticr2: u32,
    pub exticr3: u32,
    pub exticr4: u32,
    pub mapr2: u32,

    pub gpio_bridge_ctrl: u32,
    pub gpio_bridge_src: u32,
    pub gpio_bridge_dst: u32,
    pub gpio_bridge_pins: u32,
}

impl Afio {
    pub fn new(start: u32, end: u32) -> Self {
        Self {
            start,
            end,
            ..Default::default()
        }
    }

    #[inline(always)]
    pub fn usart1_pin_mapping(&self) -> (u32, u8, u8) {
        // STM32F1 AFIO MAPR.USART1_REMAP
        // 0: PA9(TX),  PA10(RX)
        // 1: PB6(TX),  PB7(RX)
        if (self.mapr & MAPR_USART1_REMAP) != 0 {
            (GPIOB_BASE, 7, 6)
        } else {
            (GPIOA_BASE, 10, 9)
        }
    }

    #[inline(always)]
    pub fn gpio_bridge_config(&self) -> Option<GpioBridgeConfig> {
        if (self.gpio_bridge_ctrl & GPIO_BRIDGE_EN) == 0 {
            return None;
        }

        let src_pin = (self.gpio_bridge_pins & 0xFF) as u8;
        let dst_pin = ((self.gpio_bridge_pins >> 8) & 0xFF) as u8;
        if src_pin >= 16 || dst_pin >= 16 {
            return None;
        }

        Some(GpioBridgeConfig {
            src_gpio_base: self.gpio_bridge_src,
            dst_gpio_base: self.gpio_bridge_dst,
            src_pin,
            dst_pin,
        })
    }
}

impl Peripheral for Afio {
    fn start(&self) -> u32 {
        self.start
    }

    fn end(&self) -> u32 {
        self.end
    }

    fn read(&self, addr: u32) -> u32 {
        let offset = addr.wrapping_sub(self.start);
        match offset {
            0x00 => self.evcr,
            0x04 => self.mapr,
            0x08 => self.exticr1,
            0x0C => self.exticr2,
            0x10 => self.exticr3,
            0x14 => self.exticr4,
            0x1C => self.mapr2,
            0x20 => self.gpio_bridge_ctrl,
            0x24 => self.gpio_bridge_src,
            0x28 => self.gpio_bridge_dst,
            0x2C => self.gpio_bridge_pins,
            _ => 0,
        }
    }

    fn write(&mut self, addr: u32, val: u32) {
        let offset = addr.wrapping_sub(self.start);
        match offset {
            0x00 => self.evcr = val,
            0x04 => self.mapr = val,
            0x08 => self.exticr1 = val,
            0x0C => self.exticr2 = val,
            0x10 => self.exticr3 = val,
            0x14 => self.exticr4 = val,
            0x1C => self.mapr2 = val,
            0x20 => self.gpio_bridge_ctrl = val,
            0x24 => self.gpio_bridge_src = val,
            0x28 => self.gpio_bridge_dst = val,
            0x2C => self.gpio_bridge_pins = val,
            _ => {}
        }
    }

    fn tick(&mut self) {}

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
    fn afio_usart1_default_is_port_a() {
        let afio = Afio::new(0x4001_0000, 0x4001_03FF);
        let (gpio_base, rx_pin, tx_pin) = afio.usart1_pin_mapping();
        assert_eq!(gpio_base, GPIOA_BASE);
        assert_eq!(rx_pin, 10);
        assert_eq!(tx_pin, 9);
    }

    #[test]
    fn afio_usart1_remap_to_port_b() {
        let mut afio = Afio::new(0x4001_0000, 0x4001_03FF);
        afio.write(0x4001_0004, MAPR_USART1_REMAP);
        let (gpio_base, rx_pin, tx_pin) = afio.usart1_pin_mapping();
        assert_eq!(gpio_base, GPIOB_BASE);
        assert_eq!(rx_pin, 7);
        assert_eq!(tx_pin, 6);
    }

    #[test]
    fn afio_gpio_bridge_config_decoding() {
        let mut afio = Afio::new(0x4001_0000, 0x4001_03FF);
        afio.write(0x4001_0024, GPIOA_BASE);
        afio.write(0x4001_0028, GPIOB_BASE);
        // src pin=9, dst pin=7
        afio.write(0x4001_002C, 9 | (7 << 8));
        afio.write(0x4001_0020, GPIO_BRIDGE_EN);

        let bridge = afio.gpio_bridge_config().expect("bridge should be enabled");
        assert_eq!(bridge.src_gpio_base, GPIOA_BASE);
        assert_eq!(bridge.dst_gpio_base, GPIOB_BASE);
        assert_eq!(bridge.src_pin, 9);
        assert_eq!(bridge.dst_pin, 7);
    }
}
