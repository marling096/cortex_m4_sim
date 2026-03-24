#![allow(dead_code)]

use capstone::arch::arm::{ArmOperandType, ArmShift};
use capstone::prelude::*;
use object::{Object, ObjectSection, ObjectSymbol, SectionKind};
use std::collections::BTreeMap;
use std::error::Error;

use super::writer::write_disassembly_to_file;
use super::{DataInfo, InstructionInfo, OperandInfo, OperandType, RegionKind, ShiftInfo};

#[derive(Debug, Clone)]
pub struct DisassemblyResult {
    pub start_address: u64,
    pub instruction_count: usize,
    pub data_word_count: usize,
    pub output_file: String,
}

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

    let cs = Capstone::new()
        .arm()
        .mode(arch::arm::ArchMode::Thumb)
        .extra_mode([arch::arm::ArchExtraMode::MClass].iter().copied())
        .detail(true)
        .build()
        .expect("Failed to create Capstone object");

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
                if name.to_lowercase().contains("reset_handler") || name == "Reset_Handler" {
                    reset_handler_addr = Some(addr);
                }
            }
        }
    }

    let mut all_instructions: Vec<InstructionInfo> = Vec::new();
    let mut all_data: Vec<DataInfo> = Vec::new();
    let mut addr_to_func: BTreeMap<u64, String> = BTreeMap::new();

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

                if let Some(name) = func_names.get(&start) {
                    addr_to_func.insert(start, name.to_string());
                }

                match kind {
                    RegionKind::Thumb => {
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

    let start_addr = 0x0800_0000;

    let filtered_instructions: Vec<&InstructionInfo> = all_instructions
        .iter()
        .filter(|i| i.address >= start_addr)
        .collect();

    let filtered_code_segments: Vec<(u64, Vec<u8>)> = code_segments
        .into_iter()
        .filter_map(|(addr, bytes)| {
            let end_addr = addr + bytes.len() as u64;
            if end_addr <= start_addr {
                None
            } else if addr < start_addr {
                let offset = (start_addr - addr) as usize;
                Some((start_addr, bytes[offset..].to_vec()))
            } else {
                Some((addr, bytes))
            }
        })
        .collect();

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

pub fn parse_axf_file(
    input_path: &str,
) -> Result<(u64, Capstone, Vec<u8>, BTreeMap<u32, u32>), Box<dyn Error>> {
    let bin_data = std::fs::read(input_path)?;
    let obj_file = object::File::parse(&*bin_data)?;

    let cs = Capstone::new()
        .arm()
        .mode(arch::arm::ArchMode::Thumb)
        .extra_mode([arch::arm::ArchExtraMode::MClass].iter().copied())
        .detail(true)
        .build()
        .expect("Failed to create Capstone object");

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

fn parse_instruction(cs: &Capstone, insn: &capstone::Insn) -> Result<InstructionInfo, Box<dyn Error>> {
    let mut operands = Vec::new();
    let mut condition = String::from("AL");
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
