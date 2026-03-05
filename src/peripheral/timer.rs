/// STM32F4 通用定时器 (TIM2 ~ TIM5) 仿真
///
/// 地址映射（APB1 外设）：
///   TIM2: 0x4000_0000 - 0x4000_03FF  IRQ 28
///   TIM3: 0x4000_0400 - 0x4000_07FF  IRQ 29
///   TIM4: 0x4000_0800 - 0x4000_0BFF  IRQ 30
///   TIM5: 0x4000_0C00 - 0x4000_0FFF  IRQ 50
///
/// 每次 tick() 代表一个定时器输入时钟周期（由上层CPU/外设总线驱动）。
/// 预分频器: CLK / (PSC + 1)，PSC 在更新事件时重载（缓冲）。
/// 自动重装: CR1.ARPE=1 时 ARR 在更新事件时重载；=0 时立即生效。
use super::peripheral::Peripheral;
use std::any::Any;
use std::cell::Cell;

// ---------------------------------------------------------------------------
// CR1 位定义
// ---------------------------------------------------------------------------
const CR1_CEN: u32 = 1 << 0; // 计数器使能
const CR1_UDIS: u32 = 1 << 1; // 禁止更新事件
const CR1_URS: u32 = 1 << 2; // 更新请求源（0=所有, 1=仅溢出/UG）
const CR1_OPM: u32 = 1 << 3; // 单脉冲模式
const CR1_DIR: u32 = 1 << 4; // 方向（0=向上, 1=向下）
const CR1_ARPE: u32 = 1 << 7; // 自动重装预装载使能

// ---------------------------------------------------------------------------
// DIER 位定义
// ---------------------------------------------------------------------------
const DIER_UIE: u32 = 1 << 0; // 更新中断使能
const DIER_CC1IE: u32 = 1 << 1; // 捕获/比较 1 中断使能
const DIER_CC2IE: u32 = 1 << 2; // 捕获/比较 2 中断使能
const DIER_CC3IE: u32 = 1 << 3; // 捕获/比较 3 中断使能
const DIER_CC4IE: u32 = 1 << 4; // 捕获/比较 4 中断使能

// ---------------------------------------------------------------------------
// SR 位定义
// ---------------------------------------------------------------------------
const SR_UIF: u32 = 1 << 0; // 更新中断标志
const SR_CC1IF: u32 = 1 << 1; // 捕获/比较 1 标志
const SR_CC2IF: u32 = 1 << 2; // 捕获/比较 2 标志
const SR_CC3IF: u32 = 1 << 3; // 捕获/比较 3 标志
const SR_CC4IF: u32 = 1 << 4; // 捕获/比较 4 标志

// ---------------------------------------------------------------------------
// EGR 位定义（只写）
// ---------------------------------------------------------------------------
const EGR_UG: u32 = 1 << 0; // 软件更新生成
const EGR_CC1G: u32 = 1 << 1;
const EGR_CC2G: u32 = 1 << 2;
const EGR_CC3G: u32 = 1 << 3;
const EGR_CC4G: u32 = 1 << 4;

// ---------------------------------------------------------------------------
// SR 中可写清零的位掩码（RC_W0 类型）
// ---------------------------------------------------------------------------
const SR_WRITE_MASK: u32 = SR_UIF | SR_CC1IF | SR_CC2IF | SR_CC3IF | SR_CC4IF;

/// 通用定时器外设（TIM2 ~ TIM5）
pub struct GeneralTimer {
    pub start: u32,
    pub end: u32,
    pub irq_num: u32, // 对应 NVIC IRQ 编号（0-based）

    // ---- 寄存器（Cell 避免按位借用问题）----
    cr1: Cell<u32>,
    cr2: Cell<u32>,
    smcr: Cell<u32>,
    dier: Cell<u32>,
    sr: Cell<u32>,
    ccmr1: Cell<u32>,
    ccmr2: Cell<u32>,
    ccer: Cell<u32>,
    cnt: Cell<u32>,
    psc: Cell<u32>,
    arr: Cell<u32>,
    ccr1: Cell<u32>,
    ccr2: Cell<u32>,
    ccr3: Cell<u32>,
    ccr4: Cell<u32>,
    dcr: Cell<u32>,
    dmar: Cell<u32>,

