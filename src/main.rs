fn hex_ass_test(cs: &Capstone, insns: capstone::Instructions<'_>, file_path: &str) {
    let mut file = File::create(file_path).expect("Unable to create output file");
    for i in insns.iter() {
        // 打印基础信息

        writeln!(
            file,
            "0x{:08x}: {:<8} {},  id:{}  len:{} ",
            i.address(),
            i.mnemonic().unwrap_or(""),
            i.op_str().unwrap_or(""),
            i.id().0,
            i.len(),
        )
        .unwrap();

        // 2. 获取详细信息 (Detail)
        if let Ok(detail) = cs.insn_detail(&i) {
            let arch_detail = detail.arch_detail();
            let arm_detail = arch_detail.arm().unwrap(); // 获取 ARM 特有的详细数据

            let ops = arm_detail.operands();
            writeln!(file, "  └─ Operands count: {}", ops.len()).unwrap(); //.contains("],")

            writeln!(
                file,
                "--- Operands Detail --- writeback:{}  Cond:{}",
                arm_detail.writeback(),
                format!("{:?}", arm_detail.cc()),
                // arm_detail.post_index(),
            )
            .unwrap();
            if i.op_str().unwrap_or("").contains("],") {
                writeln!(file, "-----post_index: true").unwrap();
            }
            for (index, op) in ops.enumerate() {
                match op.op_type {
                    ArmOperandType::Reg(reg_id) => {
                        writeln!(
                            file,
                            "     [Op {}] Type: Register, Value: {}",
                            index,
                            cs.reg_name(reg_id).unwrap_or("unknown".to_string())
                        )
                        .unwrap();
                    }
                    ArmOperandType::Imm(imm) => {
                        writeln!(
                            file,
                            "     [Op {}] Type: Immediate, Value: 0x{:x}",
                            index, imm
                        )
                        .unwrap();
                    }
                    ArmOperandType::Mem(mem) => {
                        writeln!(
                            file,
                            "     [Op {}] Type: Memory, Base: {}, Index: {}, Scale: {}, Disp: {}",
                            index,
                            cs.reg_name(mem.base()).unwrap_or("none".to_string()),
                            cs.reg_name(mem.index()).unwrap_or("none".to_string()),
                            mem.scale(),
                            mem.disp()
                        )
                        .unwrap();
                    }
                    _ => writeln!(file, "     [Op {}] Type: Other", index).unwrap(),
                }

                // 检查是否有移位 (Shift) 信息 (常见于 ARM/Thumb-2)
                let shift = op.shift;
                match shift {
                    ArmShift::Invalid => {
                        // 无移位，不做处理
                    }
                    ArmShift::Lsl(val) => {
                        writeln!(file, "        └─ Shift: LSL, Value: {}", val).unwrap()
                    }
                    ArmShift::Lsr(val) => {
                        writeln!(file, "        └─ Shift: LSR, Value: {}", val).unwrap()
                    }
                    ArmShift::Asr(val) => {
                        writeln!(file, "        └─ Shift: ASR, Value: {}", val).unwrap()
                    }
                    ArmShift::Ror(val) => {
                        writeln!(file, "        └─ Shift: ROR, Value: {}", val).unwrap()
                    }
                    ArmShift::Rrx(val) => {
                        writeln!(file, "        └─ Shift: RRX, Value: {}", val).unwrap()
                    }

                    ArmShift::LslReg(reg_id) => writeln!(
                        file,
                        "        └─ Shift: LSL (Reg), Value: {}",
                        cs.reg_name(reg_id).unwrap_or("unknown".to_string())
                    )
                    .unwrap(),
                    ArmShift::LsrReg(reg_id) => writeln!(
                        file,
                        "        └─ Shift: LSR (Reg), Value: {}",
                        cs.reg_name(reg_id).unwrap_or("unknown".to_string())
                    )
                    .unwrap(),
                    ArmShift::AsrReg(reg_id) => writeln!(
                        file,
                        "        └─ Shift: ASR (Reg), Value: {}",
                        cs.reg_name(reg_id).unwrap_or("unknown".to_string())
                    )
                    .unwrap(),
                    ArmShift::RorReg(reg_id) => writeln!(
                        file,
                        "        └─ Shift: ROR (Reg), Value: {}",
                        cs.reg_name(reg_id).unwrap_or("unknown".to_string())
                    )
                    .unwrap(),
                    ArmShift::RrxReg(reg_id) => writeln!(
                        file,
                        "        └─ Shift: RRX (Reg), Value: {}",
                        cs.reg_name(reg_id).unwrap_or("unknown".to_string())
                    )
                    .unwrap(),
                }
            }
        }
        writeln!(file, "---").unwrap();
    }
}

