# cortex_m4_sim

一个基于 Rust 实现的 Cortex-M4 指令级模拟器，当前面向 Thumb/M-Class 固件执行场景，集成了 AXF 反汇编、解释执行器、基于 Cranelift 的块级 JIT，以及一组面向 STM32 风格片上外设的仿真实现。

项目当前更接近“内核+样例固件驱动”的研发工具，而不是一个已经完成参数化封装的通用命令行模拟器。

## 项目能力

- 解析 AXF/ELF 固件，读取向量表中的初始栈指针和 Reset_Handler。
- 使用 Capstone 对 Thumb 指令流进行反汇编，并将详细结果输出到 `disassembly_detail.asm`。
- 提供两条执行路径：
  - 解释执行器
  - 基于 Cranelift 的块级 JIT 执行器
- 模拟基础 CPU 上下文：
  - 通用寄存器 R0-R15
  - APSR
  - MSP/PSP 基本访问
  - Thumb 可见 PC 更新
  - 异常/中断相关状态
- 提供片上外设模拟：
  - GPIO
  - AFIO
  - RCC
  - Flash 接口
  - USART1/UART
  - SysTick
  - NVIC
  - TIM2-TIM5 通用定时器
- 支持外设调度与 IRQ 注入，CPU 与外设总线解耦。
- 包含指令执行性能统计与 CSV 报表生成逻辑。

## 当前默认行为

当前入口在 `src/main.rs`，默认会：

- 加载根目录下的 `uart_loop.axf`
- 将反汇编结果写入 `disassembly_detail.asm`
- 根据环境变量选择解释器或 JIT 模式

这意味着当前版本并不通过命令行参数指定输入文件；如果要切换样例固件，需要修改 `src/main.rs` 中的 `input_path`。

## 架构概览

### 1. 固件解析与反汇编

`src/disassembler` 负责：

- 读取 AXF/ELF 文件
- 识别 `$t` / `$d` 标记划分代码与数据区域
- 提取 Reset_Handler、初始 SP 和数据区内容
- 将 Thumb 指令解析为内部可用的执行输入

### 2. CPU 与总线

`src/cpu.rs` 和 `src/peripheral/bus.rs` 负责：

- CPU 寄存器、内存与异常状态维护
- Flash/RAM/外设/PPB 地址空间映射
- 外设读写分发
- 外设 tick 调度
- IRQ 事件汇总与 CPU 侧消费

### 3. 指令执行

`src/opcodes` 负责：

- 指令定义表
- Capstone 指令到内部执行表示的映射
- 解释执行路径中的指令匹配与执行

当前仓库中已经组织了以下指令组定义：

- ADR
- Bit Field
- Bit Operation
- Branch
- Breakpoint
- Calculate
- CMP/CMN
- Compare and Branch
- Extend
- Hint
- IT
- LDM
- LDR/LDRB/LDRH/LDRSB/LDRSH
- MOV/MOVS
- NOP
- Shift
- Stack
- STM
- STR/STRB/STRH
- TST

### 4. JIT 执行

`src/jit_engine` 基于 Cranelift 实现块级编译执行：

- 先构建 JIT block table
- 将一段顺序指令 lowering 为 Cranelift IR
- 已对部分热点访存/寄存器路径进行了 helper 减少与块级缓存优化
- 不支持的模式仍可回退到 Rust 侧解释执行逻辑

## 目录说明

- `src/main.rs`: 程序入口，装配样例固件、CPU、总线与外设
- `src/simulator.rs`: 主模拟循环，统一处理解释器/JIT 执行、节流与性能打印
- `src/cpu.rs`: CPU 状态、存储访问、异常与中断处理
- `src/disassembler/`: AXF 解析与反汇编输出
- `src/opcodes/`: 指令定义与解释执行
- `src/jit_engine/`: Cranelift JIT 实现
- `src/peripheral/`: 外设与总线模型
- `src/perf_tests.rs`: 指令性能统计与报表测试
- `perf_reports/`: 生成的性能 CSV 报表
- `*.axf`: 仓库内附带的示例固件

