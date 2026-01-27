use capstone::InsnDetail;
use capstone::arch::arm::{ArmOperandType, ArmShift};
use capstone::prelude::*;
use ihex::Record;
use std::fs::{self, File};
use std::io::Write;

mod context;
mod cpu;
use crate::cpu::Cpu;
mod opcodes;
mod simulator;
use crate::opcodes::instruction::{Cpu_InstrTable, Cpu_Instruction, OpcodeTable};
use crate::opcodes::opcode::{ArmOpcode, Opcode};

fn main() -> CsResult<()> {
    // 1. 初始化 Capstone (包含 MClass 模式)
    let cs: Capstone = Capstone::new()
        .arm()
        .mode(arch::arm::ArchMode::Thumb)
        .extra_mode([arch::arm::ArchExtraMode::MClass].iter().copied())
        .detail(true) // 必须开启 detail
        .build()
        .expect("Failed to create Capstone object");

    let hex_contents =
        fs::read_to_string("project.hex").expect("Something went wrong reading the file");
    let mut code = Vec::new();
    for record in ihex::Reader::new(&hex_contents) {
        if let Ok(Record::Data { value, .. }) = record {
            code.extend(value);
        }
    }
    let insns: capstone::Instructions<'_> = cs.disasm_all(&code, 0x0800_0000)?;

    let opcode_table = OpcodeTable::new();
    let table = opcode_table.get_table();

    // hex_ass_test(&cs, insns, "output.txt");

    // let Cpu_InstrTable = {
    //     let mut instr_table = Cpu_InstrTable::new();
    //     for i in insns.iter() {
    //         let key = i.id().0 as u16;
    //         if let Some(instructions) = table.get(&key) {
    //             for instruction in instructions {
    //                 let arm_opcode = ArmOpcode::new(&cs, &i);
    //                 let cpu_instruction =
    //                     Cpu_Instruction::new(instruction.clone(), arm_opcode.unwrap());
    //                 instr_table.add_instruction(cpu_instruction);
    //             }
    //         }
    //     }
    //     instr_table
    // };

    // let cpu = Cpu::new(48_000_000, 1);
    // let mut simulator = simulator::Simulator::new(cpu);
    // simulator.sim_loop(Cpu_InstrTable);
    let mut ins_count = 0;
    for i in insns.iter() {
        ins_count += 1;
        let key = i.id().0 as u16;
        if let Some(instructions) = table.get(&key) {
            for instruction in instructions {
                // println!(
                //     "Matched Instruction: 0x{:08x}: len:{:<8} {} {}",
                //     i.address(),
                //     i.len(),
                //     i.mnemonic().unwrap_or(""),
                //     i.op_str().unwrap_or("")
                // );
                // println!("Instruction Details: {:?}", instruction.name);
            }
        } else {
            println!(
                "No match for Instruction: 0x{:08x}: {:<8} len: {} OP: {} id: {}",
                i.address(),
                i.len(),
                i.mnemonic().unwrap_or(""),
                i.op_str().unwrap_or(""),
                i.id().0,
            );
        }
    }
    println!("Total Instructions Processed: {}", ins_count);

    Ok(())
}

fn hex_ass_test(cs: &Capstone, insns: capstone::Instructions<'_>, file_path: &str) {
    let mut file = File::create(file_path).expect("Unable to create output file");
    for i in insns.iter() {
        // 打印基础信息

        writeln!(
            file,
            "0x{:08x}: {:<8} {}",
            i.address(),
            i.mnemonic().unwrap_or(""),
            i.op_str().unwrap_or("")
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
                "--- Operands Detail --- writeback:{}",
                arm_detail.writeback(),
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
