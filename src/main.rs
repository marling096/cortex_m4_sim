mod context;
mod cpu;
mod opcodes;
mod tcg;
use capstone::InsnDetail;
use capstone::arch::arm::{ArmOperandType, ArmShift};
use capstone::prelude::*;

fn main() -> CsResult<()> {
    // 1. 初始化 Capstone (包含 MClass 模式)
    let cs = Capstone::new()
        .arm()
        .mode(arch::arm::ArchMode::Thumb)
        .extra_mode([arch::arm::ArchExtraMode::MClass].iter().copied())
        .detail(true) // 必须开启 detail
        .build()
        .expect("Failed to create Capstone object");

    let code: &[u8] = &[
        0xD2, 0xF8, 0x04, 0x10, 0x52, 0xF8, 0x04, 0x1F, 0x52, 0xF8, 0x04, 0x1B, 0x51, 0xF8, 0x22,
        0x00, 0x08, 0x68, 0x73, 0xE8, 0x01, 0x12,
    ];
    let insns = cs.disasm_all(code, 0x0800_0000)?;

    for i in insns.iter() {
        // 打印基础信息
        println!(
            "0x{:08x}: {:<8} {}",
            i.address(),
            i.mnemonic().unwrap_or(""),
            i.op_str().unwrap_or("")
        );

        // 2. 获取详细信息 (Detail)
        if let Ok(detail) = cs.insn_detail(&i) {
            let arch_detail = detail.arch_detail();
            let arm_detail = arch_detail.arm().unwrap(); // 获取 ARM 特有的详细数据

            let ops = arm_detail.operands();
            println!("  └─ Operands count: {}", ops.len()); //.contains("],")

            println!(
                "--- Operands Detail --- writeback:{}",
                arm_detail.writeback(),
                // arm_detail.post_index(),
            );
            if i.op_str().unwrap_or("").contains("],") {
                println!("-----post_index: true");
            }
            for (index, op) in ops.enumerate() {
                match op.op_type {
                    ArmOperandType::Reg(reg_id) => {
                        println!(
                            "     [Op {}] Type: Register, Value: {}",
                            index,
                            cs.reg_name(reg_id).unwrap_or("unknown".to_string())
                        );
                    }
                    ArmOperandType::Imm(imm) => {
                        println!("     [Op {}] Type: Immediate, Value: 0x{:x}", index, imm);
                    }
                    ArmOperandType::Mem(mem) => {
                        println!(
                            "     [Op {}] Type: Memory, Base: {}, Index: {}, Scale: {}, Disp: {}",
                            index,
                            cs.reg_name(mem.base()).unwrap_or("none".to_string()),
                            cs.reg_name(mem.index()).unwrap_or("none".to_string()),
                            mem.scale(),
                            mem.disp()
                        );
                    }
                    _ => println!("     [Op {}] Type: Other", index),
                }

                // 检查是否有移位 (Shift) 信息 (常见于 ARM/Thumb-2)
                let shift = op.shift;
                match shift {
                    ArmShift::Invalid => {
                        // 无移位，不做处理
                    }
                    ArmShift::Lsl(val) => println!("        └─ Shift: LSL, Value: {}", val),
                    ArmShift::Lsr(val) => println!("        └─ Shift: LSR, Value: {}", val),
                    ArmShift::Asr(val) => println!("        └─ Shift: ASR, Value: {}", val),
                    ArmShift::Ror(val) => println!("        └─ Shift: ROR, Value: {}", val),
                    ArmShift::Rrx(val) => println!("        └─ Shift: RRX, Value: {}", val),

                    ArmShift::LslReg(reg_id) => println!(
                        "        └─ Shift: LSL (Reg), Value: {}",
                        cs.reg_name(reg_id).unwrap_or("unknown".to_string())
                    ),
                    ArmShift::LsrReg(reg_id) => println!(
                        "        └─ Shift: LSR (Reg), Value: {}",
                        cs.reg_name(reg_id).unwrap_or("unknown".to_string())
                    ),
                    ArmShift::AsrReg(reg_id) => println!(
                        "        └─ Shift: ASR (Reg), Value: {}",
                        cs.reg_name(reg_id).unwrap_or("unknown".to_string())
                    ),
                    ArmShift::RorReg(reg_id) => println!(
                        "        └─ Shift: ROR (Reg), Value: {}",
                        cs.reg_name(reg_id).unwrap_or("unknown".to_string())
                    ),
                    ArmShift::RrxReg(reg_id) => println!(
                        "        └─ Shift: RRX (Reg), Value: {}",
                        cs.reg_name(reg_id).unwrap_or("unknown".to_string())
                    ),
                }
            }
        }
        println!("---");
    }

    Ok(())
}
