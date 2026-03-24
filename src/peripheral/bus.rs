use core::panic;
use std::any::Any;
use std::cell::Cell;

use crate::peripheral::afio::Afio;
use crate::peripheral::gpio::Gpio;
use crate::peripheral::peripheral::Peripheral;
use crate::peripheral::uart::Uart;

pub struct Bus {
    peripherals: Vec<(u32, u32, Box<dyn Peripheral>)>,
    tick_indices: Vec<usize>,
    active_tick_indices: Vec<usize>,
    tickable_flags: Vec<bool>,
    active_tick_flags: Vec<bool>,
    next_tick_due_cycles: Vec<u32>,
    cached_next_tick_cycle: u32,
    /// 可产生 IRQ 的外设索引列表（通过 pending_irq() 非 None 判断）
    irq_indices: Vec<usize>,
    last_hit_index: Cell<usize>,
    afio_index: Option<usize>,
    usart1_index: Option<usize>,
    bridge_event_pending: bool,
    last_usart1_tx_level: bool,
    last_usart1_tx_level_valid: bool,
}

impl Bus {
    const INVALID_INDEX: usize = usize::MAX;
    const AFIO_BASE: u32 = 0x4001_0000;
    const USART1_BASE: u32 = 0x4001_3800;
    const GPIO_MIN_BASE: u32 = 0x4001_0800;
    const GPIO_MAX_END: u32 = 0x4001_1BFF;

    pub fn new() -> Self {
        Self {
            peripherals: Vec::new(),
            tick_indices: Vec::new(),
            active_tick_indices: Vec::new(),
            tickable_flags: Vec::new(),
            active_tick_flags: Vec::new(),
            next_tick_due_cycles: Vec::new(),
            cached_next_tick_cycle: u32::MAX,
            irq_indices: Vec::new(),
            last_hit_index: Cell::new(Self::INVALID_INDEX),
            afio_index: None,
            usart1_index: None,
            bridge_event_pending: false,
            last_usart1_tx_level: true,
            last_usart1_tx_level_valid: false,
        }
    }

    #[inline(always)]
    fn is_gpio_write_addr(addr: u32) -> bool {
        (Self::GPIO_MIN_BASE..=Self::GPIO_MAX_END).contains(&addr)
    }

    #[inline(always)]
    fn is_bridge_related_write(&self, index: usize, addr: u32) -> bool {
        Some(index) == self.afio_index
            || Some(index) == self.usart1_index
            || Self::is_gpio_write_addr(addr)
    }

    #[inline(always)]
    fn take_bridge_event_pending(&mut self) -> bool {
        let pending = self.bridge_event_pending;
        self.bridge_event_pending = false;
        pending
    }

    #[inline(always)]
    fn normalize_next_tick_cycle(dev: &dyn Peripheral) -> u32 {
        match dev.next_tick_cycle() {
            Some(v) => v.max(1),
            None => u32::MAX,
        }
    }

    #[inline(always)]
    fn refresh_cached_next_tick_cycle(&mut self) {
        let mut next_tick_cycle = u32::MAX;

        for &index in &self.active_tick_indices {
            if index >= self.next_tick_due_cycles.len() {
                continue;
            }

            let due = self.next_tick_due_cycles[index];
            if due < next_tick_cycle {
                next_tick_cycle = due;
            }
        }

        self.cached_next_tick_cycle = next_tick_cycle;
    }

    pub fn register_peripheral(&mut self, dev: Box<dyn Peripheral>) {
        let index = self.peripherals.len();
        let start = dev.start();

        let tickable = dev.needs_tick();
        let active = tickable && dev.is_tick_active();

        if tickable {
            self.tick_indices.push(index);
        }
        if active {
            self.active_tick_indices.push(index);
        }

        self.tickable_flags.push(tickable);
        self.active_tick_flags.push(active);

        let next_due = if active {
            Self::normalize_next_tick_cycle(&*dev)
        } else {
            u32::MAX
        };
        self.next_tick_due_cycles.push(next_due);
        if next_due < self.cached_next_tick_cycle {
            self.cached_next_tick_cycle = next_due;
        }

        self.peripherals.push((dev.start(), dev.end(), dev));

        if start == Self::AFIO_BASE {
            self.afio_index = Some(index);
        }
        if start == Self::USART1_BASE {
            self.usart1_index = Some(index);
        }
    }

