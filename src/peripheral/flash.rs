use crate::peripheral::peripheral::Peripheral;
use std::any::Any;

#[derive(Default)]
pub struct Flash {
    pub start: u32,
    pub end: u32,
    
    pub acr: u32,       // Access control register, offset 0x00
    // keyr is write-only, used for unlocking, handled via logic
    pub optkeyr: u32,   // Option byte key register, offset 0x08
    pub sr: u32,        // Status register, offset 0x0C
    pub cr: u32,        // Control register, offset 0x10
    pub ar: u32,        // Address register, offset 0x14
    pub obr: u32,       // Option byte register, offset 0x18
    pub wrpr: u32,      // Write protection register, offset 0x1C

    key_step: u8,       // Internal state for unlocking sequence
}

impl Flash {
    pub fn new(start: u32, end: u32) -> Self {
        Self {
            start,
            end,
            cr: 0x0000_0080, // Reset value: LOCK bit is set (Bit 7)
            ..Default::default()
        }
    }
}

impl Peripheral for Flash {
    fn start(&self) -> u32 {
        self.start
    }

    fn end(&self) -> u32 {
        self.end
    }

    fn read(&self, addr: u32) -> u32 {
        let offset = addr - self.start;
        match offset {
            0x00 => self.acr,
            0x04 => 0, // KEYR is write-only
            0x08 => 0, // OPTKEYR is write-only
            0x0C => self.sr,
            0x10 => self.cr,
            0x14 => self.ar,
            0x18 => self.obr,
            0x1C => self.wrpr,
            _ => 0,
        }
    }

    fn write(&mut self, addr: u32, val: u32) {
        let offset = addr - self.start;
        // Lock bit mask. STM32F1 is bit 7 (0x80).
        let lock_bit = 0x80;

        match offset {
            0x00 => self.acr = val,
            0x04 => {
                // KEYR: Flash key register
                if (self.cr & lock_bit) != 0 { // If locked
                    if self.key_step == 0 && val == 0x45670123 {
                        self.key_step = 1;
                    } else if self.key_step == 1 && val == 0xCDEF89AB {
                        self.cr &= !lock_bit; // Unlock CR
                        self.key_step = 0;
                    } else {
                        self.key_step = 0;
                        self.sr |= 1 << 4; // WRPERR?
                    }
                }
            },
            0x08 => self.optkeyr = val,
            0x0C => {
                // SR: Status register, bits are cleared by writing 1
                self.sr &= !val;
            },
            0x10 => {
                // CR: Control register
                if (self.cr & lock_bit) == 0 { // If Unlocked
                    if (val & lock_bit) != 0 {
                        self.cr = val | lock_bit; // Ensure lock bit is set
                        self.key_step = 0;
                    } else {
                        self.cr = val;
                    }
                }
            },
            0x14 => self.ar = val,
            0x18 => {}, // OBR is Read-only
            0x1C => {}, // WRPR is Read-only
            _ => {},
        }
    }

    fn tick(&mut self) {
        // Handle flash operations delay if needed
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
    fn flash_starts_locked() {
        let mut flash = Flash::new(0x4002_2000, 0x4002_201C);
        assert_ne!(flash.read(0x4002_2010) & 0x80, 0);
    }

    #[test]
    fn flash_unlock_sequence_clears_lock_bit() {
        let mut flash = Flash::new(0x4002_2000, 0x4002_201C);

        flash.write(0x4002_2004, 0x4567_0123);
        flash.write(0x4002_2004, 0xCDEF_89AB);

        assert_eq!(flash.read(0x4002_2010) & 0x80, 0);
    }

    #[test]
    fn flash_locked_cr_write_is_ignored() {
        let mut flash = Flash::new(0x4002_2000, 0x4002_201C);
        let original = flash.read(0x4002_2010);

        flash.write(0x4002_2010, 0x0000_0001);

        assert_eq!(flash.read(0x4002_2010), original);
    }

    #[test]
    fn flash_unlocked_cr_write_applies() {
        let mut flash = Flash::new(0x4002_2000, 0x4002_201C);
        flash.write(0x4002_2004, 0x4567_0123);
        flash.write(0x4002_2004, 0xCDEF_89AB);

        flash.write(0x4002_2010, 0x0000_0012);

        assert_eq!(flash.read(0x4002_2010), 0x0000_0012);
    }
}
