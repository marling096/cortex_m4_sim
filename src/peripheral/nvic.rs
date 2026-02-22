use crate::peripheral::peripheral::Peripheral;

pub struct Nvic {
    pub start: u32,
    pub end: u32,

    pub iser: [u32; 8], // Interrupt Set-Enable Registers
    pub ispr: [u32; 8], // Interrupt Set-Pending Registers
    pub iabr: [u32; 8], // Interrupt Active Bit Registers
    pub ipr: [u32; 60], // Interrupt Priority Registers
}

impl Default for Nvic {
    fn default() -> Self {
        Self {
            start: 0,
            end: 0,
            iser: [0; 8],
            ispr: [0; 8],
            iabr: [0; 8],
            ipr: [0; 60],
        }
    }
}

impl Nvic {
    pub fn new(start: u32, end: u32) -> Self {
        Self {
            start,
            end,
            ..Default::default()
        }
    }
}

impl Peripheral for Nvic {
    fn start(&self) -> u32 {
        self.start
    }

    fn end(&self) -> u32 {
        self.end
    }

    fn read(&mut self, addr: u32) -> u32 {
        let offset = addr - self.start;
        match offset {
            0x100..=0x11C => {
                let index = ((offset - 0x100) / 4) as usize;
                if index < 8 { self.iser[index] } else { 0 }
            }
            0x180..=0x19C => {
                // ICER reads return the current enabled state (same as ISER)
                let index = ((offset - 0x180) / 4) as usize;
                if index < 8 { self.iser[index] } else { 0 }
            }
            0x200..=0x21C => {
                let index = ((offset - 0x200) / 4) as usize;
                if index < 8 { self.ispr[index] } else { 0 }
            }
            0x280..=0x29C => {
                // ICPR reads return the current pending state (same as ISPR)
                let index = ((offset - 0x280) / 4) as usize;
                if index < 8 { self.ispr[index] } else { 0 }
            }
            0x300..=0x31C => {
                let index = ((offset - 0x300) / 4) as usize;
                if index < 8 { self.iabr[index] } else { 0 }
            }
            0x400..=0x4EC => {
                let index = ((offset - 0x400) / 4) as usize;
                if index < 60 { self.ipr[index] } else { 0 }
            }
            _ => 0,
        }
    }

    fn write(&mut self, addr: u32, val: u32) {
        let offset = addr - self.start;
        match offset {
            0x100..=0x11C => {
                let index = ((offset - 0x100) / 4) as usize;
                // Write 1 to enable interrupt
                if index < 8 { self.iser[index] |= val; }
            }
            0x180..=0x19C => {
                let index = ((offset - 0x180) / 4) as usize;
                // Write 1 to clear enable (disable interrupt)
                if index < 8 { self.iser[index] &= !val; }
            }
            0x200..=0x21C => {
                let index = ((offset - 0x200) / 4) as usize;
                // Write 1 to set pending
                if index < 8 { self.ispr[index] |= val; }
            }
            0x280..=0x29C => {
                let index = ((offset - 0x280) / 4) as usize;
                // Write 1 to clear pending
                if index < 8 { self.ispr[index] &= !val; }
            }
            0x300..=0x31C => {
                 // IABR is read-only
            }
            0x400..=0x4EC => {
                let index = ((offset - 0x400) / 4) as usize;
                if index < 60 { self.ipr[index] = val; }
            }
            0xE00 => {
                // STIR: Software Trigger Interrupt Register
                let interrupt_id = val & 0x1FF;
                let reg_idx = (interrupt_id / 32) as usize;
                let bit_idx = interrupt_id % 32;
                
                if reg_idx < 8 {
                    self.ispr[reg_idx] |= 1 << bit_idx;
                }
            }
            _ => {}
        }
    }

    fn tick(&mut self) {
    }
}