    #[inline(always)]
    fn set_tick_active_state(&mut self, index: usize, active_now: bool) {
        if index >= self.active_tick_flags.len() || !self.tickable_flags[index] {
            return;
        }

        let was_active = self.active_tick_flags[index];
        if was_active == active_now {
            return;
        }

        self.active_tick_flags[index] = active_now;
        if active_now {
            let due = {
                let (_, _, dev) = &self.peripherals[index];
                Self::normalize_next_tick_cycle(&**dev)
            };
            self.next_tick_due_cycles[index] = due;
        } else {
            self.next_tick_due_cycles[index] = u32::MAX;
        }

        if active_now {
            self.active_tick_indices.push(index);
        } else if let Some(pos) = self.active_tick_indices.iter().position(|&i| i == index) {
            self.active_tick_indices.swap_remove(pos);
        }

        self.refresh_cached_next_tick_cycle();
    }

    /// 注册可产生 IRQ 的外设索引（注册后 drain_pending_irqs 才会检查它）
    pub fn register_irq_peripheral(&mut self, start_addr: u32) {
        if let Some(index) = self.find_peripheral_index(start_addr) {
            if !self.irq_indices.contains(&index) {
                self.irq_indices.push(index);
            }
        }
    }

    /// 遍历所有 IRQ 外设，将待处理的中断号通过回调传出，并自动清除标志
    #[inline(always)]
    pub fn drain_pending_irqs<F: FnMut(u32)>(&mut self, mut callback: F) {
        if self.irq_indices.is_empty() {
            return;
        }

        for i in 0..self.irq_indices.len() {
            let index = self.irq_indices[i];
            debug_assert!(index < self.peripherals.len());
            unsafe {
                let (_, _, dev) = self.peripherals.get_unchecked_mut(index);
                if let Some(irq) = dev.pending_irq() {
                    callback(irq);
                    dev.clear_pending_irq();
                }
            }
        }
    }

    #[inline(always)]
    fn range_hit(addr: u32, start: u32, end: u32) -> bool {
        addr >= start && addr <= end
    }

    #[inline(always)]
    pub fn has_tickables(&self) -> bool {
        !self.tick_indices.is_empty()
    }

    #[inline(always)]
    pub fn has_irq_sources(&self) -> bool {
        !self.irq_indices.is_empty()
    }

    #[inline(always)]
    pub fn read32(&self, addr: u32) -> u32 {
        let last = self.last_hit_index.get();
        if last != Self::INVALID_INDEX && last < self.peripherals.len() {
            let (start, end, dev) = &self.peripherals[last];
            if Self::range_hit(addr, *start, *end) {
                return dev.read(addr);
            }
        }

        for (index, (start, end, dev)) in self.peripherals.iter().enumerate() {
            if Self::range_hit(addr, *start, *end) {
                self.last_hit_index.set(index);
                return dev.read(addr);
            }
        }

        panic!("Unmapped read32 at address {:08X}", addr);
    }

    #[inline(always)]
    pub fn write32(&mut self, addr: u32, val: u32) -> bool {
        let last = self.last_hit_index.get();
        let hit_index = if last != Self::INVALID_INDEX
            && last < self.peripherals.len()
            && Self::range_hit(addr, self.peripherals[last].0, self.peripherals[last].1)
        {
            Some(last)
        } else {
            self.peripherals
                .iter()
                .position(|(start, end, _)| Self::range_hit(addr, *start, *end))
        };

        if let Some(index) = hit_index {
            self.last_hit_index.set(index);

            let tickable = self.tickable_flags[index];
            let mut active_now = false;
            let mut next_due = u32::MAX;
            let mut schedule_changed = false;

            {
                let (_, _, dev) = &mut self.peripherals[index];
                dev.write(addr, val);

                if tickable {
                    active_now = dev.is_tick_active();
                    next_due = if active_now {
                        Self::normalize_next_tick_cycle(&**dev)
                    } else {
                        u32::MAX
                    };
                }
            }

            if tickable {
                self.set_tick_active_state(index, active_now);
                self.next_tick_due_cycles[index] = if self.active_tick_flags[index] {
                    next_due
                } else {
                    u32::MAX
                };
                self.refresh_cached_next_tick_cycle();
                schedule_changed = true;
            }
            if self.is_bridge_related_write(index, addr) {
                self.bridge_event_pending = true;
            }
            return schedule_changed;
        }

        panic!("Unmapped write32 at address {:08X}", addr);
    }