    // ---- 内部仿真状态 ----
    /// 预分频计数器（0 … psc_shadow）
    psc_cnt: Cell<u32>,
    /// PSC 影子寄存器（更新事件时从 psc 同步）
    psc_shadow: Cell<u32>,
    /// ARR 影子寄存器（ARPE=1 时在更新事件同步；=0 时立即同步）
    arr_shadow: Cell<u32>,

    /// 中断待处理标志（已触发但尚未被 CPU 读取）
    interrupt_pending: Cell<bool>,
}

impl GeneralTimer {
    /// 创建一个通用定时器实例。
    /// - `start` / `end`：外设寄存器地址范围
    /// - `irq_num`：对应的 NVIC IRQ 编号（0-based）
    pub fn new(start: u32, end: u32, irq_num: u32) -> Self {
        Self {
            start,
            end,
            irq_num,
            cr1: Cell::new(0),
            cr2: Cell::new(0),
            smcr: Cell::new(0),
            dier: Cell::new(0),
            sr: Cell::new(0),
            ccmr1: Cell::new(0),
            ccmr2: Cell::new(0),
            ccer: Cell::new(0),
            cnt: Cell::new(0),
            psc: Cell::new(0),
            arr: Cell::new(0), // STM32F1 16-bit timer，初始 ARR=0（STRH 写入后得到正确 16-bit 值）
            ccr1: Cell::new(0),
            ccr2: Cell::new(0),
            ccr3: Cell::new(0),
            ccr4: Cell::new(0),
            dcr: Cell::new(0),
            dmar: Cell::new(0),
            psc_cnt: Cell::new(0),
            psc_shadow: Cell::new(0),
            arr_shadow: Cell::new(0), // 与 arr 保持一致
            interrupt_pending: Cell::new(false),
        }
    }

    // ---- 便捷访问器 ----

    #[inline(always)]
    fn is_enabled(&self) -> bool {
        self.cr1.get() & CR1_CEN != 0
    }

    #[inline(always)]
    fn is_dir_down(&self) -> bool {
        self.cr1.get() & CR1_DIR != 0
    }

    #[inline(always)]
    fn is_update_disabled(&self) -> bool {
        self.cr1.get() & CR1_UDIS != 0
    }

    #[inline(always)]
    fn is_one_pulse(&self) -> bool {
        self.cr1.get() & CR1_OPM != 0
    }

    #[inline(always)]
    fn is_arpe(&self) -> bool {
        self.cr1.get() & CR1_ARPE != 0
    }

    /// 执行更新事件：
    ///   - 同步 PSC shadow
    ///   - 若 ARPE=1 同步 ARR shadow
    ///   - 若未禁止更新事件且中断使能则设置 UIF 并挂起中断
    fn do_update_event(&self) {
        // 同步预分频影子寄存器
        self.psc_shadow.set(self.psc.get());

        // 若未禁止更新事件
        if !self.is_update_disabled() {
            // 同步 ARR 影子寄存器
            if self.is_arpe() {
                self.arr_shadow.set(self.arr.get());
            }

            // 置 UIF
            let old_sr = self.sr.get();
            self.sr.set(old_sr | SR_UIF);

            // 若更新中断使能，挂起中断
            if self.dier.get() & DIER_UIE != 0 {
                self.interrupt_pending.set(true);
            }
        }
    }

    /// 检查捕获/比较匹配（CCRx == CNT），设置对应 CCxIF 标志
    fn check_cc_match(&self, cnt: u32) {
        let dier = self.dier.get();
        let mut sr = self.sr.get();
        let mut need_irq = false;

        macro_rules! check_cc {
            ($ccr:expr, $flag:expr, $ie:expr) => {
                if cnt == $ccr.get() {
                    sr |= $flag;
                    if dier & $ie != 0 {
                        need_irq = true;
                    }
                }
            };
        }

        check_cc!(self.ccr1, SR_CC1IF, DIER_CC1IE);
        check_cc!(self.ccr2, SR_CC2IF, DIER_CC2IE);
        check_cc!(self.ccr3, SR_CC3IF, DIER_CC3IE);
        check_cc!(self.ccr4, SR_CC4IF, DIER_CC4IE);

        self.sr.set(sr);
        if need_irq {
            self.interrupt_pending.set(true);
        }
    }