use capstone::arch::arm::{ArmOperandType, ArmShift};
use capstone::prelude::*;
use object::{Object, ObjectSection, ObjectSymbol, SectionKind};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};

mod context;
mod cpu;
use crate::context::CpuContext;
use crate::cpu::Cpu;
mod opcodes;
mod peripheral;
mod simulator;
use crate::opcodes::instruction::{Cpu_InstrTable, Cpu_Instruction, OpcodeTable};
use crate::opcodes::opcode::{ArmOpcode, Opcode};
use crate::peripheral::bus::Bus;
use crate::peripheral::flash::Flash;
use crate::peripheral::gpio::Gpio;
use crate::peripheral::rcc::Rcc;
use crate::peripheral::scb::Scb;
use crate::peripheral::systick::SysTick;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegionKind {
    Thumb,
    Data,
}

/// 指令详细信息结构
#[derive(Debug)]
struct InstructionInfo {
    address: u64,
    bytes: Vec<u8>,
    mnemonic: String,
    op_str: String,
    operands: Vec<OperandInfo>,
    condition: String,
    writeback: bool,
    post_index: bool,
}

/// 操作数信息
#[derive(Debug)]
struct OperandInfo {
    op_type: OperandType,
    shift: ShiftInfo,
}

/// 操作数类型
#[derive(Debug)]
enum OperandType {
    Register(String),
    Immediate(i64),
    Memory {
        base: String,
        index: String,
        scale: i32,
        disp: i64,
    },
    Other,
}

/// 移位信息
#[derive(Debug)]
enum ShiftInfo {
    None,
    Lsl(u32),
    Lsr(u32),
    Asr(u32),
    Ror(u32),
    Rrx(u32),
    LslReg(String),
    LsrReg(String),
    AsrReg(String),
    RorReg(String),
    RrxReg(String),
}

/// 数据区域信息 (DCW)
#[derive(Debug, Clone)]
struct DataInfo {
    address: u64,
    bytes: Vec<u8>,
    value: String,
}

/// 解析后的区域
#[derive(Debug)]
#[allow(dead_code)]
enum ParsedRegion {
    Code(Vec<InstructionInfo>),
    Data(Vec<DataInfo>),
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = "target.axf";
    let output_path = "disassembly_detail.asm";

    // 调用封装的反汇编函数
    let (result, cs, code_segments, dcw_data, initial_sp, reset_handler_ptr, _reset_handler_addr) =
        disassemble_from_reset_handler(input_path, output_path)?;
    println!(
        "Initial SP: 0x{:08X}, Reset_Handler Ptr: 0x{:08X}",
        initial_sp, reset_handler_ptr
    );

    // 使用返回的 Capstone 和代码片段获取 Instructions
    // 我们需要保存所有的 Instructions 对象以保持 Insn 的生命周期
    let mut all_insns_storage = Vec::new();

    for (addr, bytes) in &code_segments {
        let insns = cs.disasm_all(bytes, *addr).map_err(|e| e.to_string())?;
        all_insns_storage.push(insns);
    }

