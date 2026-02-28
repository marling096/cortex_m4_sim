use super::peripheral::Peripheral;
use std::any::Any;

const NVIC_REGISTER_COUNT: usize = 8;
const NVIC_PRIORITY_COUNT: usize = 240;

pub struct Nvic {
    pub start: u32,
    pub end: u32,

    pub iser: [u32; NVIC_REGISTER_COUNT],
    pub icer: [u32; NVIC_REGISTER_COUNT],
    pub ispr: [u32; NVIC_REGISTER_COUNT],
    pub icpr: [u32; NVIC_REGISTER_COUNT],
    pub iabr: [u32; NVIC_REGISTER_COUNT],
    pub ipr: [u8; NVIC_PRIORITY_COUNT],
    pub stir: u32,
    pub pending_word_bitmap: u32,
    pub active_word_bitmap: u32,
    pub interrupt_event: bool,
}

impl Default for Nvic {
    fn default() -> Self {
        Self {
            start: 0,
            end: 0,
            iser: [0; NVIC_REGISTER_COUNT],
            icer: [0; NVIC_REGISTER_COUNT],
            ispr: [0; NVIC_REGISTER_COUNT],
            icpr: [0; NVIC_REGISTER_COUNT],
            iabr: [0; NVIC_REGISTER_COUNT],
            ipr: [0; NVIC_PRIORITY_COUNT],
            stir: 0,
            pending_word_bitmap: 0,
            active_word_bitmap: 0,
            interrupt_event: false,
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

    #[inline(always)]
    fn decode_index(offset: u32, base: u32) -> Option<usize> {
        if offset < base || offset >= base + (NVIC_REGISTER_COUNT as u32 * 4) {
            return None;
        }
        Some(((offset - base) / 4) as usize)
    }

    #[inline(always)]
    fn read_ipr_word(&self, offset: u32) -> u32 {
        let word_index = ((offset - 0x300) / 4) as usize;
        let byte_index = word_index * 4;
        if byte_index + 3 >= NVIC_PRIORITY_COUNT {
            return 0;
        }

        (self.ipr[byte_index] as u32)
            | ((self.ipr[byte_index + 1] as u32) << 8)
            | ((self.ipr[byte_index + 2] as u32) << 16)
            | ((self.ipr[byte_index + 3] as u32) << 24)
    }

    #[inline(always)]
    fn write_ipr_word(&mut self, offset: u32, val: u32) {
        let word_index = ((offset - 0x300) / 4) as usize;
        let byte_index = word_index * 4;
        if byte_index + 3 >= NVIC_PRIORITY_COUNT {
            return;
        }

        self.ipr[byte_index] = (val & 0xFF) as u8;
        self.ipr[byte_index + 1] = ((val >> 8) & 0xFF) as u8;
        self.ipr[byte_index + 2] = ((val >> 16) & 0xFF) as u8;
        self.ipr[byte_index + 3] = ((val >> 24) & 0xFF) as u8;
    }

    #[inline(always)]
    fn set_pending_by_irq(&mut self, irq_num: u32) {
        let index = (irq_num / 32) as usize;
        if index >= NVIC_REGISTER_COUNT {
            return;
        }
        let bit = irq_num % 32;
        self.ispr[index] |= 1u32 << bit;
        self.sync_mirror_registers(index);
        self.sync_word_bitmaps(index);
    }

    #[inline(always)]
    fn sync_mirror_registers(&mut self, index: usize) {
        self.icer[index] = self.iser[index];
        self.icpr[index] = self.ispr[index];
    }

    #[inline(always)]
    fn sync_word_bitmaps(&mut self, index: usize) {
        let prev_pending = self.pending_word_bitmap;
        let prev_active = self.active_word_bitmap;
        let mask = 1u32 << index;
        if self.ispr[index] != 0 {
            self.pending_word_bitmap |= mask;
        } else {
            self.pending_word_bitmap &= !mask;
        }

        if self.iabr[index] != 0 {
            self.active_word_bitmap |= mask;
        } else {
            self.active_word_bitmap &= !mask;
        }

        if prev_pending != self.pending_word_bitmap || prev_active != self.active_word_bitmap {
            self.interrupt_event = true;
        }
    }

    #[inline(always)]
    fn service_interrupts(&mut self) {
        for index in 0..NVIC_REGISTER_COUNT {
            let activatable = self.ispr[index] & self.iser[index];
            if activatable != 0 {
                self.iabr[index] |= activatable;
                self.ispr[index] &= !activatable;
            }

            self.sync_mirror_registers(index);
            self.sync_word_bitmaps(index);
        }
    }

    pub fn clear_active_irq(&mut self, irq_num: u32) {
        let index = (irq_num / 32) as usize;
        if index >= NVIC_REGISTER_COUNT {
            return;
        }
        let bit = irq_num % 32;
        self.iabr[index] &= !(1u32 << bit);
        self.sync_mirror_registers(index);
        self.sync_word_bitmaps(index);
    }

    pub fn take_interrupt_event(&mut self) -> bool {
        let event = self.interrupt_event;
        self.interrupt_event = false;
        event
    }
}

impl Peripheral for Nvic {
    fn start(&self) -> u32 {
        self.start
    }

    fn end(&self) -> u32 {
        self.end
    }

    fn read(&self, addr: u32) -> u32 {
        let offset = addr.wrapping_sub(self.start);

        if let Some(index) = Self::decode_index(offset, 0x000) {
            return self.iser[index];
        }
        if let Some(index) = Self::decode_index(offset, 0x080) {
            return self.icer[index];
        }
        if let Some(index) = Self::decode_index(offset, 0x100) {
            return self.ispr[index];
        }
        if let Some(index) = Self::decode_index(offset, 0x180) {
            return self.icpr[index];
        }
        if let Some(index) = Self::decode_index(offset, 0x200) {
            return self.iabr[index];
        }
        if (0x300..=0x3EC).contains(&offset) {
            return self.read_ipr_word(offset);
        }
        if offset == 0xE00 {
            return self.stir;
        }

        0
    }

    fn write(&mut self, addr: u32, val: u32) {
        let offset = addr.wrapping_sub(self.start);

        if let Some(index) = Self::decode_index(offset, 0x000) {
            self.iser[index] |= val;
            self.sync_mirror_registers(index);
            self.sync_word_bitmaps(index);
            self.service_interrupts();
            return;
        }
        if let Some(index) = Self::decode_index(offset, 0x080) {
            self.iser[index] &= !val;
            self.sync_mirror_registers(index);
            self.sync_word_bitmaps(index);
            return;
        }
        if let Some(index) = Self::decode_index(offset, 0x100) {
            self.ispr[index] |= val;
            self.sync_mirror_registers(index);
            self.sync_word_bitmaps(index);
            self.service_interrupts();
            return;
        }
        if let Some(index) = Self::decode_index(offset, 0x180) {
            self.ispr[index] &= !val;
            self.sync_mirror_registers(index);
            self.sync_word_bitmaps(index);
            return;
        }
        if (0x300..=0x3EC).contains(&offset) {
            self.write_ipr_word(offset, val);
            self.interrupt_event = true;
            return;
        }
        if offset == 0xE00 {
            self.stir = val & 0x1FF;
            self.set_pending_by_irq(self.stir);
            self.service_interrupts();
        }
    }

    fn tick(&mut self) {
        self.service_interrupts();
    }

    #[inline(always)]
    fn needs_tick(&self) -> bool {
        false
    }

    #[inline(always)]
    fn interrupt_event_pending(&self) -> bool {
        self.interrupt_event
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hint::black_box;
    use std::time::Instant;

    fn report_perf(name: &str, iterations: u64, elapsed: std::time::Duration) {
        let total_ns = elapsed.as_nanos() as f64;
        let ns_per_op = if iterations == 0 {
            0.0
        } else {
            total_ns / iterations as f64
        };
        let mops = if elapsed.as_secs_f64() > 0.0 {
            iterations as f64 / elapsed.as_secs_f64() / 1_000_000.0
        } else {
            0.0
        };

        println!(
            "NVIC Perf [{}]: iters={} total={:.3}ms ns/op={:.3} throughput={:.3} Mops/s",
            name,
            iterations,
            elapsed.as_secs_f64() * 1000.0,
            ns_per_op,
            mops
        );
    }

    #[test]
    fn nvic_enable_and_disable_irq_bits() {
        let mut nvic = Nvic::new(0xE000_E100, 0xE000_E4EF);

        nvic.write(0xE000_E100, 0b11);
        assert_eq!(nvic.read(0xE000_E100) & 0b11, 0b11);
        assert_eq!(nvic.read(0xE000_E180) & 0b11, 0b11);

        nvic.write(0xE000_E180, 0b01);
        assert_eq!(nvic.read(0xE000_E100) & 0b11, 0b10);
        assert_eq!(nvic.read(0xE000_E180) & 0b11, 0b10);
    }

    #[test]
    fn nvic_set_and_clear_pending_bits() {
        let mut nvic = Nvic::new(0xE000_E100, 0xE000_E4EF);

        nvic.write(0xE000_E200, 0x0000_0004);
        assert_eq!(nvic.read(0xE000_E200), 0x0000_0004);
        assert_eq!(nvic.read(0xE000_E280), 0x0000_0004);

        nvic.write(0xE000_E280, 0x0000_0004);
        assert_eq!(nvic.read(0xE000_E200), 0);
        assert_eq!(nvic.read(0xE000_E280), 0);
    }

    #[test]
    fn nvic_priority_word_rw() {
        let mut nvic = Nvic::new(0xE000_E100, 0xE000_E4EF);

        nvic.write(0xE000_E400, 0x4433_2211);

        assert_eq!(nvic.read(0xE000_E400), 0x4433_2211);
        assert_eq!(nvic.ipr[0], 0x11);
        assert_eq!(nvic.ipr[1], 0x22);
        assert_eq!(nvic.ipr[2], 0x33);
        assert_eq!(nvic.ipr[3], 0x44);
    }

    #[test]
    fn nvic_stir_sets_pending_irq() {
        let mut nvic = Nvic::new(0xE000_E100, 0xE000_E4EF);

        nvic.write(0xE000_E104, 1 << 13);
        nvic.write(0xE000_EF00, 45);

        assert_eq!(nvic.read(0xE000_EF00), 45);
        assert_ne!(nvic.read(0xE000_E304) & (1 << 13), 0);
        assert_eq!(nvic.read(0xE000_E204) & (1 << 13), 0);
    }

    #[test]
    fn nvic_pending_promotes_to_active_when_enabled() {
        let mut nvic = Nvic::new(0xE000_E100, 0xE000_E4EF);

        nvic.write(0xE000_E200, 1 << 5);
        assert_ne!(nvic.read(0xE000_E200) & (1 << 5), 0);
        assert_eq!(nvic.read(0xE000_E300) & (1 << 5), 0);

        nvic.write(0xE000_E100, 1 << 5);

        assert_eq!(nvic.read(0xE000_E200) & (1 << 5), 0);
        assert_ne!(nvic.read(0xE000_E300) & (1 << 5), 0);
    }

    #[test]
    fn nvic_icpr_does_not_clear_active_bit() {
        let mut nvic = Nvic::new(0xE000_E100, 0xE000_E4EF);

        nvic.write(0xE000_E100, 1 << 7);
        nvic.write(0xE000_E200, 1 << 7);
        assert_ne!(nvic.read(0xE000_E300) & (1 << 7), 0);

        nvic.write(0xE000_E280, 1 << 7);
        assert_ne!(nvic.read(0xE000_E300) & (1 << 7), 0);
    }

    #[test]
    fn nvic_clear_active_irq_clears_iabr_only() {
        let mut nvic = Nvic::new(0xE000_E100, 0xE000_E4EF);

        nvic.write(0xE000_E100, 1 << 9);
        nvic.write(0xE000_E200, 1 << 9);
        assert_ne!(nvic.read(0xE000_E300) & (1 << 9), 0);

        nvic.clear_active_irq(9);
        assert_eq!(nvic.read(0xE000_E300) & (1 << 9), 0);
        assert_eq!(nvic.read(0xE000_E200) & (1 << 9), 0);
    }

    #[test]
    fn perf_nvic_register_rw_hot_path() {
        let loops = 1_000_000u64;
        let mut nvic = Nvic::new(0xE000_E100, 0xE000_E4EF);

        let start = Instant::now();
        for i in 0..loops {
            let bit = 1u32 << ((i as u32) & 31);
            nvic.write(0xE000_E100, bit);
            nvic.write(0xE000_E200, bit);
            black_box(nvic.read(0xE000_E100));
            black_box(nvic.read(0xE000_E300));
            nvic.write(0xE000_E280, bit);
        }
        let elapsed = start.elapsed();

        black_box(nvic.pending_word_bitmap);
        black_box(nvic.active_word_bitmap);
        report_perf("register_rw_hot_path", loops * 5, elapsed);
        assert!(elapsed.as_nanos() > 0);
    }

    #[test]
    fn perf_nvic_bitmap_event_path() {
        let loops = 1_000_000u64;
        let mut nvic = Nvic::new(0xE000_E100, 0xE000_E4EF);

        let start = Instant::now();
        for i in 0..loops {
            let irq = (i as u32) & 0x1F;
            nvic.write(0xE000_E100, 1 << irq);
            nvic.write(0xE000_EF00, irq);
            black_box(nvic.pending_word_bitmap);
            black_box(nvic.active_word_bitmap);
            black_box(nvic.take_interrupt_event());
            nvic.clear_active_irq(irq);
        }
        let elapsed = start.elapsed();

        black_box(nvic.pending_word_bitmap);
        black_box(nvic.active_word_bitmap);
        report_perf("bitmap_event_path", loops * 6, elapsed);
        assert!(elapsed.as_nanos() > 0);
    }
}