    #[inline(always)]
    pub fn tick(&mut self) -> bool {
        self.tick_n(1)
    }

    #[inline(always)]
    pub fn tick_n(&mut self, cycles: u32) -> bool {
        if cycles == 0 {
            return false;
        }
        let mut has_event = false;
        let mut remaining = cycles;

        while remaining > 0 {
            let next_due = self.cached_next_tick_cycle;

            if next_due == u32::MAX {
                if self.take_bridge_event_pending() {
                    self.process_uart_afio_bridge();
                }
                break;
            }

            if next_due > remaining {
                for &index in &self.active_tick_indices {
                    if index < self.next_tick_due_cycles.len() {
                        let due = self.next_tick_due_cycles[index];
                        if due != u32::MAX {
                            self.next_tick_due_cycles[index] = due.saturating_sub(remaining);
                        }
                    }
                }
                if self.cached_next_tick_cycle != u32::MAX {
                    self.cached_next_tick_cycle = self.cached_next_tick_cycle.saturating_sub(remaining);
                }
                if self.take_bridge_event_pending() {
                    self.process_uart_afio_bridge();
                }
                break;
            }

            for &index in &self.active_tick_indices {
                if index < self.next_tick_due_cycles.len() {
                    let due = self.next_tick_due_cycles[index];
                    if due != u32::MAX {
                        self.next_tick_due_cycles[index] = due.saturating_sub(next_due);
                    }
                }
            }
            remaining -= next_due;

            let mut fired: Vec<usize> = Vec::new();
            for &index in &self.active_tick_indices {
                if index < self.next_tick_due_cycles.len() && self.next_tick_due_cycles[index] == 0 {
                    fired.push(index);
                }
            }

            if fired.is_empty() {
                self.refresh_cached_next_tick_cycle();
                break;
            }

            let mut uart_fired = false;

            for index in fired {
                if index >= self.peripherals.len() {
                    continue;
                }
                if Some(index) == self.usart1_index {
                    uart_fired = true;
                }
                let mut active_now = self.active_tick_flags[index];
                let mut next_due_for_dev = u32::MAX;

                {
                    let (_, _, dev) = &mut self.peripherals[index];
                    dev.tick_n(next_due.max(1));
                    if !has_event {
                        has_event = dev.interrupt_event_pending() || dev.pending_irq().is_some();
                    }

                    if self.tickable_flags[index] {
                        active_now = dev.is_tick_active();
                        next_due_for_dev = if active_now {
                            Self::normalize_next_tick_cycle(&**dev)
                        } else {
                            u32::MAX
                        };
                    }
                }

                if self.tickable_flags[index] {
                    self.set_tick_active_state(index, active_now);
                    self.next_tick_due_cycles[index] = if self.active_tick_flags[index] {
                        next_due_for_dev
                    } else {
                        u32::MAX
                    };
                }
            }

            self.refresh_cached_next_tick_cycle();

            if uart_fired || self.take_bridge_event_pending() {
                self.process_uart_afio_bridge();
            }
        }

        has_event
    }

    #[inline(always)]
    pub fn next_event_in_cycles(&self) -> Option<u32> {
        if self.cached_next_tick_cycle == u32::MAX {
            None
        } else {
            Some(self.cached_next_tick_cycle)
        }
    }

    pub fn get_peripheral_mut<T: Any>(&mut self, start_addr: u32) -> Option<&mut T> {
        for (start, _end, dev) in &mut self.peripherals {
            if *start == start_addr {
                return dev.as_any_mut().downcast_mut::<T>();
            }
        }
        None
    }

    pub fn find_peripheral_index(&self, start_addr: u32) -> Option<usize> {
        self.peripherals
            .iter()
            .position(|(start, _end, _dev)| *start == start_addr)
    }