    let opcode_table = OpcodeTable::new();
    let table = opcode_table.get_table();
    let Cpu_InstrTable = {
        let mut instr_table = Cpu_InstrTable::new();
        // 遍历所有存储的指令集
        for insns in &all_insns_storage {
            for i in insns.iter() {
                let key = i.id().0 as u16;
                if let Some(instructions) = table.get(&key) {
                    for instruction in instructions {
                        let arm_opcode = ArmOpcode::new(&cs, &i);
                        let cpu_instruction =
                            Cpu_Instruction::new(instruction.clone(), arm_opcode.unwrap());
                        instr_table.add_instruction(cpu_instruction);
                    }
                }
            }
        }
        instr_table
    };
    let target_addr = 0x08000510;
    if let Some(instr) = Cpu_InstrTable.table.get(&target_addr) {
        println!(
            "Found instruction at 0x{:08X}: {} {}",
            target_addr,
            instr.data.insn.mnemonic().unwrap_or(""),
            instr.data.insn.op_str().unwrap_or("")
        );
    } else {
        println!(
            "Instruction at 0x{:08X} not found in Cpu_InstrTable",
            target_addr
        );
    }

    let gpioc = Gpio::new(0x4001_1000, 0x4001_13FF);
    let rcc = Rcc::new(0x4002_0000, 0x4002_1024);
    let flash_interface = Flash::new(0x40022000, 0x4002201C);
    let mut bus = Bus::new();
    bus.register_peripheral(Box::new(gpioc));
    bus.register_peripheral(Box::new(flash_interface));
    bus.register_peripheral(Box::new(rcc));

    let mut ppb = Bus::new();

    let scb = Scb::new(0xE000_ED00, 0xE000_ED3C);
    let systick = SysTick::new(0xE000E010, 0xE000E01F);
    ppb.register_peripheral(Box::new(systick));
    ppb.register_peripheral(Box::new(scb));

    let cpu = Cpu::new(48_000_000, 1, bus, ppb);

    let mut simulator = simulator::Simulator::new(cpu);
    simulator.sim_reset(dcw_data, initial_sp, reset_handler_ptr);
    simulator.sim_loop(Cpu_InstrTable);

    Ok(())
}

/// 从 Reset_Handler 开始反汇编，输出详细信息到文件
///
/// # 参数
/// - `input_path`: 输入的 AXF/ELF 文件路径
/// - `output_path`: 输出的反汇编文件路径
///
/// # 返回
/// - `DisassemblyResult`: 反汇编统计信息
/// - `Capstone`: Capstone 实例（可用于调用 disasm_all 获取 Instructions）
/// - `Vec<u8>`: 从 Reset_Handler 开始的代码段原始字节
/// - `BTreeMap<u64, u16>`: DCW 数据 (地址 -> 16位数据值)
///
/// # 使用示例
/// ```ignore
/// let (result, cs, code_bytes, dcw_data) = disassemble_from_reset_handler("target.axf", "output.asm")?;
/// let insns: capstone::Instructions = cs.disasm_all(&code_bytes, result.start_address)?;
/// for insn in insns.iter() {
///     println!("{:08X}: {} {}", insn.address(), insn.mnemonic().unwrap_or(""), insn.op_str().unwrap_or(""));
/// }
/// ```
pub fn disassemble_from_reset_handler(
    input_path: &str,
    output_path: &str,
) -> Result<
    (
        DisassemblyResult,
        Capstone,
        Vec<(u64, Vec<u8>)>,
        BTreeMap<u32, u32>,
        u32,
        u32,
        u32,
    ),
    Box<dyn Error>,