    /// 软件触发更新（EGR.UG）
    fn sw_update(&self) {
        self.psc_cnt.set(0);
        // CNT 复位（向上/单脉冲=0，向下=ARR）
        if self.is_dir_down() {
            self.cnt.set(self.arr_shadow.get());
        } else {
            self.cnt.set(0);
        }
        self.do_update_event();
    }

    pub fn is_interrupt_pending(&self) -> bool {
        self.interrupt_pending.get()
    }

    pub fn clear_interrupt_pending(&self) {
        self.interrupt_pending.set(false);
    }

    #[inline(always)]
    fn cc_hit_up(start: u32, end: u32, arr: u32, target: u32, wrapped: bool) -> bool {
        if !wrapped {
            target > start && target <= end
        } else {
            (target > start && target <= arr) || target <= end
        }
    }

    #[inline(always)]
    fn cc_hit_down(start: u32, end: u32, target: u32, wrapped: bool) -> bool {
        if !wrapped {
            target < start && target >= end
        } else {
            target < start || target >= end
        }
    }

    fn set_cc_flags_batch(&self, start_cnt: u32, end_cnt: u32, arr: u32, wrapped: bool, wraps: u32, down: bool) {
        let mut sr = self.sr.get();
        let dier = self.dier.get();
        let mut need_irq = false;

        macro_rules! maybe_set {
            ($ccr:expr, $flag:expr, $ie:expr) => {
                let ccr = $ccr.get();
                let hit = if wraps >= 2 {
                    true
                } else if down {
                    Self::cc_hit_down(start_cnt, end_cnt, ccr, wrapped)
                } else {
                    Self::cc_hit_up(start_cnt, end_cnt, arr, ccr, wrapped)
                };
                if hit {
                    sr |= $flag;
                    if dier & $ie != 0 {
                        need_irq = true;
                    }
                }
            };
        }

        maybe_set!(self.ccr1, SR_CC1IF, DIER_CC1IE);
        maybe_set!(self.ccr2, SR_CC2IF, DIER_CC2IE);
        maybe_set!(self.ccr3, SR_CC3IF, DIER_CC3IE);
        maybe_set!(self.ccr4, SR_CC4IF, DIER_CC4IE);

        self.sr.set(sr);
        if need_irq {
            self.interrupt_pending.set(true);
        }
    }
}

impl Peripheral for GeneralTimer {
    fn start(&self) -> u32 {
        self.start
    }
    fn end(&self) -> u32 {
        self.end
    }

    fn read(&self, addr: u32) -> u32 {
        let offset = addr.wrapping_sub(self.start);
        match offset {
            0x00 => self.cr1.get(),
            0x04 => self.cr2.get(),
            0x08 => self.smcr.get(),
            0x0C => self.dier.get(),
            0x10 => self.sr.get(),
            0x14 => 0, // EGR 只写
            0x18 => self.ccmr1.get(),
            0x1C => self.ccmr2.get(),
            0x20 => self.ccer.get(),
            0x24 => self.cnt.get(),
            0x28 => self.psc.get(),
            0x2C => self.arr.get(),
            0x34 => self.ccr1.get(),
            0x38 => self.ccr2.get(),
            0x3C => self.ccr3.get(),
            0x40 => self.ccr4.get(),
            0x48 => self.dcr.get(),
            0x4C => self.dmar.get(),
            _ => 0,
        }
    }

