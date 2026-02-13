use super::peripheral::Peripheral;

const SYST_CSR_ENABLE: u32 = 1 << 0;
// const SYST_csr_TICKINT: u32 = 1 << 1;
// const SYST_csr_CLKSOURCE: u32 = 1 << 2;
const SYST_CSR_COUNTFLAG: u32 = 1 << 16;

#[derive(Default)]
pub struct SysTick {
    pub start: u32,
    pub end: u32,

    pub csr: u32,   // 控制和状态寄存器
    pub rvr: u32,   // 重载值寄存器
    pub cvr: u32,   // 当前值寄存器
    pub calib: u32, // 校准值寄存器
}

impl SysTick {
    pub fn new(start: u32, end: u32) -> Self {
        Self {
            start,
            end,
            csr: 4, // 默认使用处理器时钟
            rvr: 0,
            cvr: 0,
            calib: 0,
        }
    }
    fn Sys_enableflag(&self) -> bool {
        self.csr & SYST_CSR_ENABLE != 0
    }
    fn Sys_get_countflag(&self) -> bool {
        self.csr & SYST_CSR_COUNTFLAG != 0
    }
}

impl Peripheral for SysTick {
    fn start(&self) -> u32 {
        self.start
    }
    fn end(&self) -> u32 {
        self.end
    }

    fn read(&mut self, addr: u32) -> u32 {
        let offset = addr & 0x0F; // Assuming minimal offset space needed
        match offset {
            0x00 => {
                let val = self.csr;
                self.csr &= !SYST_CSR_COUNTFLAG; // 读操作清除 COUNTFLAG
                val
            }
            0x04 => self.rvr,
            0x08 => self.cvr,
            0x0C => self.calib,
            _ => 0,
        }
    }

    fn write(&mut self, addr: u32, val: u32) {
        let offset = addr & 0x0F;
        // println!("Writing to SysTick at offset 0x{:02X} value 0x{:08X}", offset, val);
        match offset {
            0x00 => {
                // COUNTFLAG (bit 16) 只有只读属性，写入不仅无效可能还需要注意
                // 这里只允许修改控制位
                // println!("Writing to CSR: {}", val);
                self.csr = val;
                if !self.Sys_enableflag() {
                    // 禁用时清除 CVR
                    self.cvr = 0;
                } else {
                    self.cvr = self.rvr; // 启用时 CVR 自动加载
                }
                // print!("SysTick CSR after write: 0x{:08X}\n", self.csr);
            }
            0x04 => {
                self.rvr = val & 0xFFFFFF; // 24位
            }
            0x08 => {
                // 写入任意值都会清除当前值和 COUNTFLAG
                // println!("Writing to CVR clears it and COUNTFLAG");
                self.cvr = 0;
                self.csr &= !SYST_CSR_COUNTFLAG;
            }
            _ => {}
        }
    }

    fn tick(&mut self) {

        if self.Sys_enableflag() {
            self.cvr -= 1;
            println!("systick tick_val {}", self.cvr);
        }

        if self.cvr == 0 {
            self.csr |= SYST_CSR_COUNTFLAG;
            println!("set conutflag");
            if self.Sys_enableflag() {
                self.cvr = self.rvr; // 自动重装载
            }
        }
    }
}
