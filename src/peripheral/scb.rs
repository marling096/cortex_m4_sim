
use crate::peripheral::peripheral::Peripheral;
use std::any::Any;

#[derive(Default)]
pub struct Scb {
    pub start: u32,
    pub end: u32,

    pub cpuid: u32, // CPUID 基地址寄存器
    pub icsr: u32,  // 中断控制和状态寄存器
    pub vtor: u32,  // 向量表偏移寄存器
    pub aircr: u32, // 应用程序中断和复位控制寄存器
    pub scr: u32,   // 系统控制寄存器
    pub ccr: u32,   // 配置和控制寄存器
    pub shpr1: u32, // 系统处理程序优先级寄存器 1
    pub shpr2: u32, // 系统处理程序优先级寄存器 2
    pub shpr3: u32, // 系统处理程序优先级寄存器 3
    pub shcsr: u32, // 系统处理程序控制和状态寄存器
    pub cfsr: u32,  // 可配置故障状态寄存器
    pub hfsr: u32,  // 硬故障状态寄存器
    pub bfsr: u32,  // 总线故障状态寄存器 (part of CFSR usually accessed as byte/halfword, but simplified here as u32 register if independent or part of logic)
                    // Note: CFSR (0xE000ED28) comprises:
                    // MMFSR (MemManage) byte 0
                    // BFSR (BusFault) byte 1
                    // UFSR (UsageFault) halfword at offset 2
                    // So we will stick to CFSR.

    pub mmar: u32,  // 存储器管理故障地址寄存器
    pub bfar: u32,  // 总线故障地址寄存器
    pub afsr: u32,  // 辅助故障状态寄存器
}

impl Scb {
    pub fn new(start: u32, end: u32) -> Self {
        Self {
            start,
            end,
            cpuid: 0x410FC241, // Cortex-M4 r0p1
            aircr: 0xFA050000, // Reset value is unpredictable but FA05 in top bits is key. Conventionally initialized or 0.
            // Let's stick to default for others or 0.
            ..Default::default()
        }
    }
}

impl Peripheral for Scb {
    fn start(&self) -> u32 {
        self.start
    }

    fn end(&self) -> u32 {
        self.end
    }

    fn read(&self, addr: u32) -> u32 {
        let offset = addr & 0xFF;
        match offset {
            0x00 => self.cpuid,
            0x04 => self.icsr,
            0x08 => self.vtor,
            0x0C => self.aircr,
            0x10 => self.scr,
            0x14 => self.ccr,
            0x18 => self.shpr1,
            0x1C => self.shpr2,
            0x20 => self.shpr3,
            0x24 => self.shcsr,
            0x28 => self.cfsr,
            0x2C => self.hfsr,
            0x34 => self.mmar,
            0x38 => self.bfar,
            0x3C => self.afsr,
            _ => 0,
        }
    }

    fn write(&mut self, addr: u32, val: u32) {
        let offset = addr & 0xFF;
        match offset {
            0x00 => { /* CPUID is read-only */ }
            0x04 => self.icsr = val, // Note: ICSR has write-to-clear or set bits logic, simplified assignment here
            0x08 => self.vtor = val,
            0x0C => {
                // VECTKEY must be 0x05FA
                if (val & 0xFFFF0000) == 0x05FA0000 {
                    let clear_bits = val & 0x0000FFFF;
                    // Apply changes based on bits (e.g. system reset request)
                    // Currently just storing lower bits for simulation state
                    self.aircr = 0xFA050000 | clear_bits;
                }
            }
            0x10 => self.scr = val,
            0x14 => self.ccr = val,
            0x18 => self.shpr1 = val,
            0x1C => self.shpr2 = val,
            0x20 => self.shpr3 = val,
            0x24 => self.shcsr = val,
            0x28 => self.cfsr = val, // Usually bits are stuck-at or W1C (write 1 to clear)
            0x2C => self.hfsr = val, // W1C
            0x34 => self.mmar = val,
            0x38 => self.bfar = val,
            0x3C => self.afsr = val,
            _ => {}
        }
    }

    fn tick(&mut self) {
        // SCB usually doesn't need periodic tick
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
    fn scb_cpuid_is_read_only() {
        let mut scb = Scb::new(0xE000_ED00, 0xE000_ED3C);
        let cpuid = scb.read(0xE000_ED00);

        scb.write(0xE000_ED00, 0xDEAD_BEEF);

        assert_eq!(scb.read(0xE000_ED00), cpuid);
    }

    #[test]
    fn scb_aircr_requires_vectkey() {
        let mut scb = Scb::new(0xE000_ED00, 0xE000_ED3C);
        let original = scb.read(0xE000_ED0C);

        scb.write(0xE000_ED0C, 0x0000_0005);
        assert_eq!(scb.read(0xE000_ED0C), original);

        scb.write(0xE000_ED0C, 0x05FA_0005);
        assert_eq!(scb.read(0xE000_ED0C), 0xFA05_0005);
    }

    #[test]
    fn scb_general_registers_can_read_write() {
        let mut scb = Scb::new(0xE000_ED00, 0xE000_ED3C);

        scb.write(0xE000_ED08, 0x2000_0000);
        scb.write(0xE000_ED10, 0x0000_0004);

        assert_eq!(scb.read(0xE000_ED08), 0x2000_0000);
        assert_eq!(scb.read(0xE000_ED10), 0x0000_0004);
    }
}