> {
    let bin_data = std::fs::read(input_path)?;
    let obj_file = object::File::parse(&*bin_data)?;

    // 查找 SP 初始值 (0x08000000) 和 Reset_Handler 地址 (0x08000004)
    let mut initial_sp = 0u32;
    let mut reset_handler_ptr = 0u32;

    for section in obj_file.sections() {
        let addr = section.address();
        let size = section.size();
        if let Ok(data) = section.data() {
            if addr <= 0x0800_0000 && 0x0800_0000 < addr + size {
                let offset = (0x0800_0000 - addr) as usize;
                if offset + 4 <= data.len() {
                    initial_sp = u32::from_le_bytes(data[offset..offset + 4].try_into()?);
                }
            }
            if addr <= 0x0800_0004 && 0x0800_0004 < addr + size {
                let offset = (0x0800_0004 - addr) as usize;
                if offset + 4 <= data.len() {
                    reset_handler_ptr = u32::from_le_bytes(data[offset..offset + 4].try_into()?);
                }
            }
        }
    }

    // 1. 初始化 Capstone，开启 Detail 模式
    let cs = Capstone::new()
        .arm()
        .mode(arch::arm::ArchMode::Thumb)
        .extra_mode([arch::arm::ArchExtraMode::MClass].iter().copied())
        .detail(true)
        .build()
        .expect("Failed to create Capstone object");

    // 2. 收集标记与函数名
    let mut markers = BTreeMap::new();
    let mut func_names: BTreeMap<u64, &str> = BTreeMap::new();
    let mut reset_handler_addr: Option<u64> = None;

    for symbol in obj_file.symbols() {
        let addr = symbol.address() & !1;
        if let Ok(name) = symbol.name() {
            if name.starts_with("$t") {
                markers.insert(addr, RegionKind::Thumb);
            } else if name.starts_with("$d") {
                markers.insert(addr, RegionKind::Data);
            } else if symbol.kind() == object::SymbolKind::Text {
                func_names.insert(addr, name);
                // 查找 Reset_Handler
                if name.to_lowercase().contains("reset_handler") || name == "Reset_Handler" {
                    reset_handler_addr = Some(addr);
                }
            }
        }
    }

    // 3. 收集所有指令和数据
    let mut all_instructions: Vec<InstructionInfo> = Vec::new();
    let mut all_data: Vec<DataInfo> = Vec::new();
    let mut addr_to_func: BTreeMap<u64, String> = BTreeMap::new();

    // 用于返回的数据结构
    let mut code_segments: Vec<(u64, Vec<u8>)> = Vec::new(); // (起始地址, 代码字节)
    let mut dcw_data: BTreeMap<u32, u32> = BTreeMap::new(); // 地址 -> 32位数据

    for section in obj_file.sections() {
        if section.kind() == SectionKind::Text {
            let section_data = section.data()?;
            let section_start = section.address();
            let section_end = section_start + section.size();

            let mut section_markers: Vec<(u64, RegionKind)> = markers
                .range(section_start..section_end)
                .map(|(&a, &k)| (a, k))
                .collect();

            if section_markers.is_empty() || section_markers[0].0 > section_start {
                section_markers.insert(0, (section_start, RegionKind::Data));
            }

            for i in 0..section_markers.len() {
                let (start, kind) = section_markers[i];
                let end = if i + 1 < section_markers.len() {
                    section_markers[i + 1].0
                } else {
                    section_end
                };
                let chunk =
                    &section_data[(start - section_start) as usize..(end - section_start) as usize];

                // 记录函数名
                if let Some(name) = func_names.get(&start) {
                    addr_to_func.insert(start, name.to_string());
                }

                match kind {
                    RegionKind::Thumb => {
                        // 保存代码段
                        code_segments.push((start, chunk.to_vec()));

                        let insns = cs.disasm_all(chunk, start).map_err(|e| e.to_string())?;
                        for insn in insns.iter() {
                            let info = parse_instruction(&cs, &insn)?;
                            all_instructions.push(info);
                        }
                    }
                    RegionKind::Data => {
                        for (idx, word) in chunk.chunks(2).enumerate() {
                            let curr_addr = start + (idx * 2) as u64;
                            let val_u16 = if word.len() == 2 {
                                u16::from_le_bytes([word[0], word[1]])
                            } else {
                                word[0] as u16
                            };
                            let val = format!("0x{:04X}", val_u16);

                            // 保存到返回的 BTreeMap
                            dcw_data.insert(curr_addr as u32, val_u16 as u32);

                            all_data.push(DataInfo {
                                address: curr_addr,
                                bytes: word.to_vec(),
                                value: val,
                            });
                        }
                    }
                }
            }
        }
    }

    // 4. 从 reset_handler 开始排序输出
    let start_addr = 0x0800_0000;

    // 过滤出从 reset_handler 开始的指令
    let filtered_instructions: Vec<&InstructionInfo> = all_instructions
        .iter()
        .filter(|i| i.address >= start_addr)
        .collect();

    // 5. 过滤出有效的代码段（保持原来的分块结构）
    let filtered_code_segments: Vec<(u64, Vec<u8>)> = code_segments
        .into_iter()
        .filter_map(|(addr, bytes)| {
            let end_addr = addr + bytes.len() as u64;
            if end_addr <= start_addr {
                // 完全在 start_addr 之前，丢弃
                None
            } else if addr < start_addr {
                // 部分重叠，截取后半部分
                let offset = (start_addr - addr) as usize;
                Some((start_addr, bytes[offset..].to_vec()))
            } else {
                // 完全在 start_addr 之后，保留
                Some((addr, bytes))
            }
        })
        .collect();

    // 6. 输出到文件
    write_disassembly_to_file(
        output_path,
        start_addr,
        &filtered_instructions,
        &all_data,
        &addr_to_func,
    )?;

    let result = DisassemblyResult {
        start_address: start_addr,
        instruction_count: filtered_instructions.len(),
        data_word_count: all_data.len(),
        output_file: output_path.to_string(),
    };

    Ok((
        result,
        cs,
        filtered_code_segments,
        dcw_data,
        initial_sp,
        reset_handler_ptr,
        reset_handler_addr.unwrap_or(0) as u32,
    ))
}
/// 仅解析文件，不输出到文件，返回 Capstone、代码字节和 DCW 数据
///
/// # 参数
/// - `input_path`: 输入的 AXF/ELF 文件路径
///
/// # 返回
/// - `u64`: Reset_Handler 起始地址
/// - `Capstone`: Capstone 实例
/// - `Vec<u8>`: 从 Reset_Handler 开始的代码段原始字节
/// - `BTreeMap<u64, u16>`: DCW 数据 (地址 -> 16位数据值)
///
/// # 使用示例
/// ```ignore
/// let (start_addr, cs, code_bytes, dcw_data) = parse_axf_file("target.axf")?;
/// let insns: capstone::Instructions = cs.disasm_all(&code_bytes, start_addr)?;
/// ```
pub fn parse_axf_file(
    input_path: &str,
) -> Result<(u64, Capstone, Vec<u8>, BTreeMap<u32, u32>), Box<dyn Error>> {
    let bin_data = std::fs::read(input_path)?;
    let obj_file = object::File::parse(&*bin_data)?;

    // 初始化 Capstone
    let cs = Capstone::new()
        .arm()
        .mode(arch::arm::ArchMode::Thumb)
        .extra_mode([arch::arm::ArchExtraMode::MClass].iter().copied())
        .detail(true)
        .build()
        .expect("Failed to create Capstone object");

    // 收集标记与函数名
    let mut markers = BTreeMap::new();
    let mut reset_handler_addr: Option<u64> = None;

    for symbol in obj_file.symbols() {
        let addr = symbol.address() & !1;
        if let Ok(name) = symbol.name() {
            if name.starts_with("$t") {
                markers.insert(addr, RegionKind::Thumb);
            } else if name.starts_with("$d") {
                markers.insert(addr, RegionKind::Data);
            } else if symbol.kind() == object::SymbolKind::Text {
                if name.to_lowercase().contains("reset_handler") || name == "Reset_Handler" {
                    reset_handler_addr = Some(addr);
                }
            }
        }
    }

    let mut code_segments: Vec<(u64, Vec<u8>)> = Vec::new();
    let mut dcw_data: BTreeMap<u32, u32> = BTreeMap::new();

    for section in obj_file.sections() {
        if section.kind() == SectionKind::Text {
            let section_data = section.data()?;
            let section_start = section.address();
            let section_end = section_start + section.size();

            let mut section_markers: Vec<(u64, RegionKind)> = markers
                .range(section_start..section_end)
                .map(|(&a, &k)| (a, k))
                .collect();

            if section_markers.is_empty() || section_markers[0].0 > section_start {
                section_markers.insert(0, (section_start, RegionKind::Data));
            }

            for i in 0..section_markers.len() {
                let (start, kind) = section_markers[i];
                let end = if i + 1 < section_markers.len() {
                    section_markers[i + 1].0
                } else {
                    section_end
                };
                let chunk =
                    &section_data[(start - section_start) as usize..(end - section_start) as usize];

                match kind {
                    RegionKind::Thumb => {
                        code_segments.push((start, chunk.to_vec()));
                    }
                    RegionKind::Data => {
                        for (idx, word) in chunk.chunks(2).enumerate() {
                            let curr_addr = start + (idx * 2) as u64;
                            let val_u16 = if word.len() == 2 {
                                u16::from_le_bytes([word[0], word[1]])
                            } else {
                                word[0] as u16
                            };
                            dcw_data.insert(curr_addr as u32, val_u16 as u32);
                        }
                    }
                }
            }
        }
    }

    let start_addr = reset_handler_addr.unwrap_or(0);

    // 从 section 中提取从 reset_handler 开始的完整字节（保持地址连续性）
    let mut filtered_code_bytes: Vec<u8> = Vec::new();
    for section in obj_file.sections() {
        if section.kind() == SectionKind::Text {
            let section_data = section.data().unwrap_or(&[]);
            let section_start = section.address();
            let section_end = section_start + section.size();

            if start_addr >= section_start && start_addr < section_end {
                let offset = (start_addr - section_start) as usize;
                filtered_code_bytes.extend(&section_data[offset..]);
            } else if section_start >= start_addr {
                filtered_code_bytes.extend(section_data);
            }
        }
    }

    Ok((start_addr, cs, filtered_code_bytes, dcw_data))
}

