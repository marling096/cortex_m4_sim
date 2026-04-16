mod arch;
mod context;
mod cpu;
mod disassembler;
mod jit_engine;
mod opcodes;
mod peripheral;
mod simulator;
#[cfg(test)]
mod perf_tests;

use crate::cpu::Cpu;
use crate::disassembler::disassemble_from_reset_handler;
use crate::peripheral::bus::Bus;
use crate::peripheral::afio::Afio;
use crate::peripheral::flash::Flash;
use crate::peripheral::gpio::Gpio;
use crate::peripheral::nvic::Nvic;
use crate::peripheral::rcc::Rcc;
use crate::peripheral::scb::Scb;
use crate::peripheral::systick::SysTick;
use crate::peripheral::timer::GeneralTimer;
use crate::peripheral::uart::Uart;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = std::env::var("SIM_INPUT_PATH")
        .unwrap_or_else(|_| "uart_loop.axf".to_string());
    let output_path = "disassembly_detail.asm";
    let use_jit = std::env::var("SIM_USE_BLOCK")
        .map(|v| v != "0")
        .unwrap_or(false);

    let (_result, _cs, code_segments, dcw_data, initial_sp, reset_handler_ptr, _reset_handler_addr) =
        disassemble_from_reset_handler(&input_path, output_path)?;
    println!(
        "Initial SP: 0x{:08X}, Reset_Handler Ptr: 0x{:08X}",
        initial_sp, reset_handler_ptr
    );

    let shared_freq = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(8_000_000));
    let gpioa = Gpio::new(0x4001_0800, 0x4001_0BFF);
    let gpiob = Gpio::new(0x4001_0C00, 0x4001_0FFF);
    let gpioc = Gpio::new(0x4001_1000, 0x4001_13FF);
    let afio = Afio::new(0x4001_0000, 0x4001_03FF);
    let usart1 = Uart::new(0x4001_3800, 0x4001_3BFF);
    let rcc = Rcc::new(0x4002_0000, 0x4002_1024, shared_freq.clone());
    let flash_interface = Flash::new(0x40022000, 0x4002201C);

    // 通用定时器 TIM2-TIM5（APB1，IRQ 编号参见 STM32F4 参考手册）
    let tim2 = GeneralTimer::new(0x4000_0000, 0x4000_03FF, 28);
    let tim3 = GeneralTimer::new(0x4000_0400, 0x4000_07FF, 29);
    let tim4 = GeneralTimer::new(0x4000_0800, 0x4000_0BFF, 30);
    let tim5 = GeneralTimer::new(0x4000_0C00, 0x4000_0FFF, 50);

    let mut bus = Bus::new();
    bus.register_peripheral(Box::new(afio));
    bus.register_peripheral(Box::new(gpioa));
    bus.register_peripheral(Box::new(gpiob));
    bus.register_peripheral(Box::new(gpioc));
    bus.register_peripheral(Box::new(usart1));
    bus.register_peripheral(Box::new(flash_interface));
    bus.register_peripheral(Box::new(rcc));
    // 注册 TIM2-TIM5，并告知总线它们可产生 IRQ
    bus.register_peripheral(Box::new(tim2));
    bus.register_irq_peripheral(0x4000_0000);
    bus.register_peripheral(Box::new(tim3));
    bus.register_irq_peripheral(0x4000_0400);
    bus.register_peripheral(Box::new(tim4));
    bus.register_irq_peripheral(0x4000_0800);
    bus.register_peripheral(Box::new(tim5));
    bus.register_irq_peripheral(0x4000_0C00);

    let mut ppb = Bus::new();
    let scb = Scb::new(0xE000_ED00, 0xE000_ED3C);
    let systick = SysTick::new(0xE000E010, 0xE000E01F);
    let nvic = Nvic::new(0xE000_E100, 0xE000_E4EF);
    ppb.register_peripheral(Box::new(systick));
    ppb.register_peripheral(Box::new(nvic));
    ppb.register_peripheral(Box::new(scb));

    let cpu = Cpu::new(shared_freq, 1, bus, ppb);
    let mut simulator = simulator::Simulator::new(cpu);
    simulator.sim_reset(&code_segments, dcw_data, initial_sp, reset_handler_ptr);
    if use_jit {
        simulator.sim_loop_jit()?;
    } else {
        simulator.sim_loop_interpreter()?;
    }

    Ok(())
}
