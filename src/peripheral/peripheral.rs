pub trait Peripheral: Send {

    fn start(&self) -> u32;
    fn end(&self) -> u32;

    // 读取外设寄存器
    fn read(&mut self, addr: u32) -> u32;
    // 写入外设寄存器
    fn write(&mut self, addr: u32, val: u32);
    // 模拟时钟步进（比如定时器计数、串口发送数据）
    fn tick(&mut self);

    fn read8(&mut self, addr: u32) -> u8 {
        let val = self.read(addr & !3);
        ((val >> ((addr & 3) * 8)) & 0xFF) as u8
    }

    fn read16(&mut self, addr: u32) -> u16 {
        let b0 = self.read8(addr) as u16;
        let b1 = self.read8(addr + 1) as u16;
        b0 | (b1 << 8)
    }

    fn write8(&mut self, addr: u32, val: u8) {
        let aligned = addr & !3;
        let current = self.read(aligned);
        let shift = (addr & 3) * 8;
        let mask = 0xFF << shift;
        let new_val = (current & !mask) | ((val as u32) << shift);
        self.write(aligned, new_val);
    }

    fn write16(&mut self, addr: u32, val: u16) {
        let bytes = val.to_le_bytes();
        self.write8(addr, bytes[0]);
        self.write8(addr + 1, bytes[1]);
    }
}