/// 反汇编结果统计
#[derive(Debug, Clone)]
pub struct DisassemblyResult {
    pub start_address: u64,
    pub instruction_count: usize,
    pub data_word_count: usize,
    pub output_file: String,
}

/// 将反汇编结果写入文件
fn write_disassembly_to_file(
    output_path: &str,
    start_addr: u64,
    instructions: &[&InstructionInfo],
    data: &[DataInfo],
    addr_to_func: &BTreeMap<u64, String>,
) -> Result<(), Box<dyn Error>> {
    let mut writer = BufWriter::new(File::create(output_path)?);

    // 写入 UTF-8 BOM
    writer.write_all(&[0xEF, 0xBB, 0xBF])?;

    writeln!(
        writer,
        "================================================================================"
    )?;
    writeln!(writer, "                    ARM Cortex-M4 反汇编详细信息")?;
    writeln!(
        writer,
        "                    起始地址: 0x{:08X} (Reset_Handler)",
        start_addr
    )?;
    writeln!(
        writer,
        "================================================================================"
    )?;
    writeln!(writer)?;

    // 输出代码段
    writeln!(
        writer,
        "┌─────────────────────────────────────────────────────────────────────────────┐"
    )?;
    writeln!(
        writer,
        "│                              CODE SECTION                                    │"
    )?;
    writeln!(
        writer,
        "└─────────────────────────────────────────────────────────────────────────────┘"
    )?;
    writeln!(writer)?;

    for info in instructions {
        // 检查是否进入新函数
        if let Some(func_name) = addr_to_func.get(&info.address) {
            writeln!(writer)?;
            writeln!(
                writer,
                "════════════════════════════════════════════════════════════════════════════"
            )?;
            writeln!(writer, "FUNCTION: <{}>", func_name)?;
            writeln!(
                writer,
                "════════════════════════════════════════════════════════════════════════════"
            )?;
        }

        write_instruction_detail(&mut writer, info)?;
    }

    // 输出数据段 (DCW)
    writeln!(writer)?;
    writeln!(
        writer,
        "┌─────────────────────────────────────────────────────────────────────────────┐"
    )?;
    writeln!(
        writer,
        "│                              DATA SECTION (DCW)                              │"
    )?;
    writeln!(
        writer,
        "└─────────────────────────────────────────────────────────────────────────────┘"
    )?;
    writeln!(writer)?;

    // 按地址排序数据
    let mut sorted_data = data.to_vec();
    sorted_data.sort_by_key(|d| d.address);

    writeln!(writer, "{:<14} {:<14} {:<10}", "Address", "Hex", "Value")?;
    writeln!(writer, "{}", "-".repeat(40))?;

    for d in &sorted_data {
        writeln!(
            writer,
            "0x{:08X}     {:<14} DCW  {}",
            d.address,
            format_hex(&d.bytes),
            d.value
        )?;
    }

    writeln!(writer)?;
    writeln!(
        writer,
        "================================================================================"
    )?;
    writeln!(
        writer,
        "                    总计: {} 条指令, {} 个数据字",
        instructions.len(),
        sorted_data.len()
    )?;
    writeln!(
        writer,
        "================================================================================"
    )?;

    Ok(())
}

