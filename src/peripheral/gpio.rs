use crate::peripheral::peripheral::Peripheral;

#[derive(Default)]
pub struct Gpio {
    pub start: u32,
    pub end: u32,

    pub moder: u32,
    pub otyper: u32,
    pub ospeedr: u32,
    pub pupdr: u32,
    pub idr: u32,
    pub odr: u32,
    pub lckr: u32,
    pub afrl: u32,
    pub afrh: u32,
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

    /// Helper to update IDR based on ODR for output pins,
    /// or for external simulation to call to set input pins.
    pub fn set_pin_level(&mut self, pin: usize, high: bool) {
        if high {
            self.idr |= 1 << pin;
        } else {
            self.idr &= !(1 << pin);
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
            0x00 => self.moder,
            0x04 => self.otyper,
            0x08 => self.ospeedr,
            0x0C => self.pupdr,
            0x10 => self.idr,
            0x14 => self.odr,
            0x18 => 0, // BSRR is write-only
            0x1C => self.lckr,
            0x20 => self.afrl,
            0x24 => self.afrh,
            _ => 0,
        }
    }

    fn write(&mut self, addr: u32, val: u32) {
        let offset = addr & 0xFF;
        match offset {
            0x00 => self.moder = val,
            0x04 => self.otyper = val,
            0x08 => self.ospeedr = val,
            0x0C => self.pupdr = val,
            0x10 => {
                // IDR is typically read-only from software,
                // but we might allow writing if we treat this as a raw state set for simulation.
                // However, standard hardware ignores writes to IDR.
                // For safety/realism: ignore.
            }
            0x14 => self.odr = val,
            0x18 => {
                // BSRR: Bit Set/Reset Register
                // Low 16 bits: Set
                // High 16 bits: Reset
                let set_mask = val & 0xFFFF;
                let reset_mask = (val >> 16) & 0xFFFF;

                // If both are set, BSx (set) has priority in some docs, or atomic apply.
                // Usually: ODR = (ODR & !reset_mask) | set_mask;
                // But specifically for each bit: if BSy is 1, set it. if BRy is 1, reset it.
                // If both BSy and BRy are 1, BSy has priority.

                // Perform Reset first (if BS has priority)
                self.odr &= !reset_mask;
                // Perform Set
                self.odr |= set_mask;
            }
            0x1C => self.lckr = val,
            0x20 => self.afrl = val,
            0x24 => self.afrh = val,
            _ => {}
        }
    }

    fn tick(&mut self) {
        println!("GPIOC13 status is {}", (self.odr >> 13) & 1);
    }
}
