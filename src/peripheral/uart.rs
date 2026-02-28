use crate::peripheral::peripheral::Peripheral;
use std::any::Any;
use std::collections::VecDeque;
use std::io::{self, Write};

const SR_RXNE: u32 = 1 << 5;
const SR_TC: u32 = 1 << 6;
const SR_TXE: u32 = 1 << 7;
const CR1_UE: u32 = 1 << 13;
const CR1_TE: u32 = 1 << 3;

#[derive(Default)]
pub struct Uart {
    pub start: u32,
    pub end: u32,

    pub sr: u32,
    pub dr: u32,
    pub brr: u32,
    pub cr1: u32,

    tx_bytes: Vec<u8>,
    tx_pending: VecDeque<u8>,
    tx_line: bool,
    tx_shift: u16,
    tx_bits_left: u8,

    rx_line_prev: bool,
    rx_state: RxState,
    rx_fifo: VecDeque<u8>,
}

#[derive(Clone, Copy, Default)]
enum RxState {
    #[default]
    Idle,
    Data { bit_idx: u8, byte: u8 },
    Stop { byte: u8 },
}

impl Uart {
    pub fn new(start: u32, end: u32) -> Self {
        Self {
            start,
            end,
            sr: SR_TXE | SR_TC,
            cr1: CR1_UE | CR1_TE,
            tx_line: true,
            rx_line_prev: true,
            ..Default::default()
        }
    }

    #[inline(always)]
    fn tx_enabled(&self) -> bool {
        (self.cr1 & (CR1_UE | CR1_TE)) == (CR1_UE | CR1_TE)
    }

    #[inline(always)]
    fn push_tx_byte(&mut self, byte: u8) {
        if !self.tx_enabled() {
            return;
        }
        println!("[uart] push_tx_byte: 0x{:02X} ('{}')\n", byte, if (0x20..=0x7e).contains(&byte) { byte as char } else { '.' });
        self.tx_bytes.push(byte);
        self.tx_pending.push_back(byte);
        self.dr = byte as u32;
        self.sr &= !SR_TC;
        self.sr |= SR_TXE;

        // print!("{}", byte as char);
        let _ = io::stdout().flush();
    }

    #[inline(always)]
    fn load_next_tx_frame(&mut self) {
        if self.tx_bits_left != 0 {
            return;
        }

        if let Some(byte) = self.tx_pending.pop_front() {
            // 8N1: start(0) + data(LSB first) + stop(1)
            self.tx_shift = (1u16 << 9) | ((byte as u16) << 1);
            self.tx_bits_left = 10;
        } else {
            self.tx_line = true;
            self.sr |= SR_TC;
        }
    }

    #[inline(always)]
    pub fn push_from_gpio(&mut self, byte: u8) {
        self.push_tx_byte(byte);
    }

    #[inline(always)]
    pub fn tx_line_level(&self) -> bool {
        self.tx_line
    }

    #[inline(always)]
    pub fn set_rx_line(&mut self, level: bool) {
        match self.rx_state {
            RxState::Idle => {
                if self.rx_line_prev && !level {
                    self.rx_state = RxState::Data { bit_idx: 0, byte: 0 };
                }
            }
            RxState::Data { bit_idx, mut byte } => {
                if level {
                    byte |= 1 << bit_idx;
                }

                if bit_idx >= 7 {
                    self.rx_state = RxState::Stop { byte };
                } else {
                    self.rx_state = RxState::Data {
                        bit_idx: bit_idx + 1,
                        byte,
                    };
                }
            }
            RxState::Stop { byte } => {
                if level {
                    self.rx_fifo.push_back(byte);
                    self.sr |= SR_RXNE;
                    self.dr = byte as u32;
                    let display = if (0x20..=0x7e).contains(&byte) {
                        byte as char
                    } else {
                        '.'
                    };
                    println!("[usart-rx] byte=0x{byte:02X} ('{display}')");
                }
                self.rx_state = RxState::Idle;
            }
        }
        self.rx_line_prev = level;
    }

    #[inline(always)]
    pub fn take_tx_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.tx_bytes)
    }
}

impl Peripheral for Uart {
    fn start(&self) -> u32 {
        self.start
    }

    fn end(&self) -> u32 {
        self.end
    }

    fn read(&self, addr: u32) -> u32 {
        let offset = addr.wrapping_sub(self.start);
        match offset {
            0x00 => self.sr,
            0x04 => self.dr,
            0x08 => self.brr,
            0x0C => self.cr1,
            _ => 0,
        }
    }

    fn write(&mut self, addr: u32, val: u32) {
        let offset = addr.wrapping_sub(self.start);
        match offset {
            0x00 => {
                // USART SR 在硬件上主要是状态位；RXNE/TXE/TC 等不应被软件直接整寄存器覆盖。
                // 这里忽略写入，避免固件写 SR 时清掉仿真侧置位的接收标志。
                let _ = val;
            }
            0x04 => {
                self.push_tx_byte((val & 0xFF) as u8);
            }
            0x08 => {
                self.brr = val;
            }
            0x0C => {
                self.cr1 = val;
            }
            _ => {}
        }
    }

    fn tick(&mut self) {
        self.load_next_tx_frame();

        if self.tx_bits_left > 0 {
            self.tx_line = (self.tx_shift & 1) != 0;
            self.tx_shift >>= 1;
            self.tx_bits_left -= 1;
            if self.tx_bits_left == 0 {
                self.tx_line = true;
                if self.tx_pending.is_empty() {
                    self.sr |= SR_TC;
                }
            }
        }

        self.sr |= SR_TXE;
    }

    #[inline(always)]
    fn needs_tick(&self) -> bool {
        true
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uart_dr_write_pushes_tx() {
        let mut uart = Uart::new(0x4001_3800, 0x4001_3BFF);
        uart.write(0x4001_3804, b'A' as u32);
        assert_eq!(uart.take_tx_bytes(), vec![b'A']);
    }

    #[test]
    fn uart_gpio_bridge_pushes_tx() {
        let mut uart = Uart::new(0x4001_3800, 0x4001_3BFF);
        uart.push_from_gpio(b'B');
        assert_eq!(uart.take_tx_bytes(), vec![b'B']);
    }
}
