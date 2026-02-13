pub trait Peripheral: Send {

    fn start(&self) -> u32;
    fn end(&self) -> u32;

    // 读取外设寄存器
    fn read(&mut self, addr: u32) -> u32;
    // 写入外设寄存器
    fn write(&mut self, addr: u32, val: u32);
    // 模拟时钟步进（比如定时器计数、串口发送数据）
    fn tick(&mut self);

}
