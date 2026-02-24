use crate::peripheral::peripheral::Peripheral;

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
}

impl Peripheral for Gpio {
    fn start(&self) -> u32 {
        self.start
    }

    fn end(&self) -> u32 {
        self.end
    }
    fn read(&mut self, addr: u32) -> u32 {
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
            0x0C => self.odr = val,
            0x10 => {
                self.bsrr = val;
                // Bits 0-15: set; Bits 16-31: reset
                let set = val & 0xFFFF;
                let reset = (val >> 16) & 0xFFFF;
                self.odr = (self.odr & !reset) | set;
            }
            0x14 => {
                self.brr = val;
                // Bits 0-15: reset
                let reset = val & 0xFFFF;
                self.odr &= !reset;
            }
            0x18 => {
                self.lckr = val;
            }
            _ => {}
        }
    }

    fn tick(&mut self) {
        // if (self.odr >> 13) & 1 != 0 {
        //     println!("gpio c13  on");
        // } else {
        //     if (self.odr >> 13) & 1 == 0 {
        //         println!("gpio c13  off");
        //     }
        // }
    }
}
