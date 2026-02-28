use capstone::arch::arm::{ArmOperandType, ArmShift};
use capstone::prelude::*;
use std::fs::File;
use std::io::Write;

mod parser;
mod writer;

pub use parser::{disassemble_from_reset_handler, parse_axf_file, DisassemblyResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RegionKind {
    Thumb,
    Data,
}

#[derive(Debug)]
pub(super) struct InstructionInfo {
    pub(super) address: u64,
    pub(super) bytes: Vec<u8>,
    pub(super) mnemonic: String,
    pub(super) op_str: String,
    pub(super) operands: Vec<OperandInfo>,
    pub(super) condition: String,
    pub(super) writeback: bool,
    pub(super) post_index: bool,
}

#[derive(Debug)]
pub(super) struct OperandInfo {
    pub(super) op_type: OperandType,
    pub(super) shift: ShiftInfo,
}

#[derive(Debug)]
pub(super) enum OperandType {
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

#[derive(Debug)]
pub(super) enum ShiftInfo {
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

#[derive(Debug, Clone)]
pub(super) struct DataInfo {
    pub(super) address: u64,
    pub(super) bytes: Vec<u8>,
    pub(super) value: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub(super) enum ParsedRegion {
    Code(Vec<InstructionInfo>),
    Data(Vec<DataInfo>),
}

#[allow(dead_code)]
pub fn hex_ass_test(cs: &Capstone, insns: capstone::Instructions<'_>, file_path: &str) {
    let mut file = File::create(file_path).expect("Unable to create output file");
    for i in insns.iter() {
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

        if let Ok(detail) = cs.insn_detail(&i) {
            let arch_detail = detail.arch_detail();
            let arm_detail = arch_detail.arm().unwrap();

            let ops = arm_detail.operands();
            writeln!(file, "  └─ Operands count: {}", ops.len()).unwrap();

            writeln!(
                file,
                "--- Operands Detail --- writeback:{}  Cond:{}",
                arm_detail.writeback(),
                format!("{:?}", arm_detail.cc()),
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

                let shift = op.shift;
                match shift {
                    ArmShift::Invalid => {}
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
