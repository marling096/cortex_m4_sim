use std::any::Any;

pub trait Peripheral: Send {

    fn start(&self) -> u32;
    fn end(&self) -> u32;

    // 读取外设寄存器
    fn read(&self, addr: u32) -> u32;
    // 写入外设寄存器
    fn write(&mut self, addr: u32, val: u32);
    // 模拟时钟步进（比如定时器计数、串口发送数据）
    fn tick(&mut self);

    #[inline(always)]
    fn tick_n(&mut self, cycles: u32) {
        if cycles == 0 {
            return;
        }
        for _ in 0..cycles {
            self.tick();
        }
    }

    // 该外设是否需要参与每周期 tick
    #[inline(always)]
    fn needs_tick(&self) -> bool {
        true
    }

    // 该外设当前是否处于活跃 tick 状态（运行时门控）
    #[inline(always)]
    fn is_tick_active(&self) -> bool {
        true
    }

    #[inline(always)]
    /// 返回待处理的 IRQ 编号（0-based, 对应 NVIC 中断线），None 表示无中断
    fn pending_irq(&self) -> Option<u32> {
        None
    }

    /// 清除待处理的 IRQ 标志
    fn clear_pending_irq(&mut self) {}

    #[inline(always)]
    fn interrupt_event_pending(&self) -> bool {
        false
    }

    #[inline(always)]
    fn next_event_in_cycles(&self) -> Option<u32> {
        None
    }

    #[inline(always)]
    fn next_tick_cycle(&self) -> Option<u32> {
        self.next_event_in_cycles()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any;

}
