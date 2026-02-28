use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};

use super::{DataInfo, InstructionInfo, OperandType, ShiftInfo};

pub(super) fn write_disassembly_to_file(
    output_path: &str,
    start_addr: u64,
    instructions: &[&InstructionInfo],
    data: &[DataInfo],
    addr_to_func: &BTreeMap<u64, String>,
) -> Result<(), Box<dyn Error>> {
    let mut writer = BufWriter::new(File::create(output_path)?);

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