/// 解析单条指令的详细信息
fn parse_instruction(
    cs: &Capstone,
    insn: &capstone::Insn,
) -> Result<InstructionInfo, Box<dyn Error>> {
    let mut operands = Vec::new();
    let mut condition = String::from("AL"); // 默认无条件
    let mut writeback = false;
    let mut post_index = false;

    if let Ok(detail) = cs.insn_detail(insn) {
        let arch_detail = detail.arch_detail();
        if let Some(arm_detail) = arch_detail.arm() {
            condition = format!("{:?}", arm_detail.cc());
            writeback = arm_detail.writeback();
            post_index = insn.op_str().unwrap_or("").contains("],");

            for op in arm_detail.operands() {
                let op_type = match op.op_type {
                    ArmOperandType::Reg(reg_id) => {
                        OperandType::Register(cs.reg_name(reg_id).unwrap_or("unknown".to_string()))
                    }
                    ArmOperandType::Imm(imm) => OperandType::Immediate(imm as i64),
                    ArmOperandType::Mem(mem) => OperandType::Memory {
                        base: cs.reg_name(mem.base()).unwrap_or("none".to_string()),
                        index: cs.reg_name(mem.index()).unwrap_or("none".to_string()),
                        scale: mem.scale(),
                        disp: mem.disp() as i64,
                    },
                    _ => OperandType::Other,
                };

                let shift = match op.shift {
                    ArmShift::Invalid => ShiftInfo::None,
                    ArmShift::Lsl(val) => ShiftInfo::Lsl(val),
                    ArmShift::Lsr(val) => ShiftInfo::Lsr(val),
                    ArmShift::Asr(val) => ShiftInfo::Asr(val),
                    ArmShift::Ror(val) => ShiftInfo::Ror(val),
                    ArmShift::Rrx(val) => ShiftInfo::Rrx(val),
                    ArmShift::LslReg(reg_id) => {
                        ShiftInfo::LslReg(cs.reg_name(reg_id).unwrap_or("unknown".to_string()))
                    }
                    ArmShift::LsrReg(reg_id) => {
                        ShiftInfo::LsrReg(cs.reg_name(reg_id).unwrap_or("unknown".to_string()))
                    }
                    ArmShift::AsrReg(reg_id) => {
                        ShiftInfo::AsrReg(cs.reg_name(reg_id).unwrap_or("unknown".to_string()))
                    }
                    ArmShift::RorReg(reg_id) => {
                        ShiftInfo::RorReg(cs.reg_name(reg_id).unwrap_or("unknown".to_string()))
                    }
                    ArmShift::RrxReg(reg_id) => {
                        ShiftInfo::RrxReg(cs.reg_name(reg_id).unwrap_or("unknown".to_string()))
                    }
                };

                operands.push(OperandInfo { op_type, shift });
            }
        }
    }

    Ok(InstructionInfo {
        address: insn.address(),
        bytes: insn.bytes().to_vec(),
        mnemonic: insn.mnemonic().unwrap_or("").to_string(),
        op_str: insn.op_str().unwrap_or("").to_string(),
        operands,
        condition,
        writeback,
        post_index,
    })
}