## 环境要求

- Rust 稳定版工具链
- 能正常编译 Capstone 相关依赖的本地构建环境
- Windows 环境下建议直接使用 PowerShell 运行

## 构建

```powershell
cargo build
```

发布构建：

```powershell
cargo build --release
```

## 运行

### 解释执行模式

```powershell
cargo run --release
```

### JIT 模式

```powershell
$env:SIM_USE_BLOCK="1"
cargo run --release
```

运行后通常会看到：

- 初始栈指针与 Reset_Handler 输出
- 周期性的执行频率统计
- GPIO/UART 等外设相关调试信息
- 生成的 `disassembly_detail.asm`

## 可用环境变量

运行时支持以下环境变量：

- `SIM_USE_BLOCK`
  - `1`: 使用块级 JIT
  - `0` 或未设置: 使用解释执行器
- `SIM_REPORT_WINDOW`
  - 控制吞吐率打印窗口，默认 `10000`
- `SIM_NO_THROTTLE`
  - 非 `0` 时关闭节流，尽可能全速运行
- `SIM_PERIPH_TICK_BATCH`
  - 外设 tick 批次大小，默认 `1`
- `SIM_TRACE_INSN`
  - 非 `0` 时输出指令级 trace
- `SIM_TRACE_LIMIT`
  - 限制 trace 输出条数，`0` 表示不限
- `SIM_JIT_STATS`
  - JIT 模式下输出 JIT block 统计信息

示例：

```powershell
$env:SIM_USE_BLOCK="1"
$env:SIM_JIT_STATS="1"
$env:SIM_NO_THROTTLE="1"
cargo run --release
```

## 测试与性能分析

运行常规测试：

```powershell
cargo test
```

运行指令性能测试并打印详细结果：

```powershell
cargo test --release perf_instruction_ -- --nocapture
```

性能测试会基于仓库中的样例 AXF 收集统计，并在 `perf_reports/` 下生成例如以下文件：

- `instruction_exec_mnemonic_sorted.csv`
- `instruction_exec_definition_sorted.csv`

## 已实现外设要点

### GPIO

- 实现了 CRL/CRH/IDR/ODR/BSRR/BRR/LCKR 基本行为
- 具备引脚翻转检测与状态锁存

### AFIO

- 支持 USART1 默认引脚与重映射配置
- 支持自定义 GPIO bridge 配置，用于在引脚之间转发逻辑电平

### UART

- 建模了 SR/DR/BRR/CR1 基本寄存器
- 支持简化的 TX 串行发送状态机
- 支持基于 RX 线电平采样接收字节

### TIM2-TIM5

- 支持预分频、自动重装、更新事件与部分捕获比较标志
- 可向 NVIC 触发挂起中断

### SysTick/NVIC

- 支持基本倒计时、COUNTFLAG、中断挂起与激活逻辑

## 限制与注意事项

- 当前入口写死为 `uart_loop.axf`，尚未提供完整 CLI 参数接口。
- 指令覆盖仍是逐步扩展的，并非完整 Cortex-M4/Thumb-2 ISA。
- 外设实现以当前样例固件需求为导向，不等同于完整 STM32 外设级精确仿真。
- 仓库中部分外设路径包含调试输出，长时间运行时可能影响观测体验与性能。
- 仿真目标目前更适合作为研究、验证和性能优化平台，而不是生产级固件验证器。

## 后续可扩展方向

- 增加命令行参数，支持指定输入 AXF、输出路径和运行模式
- 继续补齐 Thumb-2 指令与异常模型
- 丰富更多 STM32 外设模型
- 将调试输出分级到日志系统而不是直接打印
- 补充回归样例和端到端固件测试

## 许可证

本项目采用 MIT License，详见 `LICENSE`。