pub trait CpuContext {
    fn read_reg(&self, r: u32) -> u32;
    fn write_reg(&mut self, r: u32, v: u32);

    fn read_mem(&self, addr: u32) -> u32;
    fn write_mem(&mut self, addr: u32, v: u32);

    fn read_gpr(&self, r: u32) -> u32;
    fn write_gpr(&mut self, r: u32, v: u32);

    fn read_msp(&self, r: u32) -> u32;
    fn write_msp(&mut self, v: u32);

    fn read_psp(&self, r: u32) -> u32;
    fn write_psp(&mut self, v: u32);

    fn read_sp(&self) -> u32;
    fn write_sp(&mut self, v: u32);

    fn read_lr(&self, r: u32) -> u32;
    fn write_lr(&mut self, v: u32);

    fn read_pc(&self) -> u32;
    fn write_pc(&mut self, pc: u32);

    fn read_apsr(&self) -> u32;
    fn write_apsr(&mut self, v: u32);
}