/// 将指令详细信息写入文件
fn write_instruction_detail(
    writer: &mut BufWriter<File>,
    info: &InstructionInfo,
) -> Result<(), Box<dyn Error>> {
    writeln!(
        writer,
        "────────────────────────────────────────────────────────────────────────────"
    )?;
    writeln!(writer, "│ Address:    0x{:08X}", info.address)?;
    writeln!(writer, "│ Bytes:      {}", format_hex(&info.bytes))?;
    writeln!(writer, "│ Mnemonic:   {}", info.mnemonic)?;
    writeln!(writer, "│ Op String:  {}", info.op_str)?;
    writeln!(writer, "│ Condition:  {}", info.condition)?;
    writeln!(
        writer,
        "│ Writeback:  {}",
        if info.writeback { "Yes" } else { "No" }
    )?;
    writeln!(
        writer,
        "│ Post-Index: {}",
        if info.post_index { "Yes" } else { "No" }
    )?;

    if !info.operands.is_empty() {
        writeln!(writer, "│")?;
        writeln!(writer, "│ Operands ({}):", info.operands.len())?;
        for (idx, op) in info.operands.iter().enumerate() {
            let type_str = match &op.op_type {
                OperandType::Register(name) => format!("Register: {}", name),
                OperandType::Immediate(val) => format!("Immediate: 0x{:X} ({})", val, val),
                OperandType::Memory {
                    base,
                    index,
                    scale,
                    disp,
                } => {
                    format!(
                        "Memory [base={}, index={}, scale={}, disp={}]",
                        base, index, scale, disp
                    )
                }
                OperandType::Other => "Other".to_string(),
            };

            let shift_str = match &op.shift {
                ShiftInfo::None => "None".to_string(),
                ShiftInfo::Lsl(v) => format!("LSL #{}", v),
                ShiftInfo::Lsr(v) => format!("LSR #{}", v),
                ShiftInfo::Asr(v) => format!("ASR #{}", v),
                ShiftInfo::Ror(v) => format!("ROR #{}", v),
                ShiftInfo::Rrx(v) => format!("RRX #{}", v),
                ShiftInfo::LslReg(r) => format!("LSL {}", r),
                ShiftInfo::LsrReg(r) => format!("LSR {}", r),
                ShiftInfo::AsrReg(r) => format!("ASR {}", r),
                ShiftInfo::RorReg(r) => format!("ROR {}", r),
                ShiftInfo::RrxReg(r) => format!("RRX {}", r),
            };

            writeln!(writer, "│   [{}] Type:  {}", idx, type_str)?;
            writeln!(writer, "│       Shift: {}", shift_str)?;
        }
    }

    Ok(())
}

fn format_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}