    fn write(&mut self, addr: u32, val: u32) {
        let offset = addr.wrapping_sub(self.start);
        match offset {
            0x00 => {
                let was_enabled = self.is_enabled();
                self.cr1.set(val);
                // 若初次使能
                if !was_enabled && self.is_enabled() {
                    if !self.is_arpe() {
                        self.arr_shadow.set(self.arr.get());
                    }
                    // STM32 向下计数：使能时 CNT 从 ARR 开始（若 CNT 仍为复位初值 0）
                    if self.is_dir_down() && self.cnt.get() == 0 {
                        self.cnt.set(self.arr_shadow.get());
                    }
                }
            }
            0x04 => self.cr2.set(val),
            0x08 => self.smcr.set(val),
            0x0C => self.dier.set(val),
            0x10 => {
                // SR：RC_W0 写 0 清零，写 1 无效
                self.sr.set(self.sr.get() & (val | !SR_WRITE_MASK));
            }
            0x14 => {
                // EGR：写操作触发软件事件（只写）
                if val & EGR_UG != 0 {
                    self.sw_update();
                }
                // TODO: CC 软件触发（EGR_CC1G ~ EGR_CC4G）
                let _ = EGR_CC1G | EGR_CC2G | EGR_CC3G | EGR_CC4G;
            }
            0x18 => self.ccmr1.set(val),
            0x1C => self.ccmr2.set(val),
            0x20 => self.ccer.set(val),
            0x24 => self.cnt.set(val),
            0x28 => {
                self.psc.set(val & 0xFFFF);
                // PSC 写入立即同步到影子（实际在下一个更新事件生效，但写 PSC 后
                // 软件通常会通过 EGR.UG 触发更新，这里为了简化直接缓存）
            }
            0x2C => {
                let val16 = val & 0xFFFF; // STM32F1 定时器寄存器为 16-bit
                self.arr.set(val16);
                // ARPE=0 时立即生效
                if !self.is_arpe() {
                    self.arr_shadow.set(val16);
                }
            }
            0x34 => self.ccr1.set(val),
            0x38 => self.ccr2.set(val),
            0x3C => self.ccr3.set(val),
            0x40 => self.ccr4.set(val),
            0x48 => self.dcr.set(val),
            0x4C => self.dmar.set(val),
            _ => {}
        }
    }

    /// 每个定时器输入时钟周期调用一次。
    /// 若 CEN=0 则不计数。
    fn tick(&mut self) {
        self.tick_n(1);
    }

    fn tick_n(&mut self, cycles: u32) {
        if cycles == 0 {
            return;
        }

        if !self.is_enabled() {
            return;
        }

        // ---- 预分频 ----
        let psc_shadow = self.psc_shadow.get();
        let div = psc_shadow.wrapping_add(1);
        if div == 0 {
            return;
        }

        let psc_cnt = self.psc_cnt.get();
        let total = psc_cnt.saturating_add(cycles);
        let cnt_steps = total / div;
        let new_psc_cnt = total % div;
        self.psc_cnt.set(new_psc_cnt);

        if cnt_steps == 0 {
            return;
        }

        // ---- CNT 步进 ----
        let start_cnt = self.cnt.get();
        let arr = self.arr_shadow.get();
        let down = self.is_dir_down();

        if !down {
            let dist_to_wrap = arr.saturating_sub(start_cnt).saturating_add(1);
            if cnt_steps < dist_to_wrap {
                let end_cnt = start_cnt.saturating_add(cnt_steps);
                self.cnt.set(end_cnt);
                self.set_cc_flags_batch(start_cnt, end_cnt, arr, false, 0, false);
                return;
            }

            let remain = cnt_steps - dist_to_wrap;
            let period = arr.saturating_add(1);
            let wraps = 1 + (remain / period);
            let end_cnt = remain % period;

            self.set_cc_flags_batch(start_cnt, end_cnt, arr, true, wraps, false);
            self.cnt.set(end_cnt);
            self.do_update_event();

            if self.is_one_pulse() {
                let cr1 = self.cr1.get() & !CR1_CEN;
                self.cr1.set(cr1);
                self.cnt.set(0);
            }
            return;
        }

        let dist_to_wrap = start_cnt.saturating_add(1);
        if cnt_steps < dist_to_wrap {
            let end_cnt = start_cnt.saturating_sub(cnt_steps);
            self.cnt.set(end_cnt);
            self.set_cc_flags_batch(start_cnt, end_cnt, arr, false, 0, true);
            return;
        }

        let remain = cnt_steps - dist_to_wrap;
        let period = arr.saturating_add(1);
        let wraps = 1 + (remain / period);
        let rem = remain % period;
        let end_cnt = arr.saturating_sub(rem);

        self.set_cc_flags_batch(start_cnt, end_cnt, arr, true, wraps, true);
        self.cnt.set(end_cnt);
        self.do_update_event();

        if self.is_one_pulse() {
            let cr1 = self.cr1.get() & !CR1_CEN;
            self.cr1.set(cr1);
            self.cnt.set(arr);
        }
    }

