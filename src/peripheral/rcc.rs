
use crate::peripheral::peripheral::Peripheral;

#[derive(Default)]
pub struct Rcc {
	pub start: u32,
	pub end: u32,

	pub cr: u32,        // 时钟控制寄存器
	pub cfgr: u32,      // 时钟配置寄存器
	pub cir: u32,       // 时钟中断寄存器
	pub ahb1rstr: u32,  // AHB1 外设复位寄存器
	pub apb1rstr: u32,  // APB1 外设复位寄存器
    pub apb2rstr: u32,  // APB2 外设复位寄存器
	pub ahb1enr: u32,   // AHB1 外设时钟使能寄存器
	pub apb1enr: u32,   // APB1 外设时钟使能寄存器\
    pub apb2enr: u32,   // APB2 外设时钟使能寄存器
    pub bdcr: u32,      // 备份域控制寄存器
    pub csr: u32,       // 控制/状态寄存器
	// 可根据需要继续添加其他 RCC 寄存器
}

impl Rcc {
	pub fn new(start: u32, end: u32) -> Self {
		Self {
			start,
			end,
			..Default::default()
		}
	}
}

impl Peripheral for Rcc {
	fn start(&self) -> u32 {
		self.start
	}

	fn end(&self) -> u32 {
		self.end
	}

	fn read(&mut self, addr: u32) -> u32 {
		let offset = addr & 0xFF;
		match offset {
			0x00 => self.cr,
			0x04 => self.cfgr,
			0x08 => self.cir,
			0x0C => self.apb2rstr,
			0x10 => self.ahb1rstr,
            0x14 => self.ahb1enr,
            0x18 => self.apb2enr,
            0x1c => self.apb1enr,
			0x20 => self.bdcr,
			0x24 => self.csr,
			// 可根据需要继续添加
			_ => 0,
		}
	}

	fn write(&mut self, addr: u32, val: u32) {
		let offset = addr & 0xFF;
		match offset {
			0x00 => {
				let mut new_val = val;
				// HSI ON(Bit 0) -> HSI RDY(Bit 1)
				if (new_val & (1 << 0)) != 0 {
					new_val |= 1 << 1;
				} else {
					new_val &= !(1 << 1);
				}

				// HSE ON(Bit 16) -> HSE RDY(Bit 17)
				if (new_val & (1 << 16)) != 0 {
					new_val |= 1 << 17;
				} else {
					new_val &= !(1 << 17);
				}

				// PLL ON(Bit 24) -> PLL RDY(Bit 25)
				if (new_val & (1 << 24)) != 0 {
					new_val |= 1 << 25;
				} else {
					new_val &= !(1 << 25);
				}
				self.cr = new_val;
				println!("cr_val {}" , self.cr);
			}
			0x04 => {
				let mut new_val = val;
				let sw = new_val & 0x03;
				new_val &= !(0x03 << 2);
				new_val |= sw << 2;
				self.cfgr = new_val;
			}
			0x08 => self.cir = val,
			0x0C => self.apb2rstr = val,
			0x10 => self.ahb1rstr = val,
			0x14 => self.ahb1enr = val,
			0x18 => self.apb2enr = val,
			0x1c => self.apb1enr = val,
			0x20 => self.bdcr = val,
			0x24 => self.csr = val,
			// 可根据需要继续添加
			_ => {}
		}
	}

	fn tick(&mut self) {
		// RCC 通常不需要周期性 tick 行为
	}
}
