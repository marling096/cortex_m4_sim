use core::panic;

use crate::peripheral::peripheral::Peripheral;

pub struct Bus {
    pub flash: Vec<u8>,
    pub ram: Vec<u8>,
    peripherals: Vec<(u32, u32, Box<dyn Peripheral>)>,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            flash: vec![0; 512 * 1024], // 512KB
            ram: vec![0; 128 * 1024],   // 128KB
            peripherals: Vec::new(),
        }
    }

    pub fn register_peripheral(&mut self, dev: Box<dyn Peripheral>) {
        self.peripherals.push((dev.start(), dev.end(), dev));
    }

    pub fn read8(&mut self, addr: u32) -> u8 {
        // Check peripherals
        for (start, end, dev) in &mut self.peripherals {
            if addr >= *start && addr <= *end {
                return dev.read8(addr);
            }
        }

        panic!("Unmapped read8 at address {:08X}", addr);
    }

    pub fn read16(&mut self, addr: u32) -> u16 {
        for (start, end, dev) in &mut self.peripherals {
            if addr >= *start && addr <= *end {
                return dev.read16(addr);
            }
        }

        // Little endian
        let b0 = self.read8(addr) as u16;
        let b1 = self.read8(addr + 1) as u16;
        b0 | (b1 << 8)
    }

    pub fn read32(&mut self, addr: u32) -> u32 {
        // Check peripherals first for 32-bit optimization
        for (start, end, dev) in &mut self.peripherals {
            if addr >= *start && addr <= *end {
                // Assuming aligned access for peripherals
                return dev.read(addr);
            }
        }

        // Memory or unmapped
        let b0 = self.read8(addr) as u32;
        let b1 = self.read8(addr + 1) as u32;
        let b2 = self.read8(addr + 2) as u32;
        let b3 = self.read8(addr + 3) as u32;
        b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)
    }

    pub fn write8(&mut self, addr: u32, val: u8) {
        for (start, end, dev) in &mut self.peripherals {
            if addr >= *start && addr <= *end {
                dev.write8(addr, val);
                return;
            }
        }
    }

    pub fn write16(&mut self, addr: u32, val: u16) {
        for (start, end, dev) in &mut self.peripherals {
            if addr >= *start && addr <= *end {
                dev.write16(addr, val);
                return;
            }
        }

        let bytes = val.to_le_bytes();
        self.write8(addr, bytes[0]);
        self.write8(addr + 1, bytes[1]);
    }

    pub fn write32(&mut self, addr: u32, val: u32) {
        for (start, end, dev) in &mut self.peripherals {
            if addr >= *start && addr <= *end {
                dev.write(addr, val);
                return;
            }
        }

        let bytes = val.to_le_bytes();
        self.write8(addr, bytes[0]);
        self.write8(addr + 1, bytes[1]);
        self.write8(addr + 2, bytes[2]);
        self.write8(addr + 3, bytes[3]);
    }

    pub fn tick(&mut self) {
        for (_, _, dev) in &mut self.peripherals {
            dev.tick();
            
        }
    }
}