    pub fn get_peripheral_mut_by_index<T: Any>(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.peripherals.len() {
            return None;
        }

        let (_, _, dev) = &mut self.peripherals[index];
        dev.as_any_mut().downcast_mut::<T>()
    }

    /// 将 USART1 与 AFIO/GPIO 桥接处理合并为单一路径，减少每 tick 的重复查找与重复读写。
    fn process_uart_afio_bridge(&mut self) {
        let afio_index = if let Some(index) = self.afio_index {
            index
        } else {
            return;
        };
        let usart_index = if let Some(index) = self.usart1_index {
            index
        } else {
            return;
        };

        let (gpio_base, rx_sense_pin, tx_drive_pin, custom_bridge) = {
            if let Some(afio) = self.get_peripheral_mut_by_index::<Afio>(afio_index) {
                let (gpio_base, rx_sense_pin, tx_drive_pin) = afio.usart1_pin_mapping();
                (
                    gpio_base,
                    rx_sense_pin,
                    tx_drive_pin,
                    afio.gpio_bridge_config(),
                )
            } else {
                return;
            }
        };

        let (uart_tx_level, uart_tx_active) = {
            if let Some(uart) = self.get_peripheral_mut_by_index::<Uart>(usart_index) {
                (uart.tx_line_level(), uart.tx_active())
            } else {
                return;
            }
        };

        let tx_line_changed = if self.last_usart1_tx_level_valid {
            self.last_usart1_tx_level != uart_tx_level
        } else {
            true
        };
        self.last_usart1_tx_level = uart_tx_level;
        self.last_usart1_tx_level_valid = true;

        // 无自定义桥接且 UART TX 空闲时，不需要每 tick 做桥接处理。
        if custom_bridge.is_none() && !uart_tx_active && !tx_line_changed {
            return;
        }

        // UART TX -> 映射 GPIO TX
        if uart_tx_active || tx_line_changed {
            if let Some(gpio) = self.get_peripheral_mut::<Gpio>(gpio_base) {
                gpio.set_odr_pin(tx_drive_pin, uart_tx_level);
            }
        }

        // 自定义桥接（例如 PB0 -> PA10）优先；否则做默认 USART1 TX->RX 回送。
        let gpio_rx_level = if let Some(bridge) = custom_bridge {
            let level = {
                if let Some(src_gpio) = self.get_peripheral_mut::<Gpio>(bridge.src_gpio_base) {
                    src_gpio.read_odr_pin(bridge.src_pin)
                } else {
                    return;
                }
            };

            if let Some(dst_gpio) = self.get_peripheral_mut::<Gpio>(bridge.dst_gpio_base) {
                dst_gpio.set_idr_pin(bridge.dst_pin, level);
            }

            if let Some(gpio) = self.get_peripheral_mut::<Gpio>(gpio_base) {
                gpio.read_idr_pin(rx_sense_pin)
            } else {
                return;
            }
        } else {
            if let Some(gpio) = self.get_peripheral_mut::<Gpio>(gpio_base) {
                gpio.set_idr_pin(rx_sense_pin, uart_tx_level);
            }
            uart_tx_level
        };

        if let Some(uart) = self.get_peripheral_mut_by_index::<Uart>(usart_index) {
            uart.set_rx_line(gpio_rx_level);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyPeripheral {
        start: u32,
        end: u32,
        value: u32,
        ticks: u32,
    }

    impl DummyPeripheral {
        fn new(start: u32, end: u32) -> Self {
            Self {
                start,
                end,
                value: 0,
                ticks: 0,
            }
        }
    }

    impl Peripheral for DummyPeripheral {
        fn start(&self) -> u32 {
            self.start
        }

        fn end(&self) -> u32 {
            self.end
        }

        fn read(&self, _addr: u32) -> u32 {
            self.value
        }

        fn write(&mut self, _addr: u32, val: u32) {
            self.value = val;
        }

        fn tick(&mut self) {
            self.ticks = self.ticks.wrapping_add(1);
            self.value = self.ticks;
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[test]
    fn bus_dispatches_read_and_write_to_registered_peripheral() {
        let mut bus = Bus::new();
        bus.register_peripheral(Box::new(DummyPeripheral::new(0x4000_0000, 0x4000_00FF)));

        bus.write32(0x4000_0004, 0xABCD_1234);
        let value = bus.read32(0x4000_0004);

        assert_eq!(value, 0xABCD_1234);
    }

    #[test]
    fn usart1_loopback_via_afio_gpio_bridge() {
        const AFIO_BASE: u32 = 0x4001_0000;
        const GPIOA_BASE: u32 = 0x4001_0800;
        const USART1_BASE: u32 = 0x4001_3800;

        let mut bus = Bus::new();
        bus.register_peripheral(Box::new(Afio::new(AFIO_BASE, AFIO_BASE + 0x3FF)));
        bus.register_peripheral(Box::new(Gpio::new(GPIOA_BASE, GPIOA_BASE + 0x3FF)));
        bus.register_peripheral(Box::new(Uart::new(USART1_BASE, USART1_BASE + 0x3FF)));

        // GPIOA CRH: PA9 配置为复用推挽输出(0b1011)，PA10 配置为输入浮空(0b0100)
        let crh = (0b1011u32 << 4) | (0b0100u32 << 8);
        bus.write32(GPIOA_BASE + 0x04, crh);

        // AFIO GPIO Bridge: src=GPIOA pin9 -> dst=GPIOA pin10, enable
        bus.write32(AFIO_BASE + 0x24, GPIOA_BASE);
        bus.write32(AFIO_BASE + 0x28, GPIOA_BASE);
        bus.write32(AFIO_BASE + 0x2C, 9 | (10 << 8));
        bus.write32(AFIO_BASE + 0x20, 1);

        // USART1 发出一个字节
        bus.write32(USART1_BASE + 0x04, b'A' as u32);

        // 8N1: start + 8 data + stop，额外留一点余量
        bus.tick_n(12);

        let sr = bus.read32(USART1_BASE + 0x00);
        let dr = bus.read32(USART1_BASE + 0x04);
        assert_ne!(sr & (1 << 5), 0, "RXNE should be set after loopback receive");
        assert_eq!(dr & 0xFF, b'A' as u32);
    }

    #[test]
    fn usart1_default_loopback_without_custom_bridge() {
        const AFIO_BASE: u32 = 0x4001_0000;
        const GPIOA_BASE: u32 = 0x4001_0800;
        const USART1_BASE: u32 = 0x4001_3800;

        let mut bus = Bus::new();
        bus.register_peripheral(Box::new(Afio::new(AFIO_BASE, AFIO_BASE + 0x3FF)));
        bus.register_peripheral(Box::new(Gpio::new(GPIOA_BASE, GPIOA_BASE + 0x3FF)));
        bus.register_peripheral(Box::new(Uart::new(USART1_BASE, USART1_BASE + 0x3FF)));

        bus.write32(USART1_BASE + 0x04, b'B' as u32);
        bus.tick_n(16);

        let sr = bus.read32(USART1_BASE + 0x00);
        let dr = bus.read32(USART1_BASE + 0x04);
        assert_ne!(sr & (1 << 5), 0, "RXNE should be set with default loopback");
        assert_eq!(dr & 0xFF, b'B' as u32);
    }

    #[test]
    fn firmware_echo_loop_print_test() {
        const AFIO_BASE: u32 = 0x4001_0000;
        const GPIOA_BASE: u32 = 0x4001_0800;
        const GPIOB_BASE: u32 = 0x4001_0C00;
        const USART1_BASE: u32 = 0x4001_3800;
        const USART_SR_ADDR: u32 = USART1_BASE + 0x00;
        const USART_DR_ADDR: u32 = USART1_BASE + 0x04;
        const RXNE: u32 = 1 << 5;

        let mut bus = Bus::new();
        bus.register_peripheral(Box::new(Afio::new(AFIO_BASE, AFIO_BASE + 0x3FF)));
        bus.register_peripheral(Box::new(Gpio::new(GPIOA_BASE, GPIOA_BASE + 0x3FF)));
        bus.register_peripheral(Box::new(Gpio::new(GPIOB_BASE, GPIOB_BASE + 0x3FF)));
        bus.register_peripheral(Box::new(Uart::new(USART1_BASE, USART1_BASE + 0x3FF)));

        // GPIOA CRH: PA9=复用推挽输出(USART1_TX), PA10=输入浮空(USART1_RX)
        let gpioa_crh = (0b1011u32 << 4) | (0b0100u32 << 8);
        bus.write32(GPIOA_BASE + 0x04, gpioa_crh);
        // GPIOB CRL: PB0=通用输出推挽，用作外部串口输入信号源
        bus.write32(GPIOB_BASE + 0x00, 0b0001);

        // AFIO GPIO Bridge: PB0 -> PA10（向 USART1_RX 注入位流）
        bus.write32(AFIO_BASE + 0x24, GPIOB_BASE);
        bus.write32(AFIO_BASE + 0x28, GPIOA_BASE);
        bus.write32(AFIO_BASE + 0x2C, 0 | (10 << 8));
        bus.write32(AFIO_BASE + 0x20, 1);

        // 等价固件循环体：
        // if (Serial_GetRxFlag()) { RxData = Serial_GetRxData(); Serial_SendByte(RxData); }
        // Delay_us(10)
        let firmware_step = |bus: &mut Bus| {
            if (bus.read32(USART_SR_ADDR) & RXNE) != 0 {
                let rx_data = (bus.read32(USART_DR_ADDR) & 0xFF) as u8;
                println!("[FW] RX=0x{:02X} '{}'", rx_data, rx_data as char);
                bus.write32(USART_DR_ADDR, rx_data as u32);
                println!("[FW] TX=0x{:02X} '{}'", rx_data, rx_data as char);
            }
            // 当前 UART 模型为每 tick 一位采样；此处不额外推进 tick，避免拉伸位宽导致测试失真。
        };

        let drive_src = |bus: &mut Bus, level: bool| {
            if let Some(src) = bus.get_peripheral_mut::<Gpio>(GPIOB_BASE) {
                src.set_odr_pin(0, level);
            }
            bus.tick();
            firmware_step(bus);
        };

        let inject_uart_byte = |bus: &mut Bus, data: u8| {
            // idle 高电平
            drive_src(bus, true);
            // start bit
            drive_src(bus, false);
            // data bits LSB first
            for bit in 0..8 {
                let level = ((data >> bit) & 1) != 0;
                drive_src(bus, level);
            }
            // stop bit
            drive_src(bus, true);
        };

        inject_uart_byte(&mut bus, b'Z');

        let echoed = {
            let uart = bus
                .get_peripheral_mut::<Uart>(USART1_BASE)
                .expect("uart should exist");
            uart.take_tx_bytes()
        };

        println!("[TEST] echoed bytes: {:?}", echoed);
        assert!(echoed.contains(&b'Z'), "firmware echo should send back injected byte");
    }

    #[test]
    #[should_panic(expected = "Unmapped read32")]
    fn bus_panics_on_unmapped_read() {
        let bus = Bus::new();
        let _ = bus.read32(0x5000_0000);
    }

    #[test]
    #[should_panic(expected = "Unmapped write32")]
    fn bus_panics_on_unmapped_write() {
        let mut bus = Bus::new();
        bus.write32(0x5000_0000, 0x1234_5678);
    }

    #[test]
    fn bus_tick_calls_peripheral_tick() {
        let mut bus = Bus::new();
        bus.register_peripheral(Box::new(DummyPeripheral::new(0x4000_0000, 0x4000_00FF)));

        bus.tick();
        bus.tick();
        let value = bus.read32(0x4000_0000);

        assert_eq!(value, 2);
    }

    #[test]
    fn bus_cached_next_event_updates_after_uart_write_and_ticks() {
        const USART1_BASE: u32 = 0x4001_3800;

        let mut bus = Bus::new();
        bus.register_peripheral(Box::new(Uart::new(USART1_BASE, USART1_BASE + 0x3FF)));

        assert_eq!(bus.next_event_in_cycles(), None);

        bus.write32(USART1_BASE + 0x04, b'A' as u32);
        assert_eq!(bus.next_event_in_cycles(), Some(1));

        bus.tick();
        assert_eq!(bus.next_event_in_cycles(), Some(1));

        bus.tick_n(16);
        assert_eq!(bus.next_event_in_cycles(), None);
    }
}