    #[inline(always)]
    fn needs_tick(&self) -> bool {
        true
    }

    #[inline(always)]
    fn is_tick_active(&self) -> bool {
        self.is_enabled()
    }

    #[inline(always)]
    fn next_event_in_cycles(&self) -> Option<u32> {
        if !self.is_enabled() {
            return None;
        }

        let div = self.psc_shadow.get().wrapping_add(1);
        if div == 0 {
            return Some(1);
        }

        let psc_cnt = self.psc_cnt.get() % div;
        Some(div.saturating_sub(psc_cnt).max(1))
    }

    // ---- IRQ 接口（供 Bus::drain_pending_irqs 调用）----

    #[inline(always)]
    fn pending_irq(&self) -> Option<u32> {
        if self.interrupt_pending.get() {
            Some(self.irq_num)
        } else {
            None
        }
    }

    #[inline(always)]
    fn clear_pending_irq(&mut self) {
        self.interrupt_pending.set(false);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// ---------------------------------------------------------------------------
// 单元测试
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make_tim2() -> GeneralTimer {
        GeneralTimer::new(0x4000_0000, 0x4000_03FF, 28)
    }

    // ---- CR1/基本使能 ----

    #[test]
    fn timer_disabled_by_default() {
        let t = make_tim2();
        assert!(!t.is_enabled());
    }

    #[test]
    fn timer_enable_sets_cen() {
        let mut t = make_tim2();
        t.write(0x4000_0000, CR1_CEN); // CR1
        assert!(t.is_enabled());
    }

    // ---- 向上计数溢出 ----

    #[test]
    fn timer_upcounting_overflow_sets_uif() {
        let mut t = make_tim2();
        // PSC=0（不分频）, ARR=3
        t.write(0x4000_002C, 3); // ARR
        t.write(0x4000_0014, EGR_UG); // EGR.UG 触发更新，CNT 复位
        t.write(0x4000_0000, CR1_CEN); // 启用

        // tick 4 次：CNT 0→1→2→3 后溢出
        t.tick(); // CNT=1
        t.tick(); // CNT=2
        t.tick(); // CNT=3
        t.tick(); // 溢出，CNT=0，UIF 置位

        let sr = t.read(0x4000_0010);
        assert_ne!(sr & SR_UIF, 0, "Should set UIF on overflow");
        assert_eq!(t.cnt.get(), 0);
    }

    #[test]
    fn timer_uif_triggers_interrupt_when_uie_set() {
        let mut t = make_tim2();
        t.write(0x4000_002C, 2); // ARR=2
        t.write(0x4000_000C, DIER_UIE); // 使能更新中断
        t.write(0x4000_0014, EGR_UG); // 复位 CNT
        // EGR.UG 本身也触发了一次更新，此时 UIE 已设，中断会挂起
        // 为清干净，清 SR 并清中断挂起
        t.write(0x4000_0010, 0); // 清 UIF
        t.interrupt_pending.set(false);

        t.write(0x4000_0000, CR1_CEN);
        t.tick(); // CNT=1
        t.tick(); // CNT=2
        t.tick(); // 溢出

        assert!(
            t.is_interrupt_pending(),
            "IRQ should be pending after overflow with UIE set"
        );
    }

    #[test]
    fn timer_no_interrupt_without_uie() {
        let mut t = make_tim2();
        t.write(0x4000_002C, 1); // ARR=1
        // DIER.UIE 未置位
        t.write(0x4000_0014, EGR_UG);
        t.interrupt_pending.set(false);
        t.write(0x4000_0000, CR1_CEN);
        t.tick();
        t.tick(); // 溢出

        assert!(!t.is_interrupt_pending(), "No IRQ without UIE");
    }

    // ---- 向下计数 ----

    #[test]
    fn timer_downcounting_underflow_reloads_arr() {
        let mut t = make_tim2();
        t.write(0x4000_002C, 4); // ARR=4
        t.write(0x4000_0000, CR1_DIR | CR1_CEN); // 向下 + 使能
        // CNT 从 ARR(4) 开始倒数
        // tick 5 次 → 4→3→2→1→0 → 溢出重装 CNT=ARR=4
        for _ in 0..5 {
            t.tick();
        }
        let sr = t.read(0x4000_0010);
        assert_ne!(sr & SR_UIF, 0, "Underflow should set UIF");
        assert_eq!(t.cnt.get(), 4, "CNT should reload to ARR");
    }

    // ---- 预分频 ----

    #[test]
    fn timer_prescaler_divides_clock() {
        let mut t = make_tim2();
        // PSC=3 → 4 个 tick 才计一次
        t.write(0x4000_0028, 3); // PSC=3
        t.write(0x4000_002C, 10); // ARR=10
        t.write(0x4000_0014, EGR_UG); // 同步 PSC shadow
        t.interrupt_pending.set(false);
        t.write(0x4000_0000, CR1_CEN);

        // 3 个 tick 不应改变 CNT
        t.tick();
        t.tick();
        t.tick();
        assert_eq!(t.cnt.get(), 0, "CNT should not advance before PSC expires");

        // 第 4 个 tick CNT 变成 1
        t.tick();
        assert_eq!(t.cnt.get(), 1, "CNT should advance after PSC+1 ticks");
    }

    // ---- SR 写清零 ----

    #[test]
    fn timer_sr_write_zero_clears_uif() {
        let mut t = make_tim2();
        t.sr.set(SR_UIF);
        // 写 SR = 0 应清零 UIF（RC_W0 行为）
        t.write(0x4000_0010, 0);
        assert_eq!(t.read(0x4000_0010) & SR_UIF, 0);
    }

    #[test]
    fn timer_sr_write_one_does_not_set_cleared_bit() {
        let mut t = make_tim2();
        t.sr.set(0); // UIF 为 0
        // 写 SR = UIF（写 1 不应置位，只有硬件能置位）
        t.write(0x4000_0010, SR_UIF);
        assert_eq!(t.read(0x4000_0010) & SR_UIF, 0);
    }

    // ---- OPM 单脉冲 ----

    #[test]
    fn timer_one_pulse_mode_stops_after_overflow() {
        let mut t = make_tim2();
        t.write(0x4000_002C, 2); // ARR=2
        t.write(0x4000_0014, EGR_UG);
        t.interrupt_pending.set(false);
        t.write(0x4000_0000, CR1_CEN | CR1_OPM);

        t.tick();
        t.tick();
        t.tick(); // 溢出，CEN 应自动清零

        assert!(!t.is_enabled(), "OPM: CEN should clear after overflow");
    }

    // ---- 捕获/比较匹配 ----

    #[test]
    fn timer_ccr1_match_sets_cc1if() {
        let mut t = make_tim2();
        t.write(0x4000_0034, 2); // CCR1=2
        t.write(0x4000_002C, 10); // ARR=10
        t.write(0x4000_0000, CR1_CEN);

        // CNT 从 0 经过 2 时应触发 CC1IF
        t.tick(); // CNT=1
        t.tick(); // CNT=2 → 匹配检测在 tick 前（检查旧值 CNT=2 此时匹配）
        let sr = t.read(0x4000_0010);
        assert_ne!(sr & SR_CC1IF, 0, "CC1IF should be set when CNT==CCR1");
    }

    // ---- pending_irq/clear_pending_irq 接口 ----

    #[test]
    fn pending_irq_returns_irq_num_when_pending() {
        let mut t = make_tim2(); // irq_num=28
        t.interrupt_pending.set(true);
        assert_eq!(t.pending_irq(), Some(28));
    }

    #[test]
    fn clear_pending_irq_clears_flag() {
        let mut t = make_tim2();
        t.interrupt_pending.set(true);
        t.clear_pending_irq();
        assert_eq!(t.pending_irq(), None);
    }
}
