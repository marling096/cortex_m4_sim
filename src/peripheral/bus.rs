use core::panic;

use crate::peripheral::peripheral::Peripheral;

pub struct Bus {
    peripherals: Vec<(u32, u32, Box<dyn Peripheral>)>,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            peripherals: Vec::new(),
        }
    }

    pub fn register_peripheral(&mut self, dev: Box<dyn Peripheral>) {
        self.peripherals.push((dev.start(), dev.end(), dev));
    }

    pub fn read32(&mut self, addr: u32) -> u32 {
        for (start, end, dev) in &mut self.peripherals {
            if addr >= *start && addr <= *end {
                return dev.read(addr);
            }
        }
        panic!("Unmapped read32 at address {:08X}", addr);
    }

    pub fn write32(&mut self, addr: u32, val: u32) {
        for (start, end, dev) in &mut self.peripherals {
            if addr >= *start && addr <= *end {
                dev.write(addr, val);
                return;
            }
        }
        panic!("Unmapped write32 at address {:08X}", addr);
    }

    pub fn tick(&mut self) {
        for (_, _, dev) in &mut self.peripherals {
            dev.tick();
            
        }
    }
}
