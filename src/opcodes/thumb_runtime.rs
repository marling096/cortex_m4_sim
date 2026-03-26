use crate::arch::ArmCC;

use crate::context::CpuContext;
use crate::cpu::Cpu;
use crate::opcodes::opcode::{check_condition, UpdateApsr_C, UpdateApsr_N, UpdateApsr_V, UpdateApsr_Z};

pub struct StepOutcome {
    pub cycles: u32,
    pub mnemonic: &'static str,
    pub op_str: String,
}

pub fn step(cpu: &mut Cpu, current_pc: u32) -> Result<StepOutcome, String> {
    if let Some(cycles) = cpu.begin_step() {
        return Ok(StepOutcome {
            cycles,
            mnemonic: "pipeline",
            op_str: String::new(),
        });
    }

    let hw1 = fetch16(cpu, current_pc);
    let outcome = if is_thumb32(hw1) {
        let hw2 = fetch16(cpu, current_pc.wrapping_add(2));
        decode_execute_32(cpu, current_pc, hw1, hw2)?
    } else {
        decode_execute_16(cpu, current_pc, hw1)?
    };

    let cycles = cpu.finish_step_cycles(outcome.execute_cycles, current_pc, outcome.pc_update);
    Ok(StepOutcome {
        cycles,
        mnemonic: outcome.mnemonic,
        op_str: outcome.op_str,
    })
}

struct ExecOutcome {
    mnemonic: &'static str,
    op_str: String,
    execute_cycles: u32,
    pc_update: u32,
}

#[derive(Clone, Copy)]
enum ShiftKind {
    Lsl,
    Lsr,
    Asr,
    Ror,
}

fn fetch16(cpu: &Cpu, addr: u32) -> u16 {
    let bytes = cpu.fetch_thumb_bytes(addr);
    u16::from_le_bytes([bytes[0], bytes[1]])
}

fn is_thumb32(hw1: u16) -> bool {
    matches!(hw1 >> 11, 0b11101 | 0b11110 | 0b11111)
}

fn decode_execute_16(cpu: &mut Cpu, current_pc: u32, hw: u16) -> Result<ExecOutcome, String> {
    if (hw & 0xF800) == 0xE000 {
        let imm11 = (hw & 0x07FF) as u32;
        let offset = sign_extend((imm11 << 1) as u32, 12);
        let target = current_pc.wrapping_add(4).wrapping_add_signed(offset);
        cpu.write_pc(target);
        return Ok(branch_outcome("b", format!("#0x{target:08X}"), 2));
    }

    if (hw & 0xF000) == 0xD000 && (hw & 0x0F00) != 0x0F00 {
        let cond = decode_condition((hw >> 8) & 0xF)?;
        let imm8 = (hw & 0xFF) as u32;
        let offset = sign_extend((imm8 << 1) as u32, 9);
        let target = current_pc.wrapping_add(4).wrapping_add_signed(offset);
        let suffix = cond_suffix(cond);
        if check_condition(cpu, cond) {
            cpu.write_pc(target);
            return Ok(branch_outcome(cond_branch_mnemonic(suffix), format!("#0x{target:08X}"), 2));
        }
        return Ok(normal_outcome(cond_branch_mnemonic(suffix), format!("#0x{target:08X}"), 1, 2));
    }

    if (hw & 0xF500) == 0xB100 {
        let nonzero = (hw & 0x0800) != 0;
        let rn = (hw & 0x7) as u32;
        let i = ((hw >> 9) & 0x1) as u32;
        let imm5 = ((hw >> 3) & 0x1F) as u32;
        let offset = ((i << 6) | (imm5 << 1)) as u32;
        let target = current_pc.wrapping_add(4).wrapping_add(offset);
        let value = cpu.read_reg(rn);
        let taken = if nonzero { value != 0 } else { value == 0 };
        if taken {
            cpu.write_pc(target);
        }
        return Ok(ExecOutcome {
            mnemonic: if nonzero { "cbnz" } else { "cbz" },
            op_str: format!("r{rn}, #0x{target:08X}"),
            execute_cycles: 1,
            pc_update: if taken { 0 } else { 2 },
        });
    }

    if (hw & 0xF800) == 0x4800 {
        let rt = ((hw >> 8) & 0x7) as u32;
        let imm = ((hw & 0xFF) as u32) << 2;
        let base = current_pc.wrapping_add(4) & !0x3;
        let addr = base.wrapping_add(imm);
        let value = cpu.read_mem(addr & !0x3);
        cpu.write_reg(rt, value);
        return Ok(load_outcome("ldr", format!("r{rt}, [pc, #0x{imm:X}]"), rt, 2));
    }

    if (hw & 0xF800) == 0x6800 {
        let imm5 = ((hw >> 6) & 0x1F) as u32;
        let rn = ((hw >> 3) & 0x7) as u32;
        let rt = (hw & 0x7) as u32;
        let addr = cpu.read_reg(rn).wrapping_add(imm5 << 2) & !0x3;
        let value = cpu.read_mem(addr);
        cpu.write_reg(rt, value);
        return Ok(load_outcome("ldr", format!("r{rt}, [r{rn}, #0x{:X}]", imm5 << 2), rt, 2));
    }

    if (hw & 0xF800) == 0x6000 {
        let imm5 = ((hw >> 6) & 0x1F) as u32;
        let rn = ((hw >> 3) & 0x7) as u32;
        let rt = (hw & 0x7) as u32;
        let addr = cpu.read_reg(rn).wrapping_add(imm5 << 2) & !0x3;
        let value = cpu.read_reg(rt);
        cpu.write_mem(addr, value);
        return Ok(normal_outcome("str", format!("r{rt}, [r{rn}, #0x{:X}]", imm5 << 2), 1, 2));
    }

    if (hw & 0xF800) == 0x7000 {
        let imm5 = ((hw >> 6) & 0x1F) as u32;
        let rn = ((hw >> 3) & 0x7) as u32;
        let rt = (hw & 0x7) as u32;
        let addr = cpu.read_reg(rn).wrapping_add(imm5);
        write_u8(cpu, addr, cpu.read_reg(rt));
        return Ok(normal_outcome("strb", format!("r{rt}, [r{rn}, #0x{imm5:X}]"), 1, 2));
    }

    if (hw & 0xF800) == 0x7800 {
        let imm5 = ((hw >> 6) & 0x1F) as u32;
        let rn = ((hw >> 3) & 0x7) as u32;
        let rt = (hw & 0x7) as u32;
        let addr = cpu.read_reg(rn).wrapping_add(imm5);
        let value = read_u8(cpu, addr);
        cpu.write_reg(rt, value);
        return Ok(load_outcome("ldrb", format!("r{rt}, [r{rn}, #0x{imm5:X}]"), rt, 2));
    }

    if (hw & 0xF800) == 0x8000 {
        let imm5 = ((hw >> 6) & 0x1F) as u32;
        let rn = ((hw >> 3) & 0x7) as u32;
        let rt = (hw & 0x7) as u32;
        let addr = cpu.read_reg(rn).wrapping_add(imm5 << 1);
        write_u16(cpu, addr, cpu.read_reg(rt));
        return Ok(normal_outcome("strh", format!("r{rt}, [r{rn}, #0x{:X}]", imm5 << 1), 1, 2));
    }

    if (hw & 0xF800) == 0x8800 {
        let imm5 = ((hw >> 6) & 0x1F) as u32;
        let rn = ((hw >> 3) & 0x7) as u32;
        let rt = (hw & 0x7) as u32;
        let addr = cpu.read_reg(rn).wrapping_add(imm5 << 1);
        let value = read_u16(cpu, addr);
        cpu.write_reg(rt, value);
        return Ok(load_outcome("ldrh", format!("r{rt}, [r{rn}, #0x{:X}]", imm5 << 1), rt, 2));
    }

    if (hw & 0xF200) == 0x5000 {
        let op = (hw >> 9) & 0x7;
        let rm = ((hw >> 6) & 0x7) as u32;
        let rn = ((hw >> 3) & 0x7) as u32;
        let rt = (hw & 0x7) as u32;
        let addr = cpu.read_reg(rn).wrapping_add(cpu.read_reg(rm));
        return match op {
            0 => {
                cpu.write_mem(addr & !0x3, cpu.read_reg(rt));
                Ok(normal_outcome("str", format!("r{rt}, [r{rn}, r{rm}]"), 1, 2))
            }
            1 => {
                write_u16(cpu, addr, cpu.read_reg(rt));
                Ok(normal_outcome("strh", format!("r{rt}, [r{rn}, r{rm}]"), 1, 2))
            }
            2 => {
                write_u8(cpu, addr, cpu.read_reg(rt));
                Ok(normal_outcome("strb", format!("r{rt}, [r{rn}, r{rm}]"), 1, 2))
            }
            3 => {
                let value = sign_extend(read_u8(cpu, addr), 8) as u32;
                cpu.write_reg(rt, value);
                Ok(load_outcome("ldrsb", format!("r{rt}, [r{rn}, r{rm}]"), rt, 2))
            }
            4 => {
                let value = cpu.read_mem(addr & !0x3);
                cpu.write_reg(rt, value);
                Ok(load_outcome("ldr", format!("r{rt}, [r{rn}, r{rm}]"), rt, 2))
            }
            5 => {
                let value = read_u16(cpu, addr);
                cpu.write_reg(rt, value);
                Ok(load_outcome("ldrh", format!("r{rt}, [r{rn}, r{rm}]"), rt, 2))
            }
            6 => {
                let value = read_u8(cpu, addr);
                cpu.write_reg(rt, value);
                Ok(load_outcome("ldrb", format!("r{rt}, [r{rn}, r{rm}]"), rt, 2))
            }
            7 => {
                let value = sign_extend(read_u16(cpu, addr), 16) as u32;
                cpu.write_reg(rt, value);
                Ok(load_outcome("ldrsh", format!("r{rt}, [r{rn}, r{rm}]"), rt, 2))
            }
            _ => unreachable!(),
        };
    }

    if (hw & 0xF000) == 0x9000 {
        let load = (hw & 0x0800) != 0;
        let rt = ((hw >> 8) & 0x7) as u32;
        let imm = ((hw & 0xFF) as u32) << 2;
        let addr = cpu.read_sp().wrapping_add(imm);
        if load {
            let value = cpu.read_mem(addr & !0x3);
            cpu.write_reg(rt, value);
            return Ok(load_outcome("ldr", format!("r{rt}, [sp, #0x{imm:X}]"), rt, 2));
        }
        cpu.write_mem(addr & !0x3, cpu.read_reg(rt));
        return Ok(normal_outcome("str", format!("r{rt}, [sp, #0x{imm:X}]"), 1, 2));
    }

    if (hw & 0xFF00) == 0xB000 {
        let subtract = (hw & 0x0080) != 0;
        let imm = ((hw & 0x7F) as u32) << 2;
        let sp = cpu.read_sp();
        let result = if subtract {
            sp.wrapping_sub(imm)
        } else {
            sp.wrapping_add(imm)
        };
        cpu.write_sp(result);
        return Ok(normal_outcome(
            if subtract { "sub" } else { "add" },
            format!("sp, #0x{imm:X}"),
            1,
            2,
        ));
    }

    if (hw & 0xF800) == 0xA000 {
        let rd = ((hw >> 8) & 0x7) as u32;
        let imm = ((hw & 0xFF) as u32) << 2;
        let value = (current_pc.wrapping_add(4) & !0x3).wrapping_add(imm);
        cpu.write_reg(rd, value);
        return Ok(normal_outcome("adr", format!("r{rd}, #0x{value:08X}"), 1, 2));
    }

    if (hw & 0xF800) == 0xA800 {
        let rd = ((hw >> 8) & 0x7) as u32;
        let imm = ((hw & 0xFF) as u32) << 2;
        let value = cpu.read_sp().wrapping_add(imm);
        cpu.write_reg(rd, value);
        return Ok(normal_outcome("add", format!("r{rd}, sp, #0x{imm:X}"), 1, 2));
    }

    if (hw & 0xFE00) == 0xB400 {
        let extra_lr = ((hw >> 8) & 1) != 0;
        let mut mask = (hw & 0xFF) as u16;
        if extra_lr {
            mask |= 1 << 14;
        }
        push_reg_mask(cpu, mask);
        let count = mask.count_ones();
        return Ok(normal_outcome("push", reg_mask_to_string(mask), 1 + count, 2));
    }

    if (hw & 0xFE00) == 0xBC00 {
        let extra_pc = ((hw >> 8) & 1) != 0;
        let mut mask = (hw & 0xFF) as u16;
        if extra_pc {
            mask |= 1 << 15;
        }
        let wrote_pc = pop_reg_mask(cpu, mask);
        let count = mask.count_ones();
        return Ok(ExecOutcome {
            mnemonic: "pop",
            op_str: reg_mask_to_string(mask),
            execute_cycles: 1 + count + if wrote_pc { 1 } else { 0 },
            pc_update: if wrote_pc { 0 } else { 2 },
        });
    }

    if (hw & 0xF000) == 0xC000 {
        let load = (hw & 0x0800) != 0;
        let rn = ((hw >> 8) & 0x7) as u32;
        let mask = (hw & 0xFF) as u16;
        if load {
            ldm_mask(cpu, rn, mask, false);
            return Ok(normal_outcome("ldm", format!("r{rn}, {}", reg_mask_to_string(mask)), 1 + mask.count_ones(), 2));
        }
        stm_mask(cpu, rn, mask, false);
        return Ok(normal_outcome("stm", format!("r{rn}, {}", reg_mask_to_string(mask)), 1 + mask.count_ones(), 2));
    }

    if (hw & 0xFC00) == 0x4400 {
        let op = (hw >> 8) & 0x3;
        let h1 = ((hw >> 7) & 0x1) as u32;
        let h2 = ((hw >> 6) & 0x1) as u32;
        let rm = (((h2 << 3) | ((hw >> 3) & 0x7) as u32) & 0xF) as u32;
        let rdn = (((h1 << 3) | (hw & 0x7) as u32) & 0xF) as u32;
        return match op {
            0 => {
                let result = cpu.read_reg(rdn).wrapping_add(cpu.read_reg(rm));
                cpu.write_reg(rdn, result);
                Ok(normal_outcome("add", format!("r{rdn}, r{rm}"), 1, 2))
            }
            1 => {
                let lhs = cpu.read_reg(rdn);
                let rhs = cpu.read_reg(rm);
                update_sub_flags(cpu, lhs, rhs, lhs.wrapping_sub(rhs));
                Ok(normal_outcome("cmp", format!("r{rdn}, r{rm}"), 1, 2))
            }
            2 => {
                let value = cpu.read_reg(rm);
                cpu.write_reg(rdn, value);
                Ok(normal_outcome("mov", format!("r{rdn}, r{rm}"), 1, if rdn == 15 { 0 } else { 2 }))
            }
            3 => {
                let target = cpu.read_reg(rm);
                if (hw & 0x0080) != 0 {
                    let return_addr = current_pc.wrapping_add(2) | 1;
                    cpu.write_lr(return_addr);
                    cpu.write_pc(target & !1);
                    Ok(branch_outcome("blx", format!("r{rm}"), 2))
                } else {
                    cpu.write_pc(target & !1);
                    Ok(branch_outcome("bx", format!("r{rm}"), 2))
                }
            }
            _ => unreachable!(),
        };
    }

    if (hw & 0xFC00) == 0x4000 {
        let op = (hw >> 6) & 0xF;
        let rm = ((hw >> 3) & 0x7) as u32;
        let rdn = (hw & 0x7) as u32;
        return execute_data_proc_16(cpu, op, rdn, rm);
    }

    if (hw & 0xF800) == 0x0000 || (hw & 0xF800) == 0x0800 || (hw & 0xF800) == 0x1000 {
        let opcode = (hw >> 11) & 0x3;
        let imm5 = ((hw >> 6) & 0x1F) as u32;
        let rm = ((hw >> 3) & 0x7) as u32;
        let rd = (hw & 0x7) as u32;
        let value = cpu.read_reg(rm);
        return match opcode {
            0 => execute_shift_imm(cpu, "lsls", ShiftKind::Lsl, rd, value, imm5, true),
            1 => execute_shift_imm(cpu, "lsrs", ShiftKind::Lsr, rd, value, if imm5 == 0 { 32 } else { imm5 }, true),
            2 => execute_shift_imm(cpu, "asrs", ShiftKind::Asr, rd, value, if imm5 == 0 { 32 } else { imm5 }, true),
            _ => Err(format!("unsupported 16-bit shift opcode at 0x{current_pc:08X}")),
        };
    }

    if (hw & 0xF800) == 0x1800 {
        let op = (hw >> 9) & 0x1;
        let imm_flag = (hw >> 10) & 0x1;
        let rn = ((hw >> 3) & 0x7) as u32;
        let rd = (hw & 0x7) as u32;
        if imm_flag == 0 {
            let rm = ((hw >> 6) & 0x7) as u32;
            return if op == 0 {
                execute_add(cpu, rd, rn, cpu.read_reg(rm), true, format!("r{rd}, r{rn}, r{rm}"), 2)
            } else {
                execute_sub(cpu, rd, rn, cpu.read_reg(rm), true, format!("r{rd}, r{rn}, r{rm}"), 2)
            };
        }
        let imm3 = ((hw >> 6) & 0x7) as u32;
        return if op == 0 {
            execute_add(cpu, rd, rn, imm3, true, format!("r{rd}, r{rn}, #{imm3}"), 2)
        } else {
            execute_sub(cpu, rd, rn, imm3, true, format!("r{rd}, r{rn}, #{imm3}"), 2)
        };
    }

    if (hw & 0xE000) == 0x2000 {
        let op = (hw >> 11) & 0x3;
        let rd = ((hw >> 8) & 0x7) as u32;
        let imm8 = (hw & 0xFF) as u32;
        return match op {
            0 => {
                cpu.write_reg(rd, imm8);
                UpdateApsr_N(cpu, imm8);
                UpdateApsr_Z(cpu, imm8);
                Ok(normal_outcome("movs", format!("r{rd}, #{imm8}"), 1, 2))
            }
            1 => {
                let lhs = cpu.read_reg(rd);
                update_sub_flags(cpu, lhs, imm8, lhs.wrapping_sub(imm8));
                Ok(normal_outcome("cmp", format!("r{rd}, #{imm8}"), 1, 2))
            }
            2 => execute_add(cpu, rd, rd, imm8, true, format!("r{rd}, #{imm8}"), 2),
            3 => execute_sub(cpu, rd, rd, imm8, true, format!("r{rd}, #{imm8}"), 2),
            _ => unreachable!(),
        };
    }

    if (hw & 0xFF00) == 0xBE00 {
        let imm = (hw & 0xFF) as u32;
        println!("BKPT #{}", imm);
        return Ok(branch_outcome("bkpt", format!("#{imm}"), 1));
    }

    if hw == 0xBF00 {
        return Ok(normal_outcome("nop", String::new(), 1, 2));
    }

    Err(format!("unsupported 16-bit instruction 0x{hw:04X} at PC 0x{current_pc:08X}"))
}

fn decode_execute_32(cpu: &mut Cpu, current_pc: u32, hw1: u16, hw2: u16) -> Result<ExecOutcome, String> {
    if (hw1 & 0xF800) == 0xF000 && (hw2 & 0xD000) == 0xD000 {
        let s = ((hw1 >> 10) & 1) as u32;
        let j1 = ((hw2 >> 13) & 1) as u32;
        let j2 = ((hw2 >> 11) & 1) as u32;
        let i1 = (!(j1 ^ s)) & 1;
        let i2 = (!(j2 ^ s)) & 1;
        let imm10 = (hw1 & 0x03FF) as u32;
        let imm11 = (hw2 & 0x07FF) as u32;
        let imm25 = (s << 24) | (i1 << 23) | (i2 << 22) | (imm10 << 12) | (imm11 << 1);
        let offset = sign_extend(imm25, 25);
        let target = current_pc.wrapping_add(4).wrapping_add_signed(offset);
        cpu.write_lr((current_pc.wrapping_add(4)) | 1);
        cpu.write_pc(target);
        return Ok(branch_outcome("bl", format!("#0x{target:08X}"), 2));
    }

    if hw1 == 0xF8DF {
        let rt = ((hw2 >> 12) & 0xF) as u32;
        let imm12 = (hw2 & 0x0FFF) as u32;
        let base = current_pc.wrapping_add(4) & !0x3;
        let addr = base.wrapping_add(imm12);
        let value = cpu.read_mem(addr & !0x3);
        cpu.write_reg(rt, value);
        return Ok(load_outcome("ldr.w", format!("r{rt}, [pc, #0x{imm12:X}]"), rt, 4));
    }

    if (hw1 & 0xFFF0) == 0xF890 {
        let rn = (hw1 & 0xF) as u32;
        let rt = ((hw2 >> 12) & 0xF) as u32;
        let imm12 = (hw2 & 0x0FFF) as u32;
        let value = read_u8(cpu, cpu.read_reg(rn).wrapping_add(imm12));
        cpu.write_reg(rt, value);
        return Ok(load_outcome("ldrb.w", format!("r{rt}, [r{rn}, #0x{imm12:X}]"), rt, 4));
    }

    if (hw1 & 0xFFF0) == 0xF8B0 {
        let rn = (hw1 & 0xF) as u32;
        let rt = ((hw2 >> 12) & 0xF) as u32;
        let imm12 = (hw2 & 0x0FFF) as u32;
        let value = read_u16(cpu, cpu.read_reg(rn).wrapping_add(imm12));
        cpu.write_reg(rt, value);
        return Ok(load_outcome("ldrh.w", format!("r{rt}, [r{rn}, #0x{imm12:X}]"), rt, 4));
    }

    if (hw1 & 0xFFF0) == 0xF8D0 {
        let rn = (hw1 & 0xF) as u32;
        let rt = ((hw2 >> 12) & 0xF) as u32;
        let imm12 = (hw2 & 0x0FFF) as u32;
        let addr = cpu.read_reg(rn).wrapping_add(imm12) & !0x3;
        let value = cpu.read_mem(addr);
        cpu.write_reg(rt, value);
        return Ok(load_outcome("ldr.w", format!("r{rt}, [r{rn}, #0x{imm12:X}]"), rt, 4));
    }

    if (hw1 & 0xFFF0) == 0xF8C0 {
        let rn = (hw1 & 0xF) as u32;
        let rt = ((hw2 >> 12) & 0xF) as u32;
        let imm12 = (hw2 & 0x0FFF) as u32;
        let addr = cpu.read_reg(rn).wrapping_add(imm12) & !0x3;
        cpu.write_mem(addr, cpu.read_reg(rt));
        return Ok(normal_outcome("str.w", format!("r{rt}, [r{rn}, #0x{imm12:X}]"), 1, 4));
    }

    if (hw1 & 0xFFF0) == 0xF880 {
        let rn = (hw1 & 0xF) as u32;
        let rt = ((hw2 >> 12) & 0xF) as u32;
        let imm12 = (hw2 & 0x0FFF) as u32;
        let addr = cpu.read_reg(rn).wrapping_add(imm12);
        write_u8(cpu, addr, cpu.read_reg(rt));
        return Ok(normal_outcome("strb.w", format!("r{rt}, [r{rn}, #0x{imm12:X}]"), 1, 4));
    }

    if (hw1 & 0xFFF0) == 0xF8A0 {
        let rn = (hw1 & 0xF) as u32;
        let rt = ((hw2 >> 12) & 0xF) as u32;
        let imm12 = (hw2 & 0x0FFF) as u32;
        let addr = cpu.read_reg(rn).wrapping_add(imm12);
        write_u16(cpu, addr, cpu.read_reg(rt));
        return Ok(normal_outcome("strh.w", format!("r{rt}, [r{rn}, #0x{imm12:X}]"), 1, 4));
    }

    if (hw1 & 0xFBF0) == 0xF240 && (hw2 & 0x8000) == 0 {
        let rd = ((hw2 >> 8) & 0xF) as u32;
        let imm4 = (hw1 & 0xF) as u32;
        let i = ((hw1 >> 10) & 1) as u32;
        let imm3 = ((hw2 >> 12) & 0x7) as u32;
        let imm8 = (hw2 & 0xFF) as u32;
        let imm16 = (imm4 << 12) | (i << 11) | (imm3 << 8) | imm8;
        cpu.write_reg(rd, imm16);
        return Ok(normal_outcome("movw", format!("r{rd}, #0x{imm16:X}"), 1, if rd == 15 { 0 } else { 4 }));
    }

    if (hw1 & 0xFBF0) == 0xF2C0 && (hw2 & 0x8000) == 0 {
        let rd = ((hw2 >> 8) & 0xF) as u32;
        let imm4 = (hw1 & 0xF) as u32;
        let i = ((hw1 >> 10) & 1) as u32;
        let imm3 = ((hw2 >> 12) & 0x7) as u32;
        let imm8 = (hw2 & 0xFF) as u32;
        let imm16 = (imm4 << 12) | (i << 11) | (imm3 << 8) | imm8;
        let value = (cpu.read_reg(rd) & 0x0000_FFFF) | (imm16 << 16);
        cpu.write_reg(rd, value);
        return Ok(normal_outcome("movt", format!("r{rd}, #0x{imm16:X}"), 1, if rd == 15 { 0 } else { 4 }));
    }

    if (hw1 & 0xFFF0) == 0xFBB0 && (hw2 & 0x00F0) == 0x00F0 {
        let rn = (hw1 & 0xF) as u32;
        let rd = ((hw2 >> 8) & 0xF) as u32;
        let rm = (hw2 & 0xF) as u32;
        let numerator = cpu.read_reg(rn);
        let denominator = cpu.read_reg(rm);
        let result = if denominator == 0 { 0 } else { numerator / denominator };
        cpu.write_reg(rd, result);
        return Ok(normal_outcome("udiv", format!("r{rd}, r{rn}, r{rm}"), 1, if rd == 15 { 0 } else { 4 }));
    }

    if (hw1 & 0xFFF0) == 0xFB00 && (hw2 & 0x00F0) == 0x0010 {
        let rn = (hw1 & 0xF) as u32;
        let ra = ((hw2 >> 12) & 0xF) as u32;
        let rd = ((hw2 >> 8) & 0xF) as u32;
        let rm = (hw2 & 0xF) as u32;
        let result = cpu.read_reg(ra).wrapping_sub(cpu.read_reg(rn).wrapping_mul(cpu.read_reg(rm)));
        cpu.write_reg(rd, result);
        return Ok(normal_outcome("mls", format!("r{rd}, r{rn}, r{rm}, r{ra}"), 1, if rd == 15 { 0 } else { 4 }));
    }

    if (hw1 & 0xFFF0) == 0xF3C0 {
        let rn = (hw1 & 0xF) as u32;
        let rd = ((hw2 >> 8) & 0xF) as u32;
        let lsb = ((((hw2 >> 12) & 0x7) as u32) << 2) | (((hw2 >> 6) & 0x3) as u32);
        let width = ((hw2 & 0x1F) as u32) + 1;
        let source = cpu.read_reg(rn);
        let effective_width = width.min(32u32.saturating_sub(lsb));
        let result = if effective_width == 0 {
            0
        } else if effective_width >= 32 {
            source
        } else {
            (source >> lsb) & ((1u32 << effective_width) - 1)
        };
        cpu.write_reg(rd, result);
        return Ok(normal_outcome("ubfx", format!("r{rd}, r{rn}, #{lsb}, #{width}"), 1, if rd == 15 { 0 } else { 4 }));
    }

    if (hw1 & 0xFA00) == 0xF000 && (hw2 & 0x8000) == 0 {
        let op = (hw1 >> 5) & 0xF;
        let rn = (hw1 & 0xF) as u32;
        let rd = ((hw2 >> 8) & 0xF) as u32;
        let setflags = ((hw1 >> 4) & 0x1) != 0;
        let imm12 = (((hw1 >> 10) & 1) as u16) << 11 | ((hw2 >> 12) & 0x7) << 8 | (hw2 & 0xFF);
        let imm = thumb_expand_imm12(imm12) as u32;
        return match op {
            0 => {
                let result = cpu.read_reg(rn) & imm;
                cpu.write_reg(rd, result);
                if setflags {
                    UpdateApsr_N(cpu, result);
                    UpdateApsr_Z(cpu, result);
                }
                Ok(normal_outcome("and", format!("r{rd}, r{rn}, #0x{imm:X}"), 1, 4))
            }
            1 => {
                let result = cpu.read_reg(rn) & !imm;
                cpu.write_reg(rd, result);
                if setflags {
                    UpdateApsr_N(cpu, result);
                    UpdateApsr_Z(cpu, result);
                }
                Ok(normal_outcome("bic", format!("r{rd}, r{rn}, #0x{imm:X}"), 1, if rd == 15 { 0 } else { 4 }))
            }
            2 if rn == 15 => {
                cpu.write_reg(rd, imm);
                if setflags {
                    UpdateApsr_N(cpu, imm);
                    UpdateApsr_Z(cpu, imm);
                }
                Ok(normal_outcome("mov.w", format!("r{rd}, #0x{imm:X}"), 1, if rd == 15 { 0 } else { 4 }))
            }
            2 => {
                let result = cpu.read_reg(rn) | imm;
                cpu.write_reg(rd, result);
                if setflags {
                    UpdateApsr_N(cpu, result);
                    UpdateApsr_Z(cpu, result);
                }
                Ok(normal_outcome("orr", format!("r{rd}, r{rn}, #0x{imm:X}"), 1, if rd == 15 { 0 } else { 4 }))
            }
            8 => execute_add(cpu, rd, rn, imm, setflags, format!("r{rd}, r{rn}, #0x{imm:X}"), 4),
            13 => {
                let lhs = cpu.read_reg(rn);
                update_sub_flags(cpu, lhs, imm, lhs.wrapping_sub(imm));
                Ok(normal_outcome("cmp.w", format!("r{rn}, #0x{imm:X}"), 1, 4))
            }
            _ => Err(format!("unsupported modified-immediate opcode op={} at 0x{current_pc:08X}", op)),
        };
    }

    if (hw1 & 0xFE00) == 0xEA00 || (hw1 & 0xFE00) == 0xEB00 {
        let op = (hw1 >> 5) & 0xF;
        let rn = (hw1 & 0xF) as u32;
        let rd = ((hw2 >> 8) & 0xF) as u32;
        let rm = (hw2 & 0xF) as u32;
        let imm3 = ((hw2 >> 12) & 0x7) as u32;
        let imm2 = ((hw2 >> 6) & 0x3) as u32;
        let shift_type = ((hw2 >> 4) & 0x3) as u32;
        let shift_amount = (imm3 << 2) | imm2;
        let rhs = apply_shift(cpu.read_reg(rm), shift_type_to_kind(shift_type)?, shift_amount, ((cpu.read_apsr() >> 29) & 1) as u8).0;
        return match op {
            0 => {
                let result = cpu.read_reg(rn) & rhs;
                cpu.write_reg(rd, result);
                Ok(normal_outcome("and.w", format!("r{rd}, r{rn}, r{rm}"), 1, 4))
            }
            2 => {
                let result = cpu.read_reg(rn) | rhs;
                cpu.write_reg(rd, result);
                Ok(normal_outcome("orr.w", format!("r{rd}, r{rn}, r{rm}"), 1, 4))
            }
            8 => execute_add(cpu, rd, rn, rhs, false, format!("r{rd}, r{rn}, r{rm}"), 4),
            _ => Err(format!("unsupported shifted-register opcode op={} at 0x{current_pc:08X}", op)),
        };
    }

    if (hw1 & 0xFFF0) == 0xFA00 {
        let rm = (hw1 & 0xF) as u32;
        let rd = ((hw2 >> 8) & 0xF) as u32;
        let rs = (hw2 & 0xF) as u32;
        let shift = cpu.read_reg(rs) & 0xFF;
        return execute_shift_reg(cpu, "lsl.w", ShiftKind::Lsl, rd, rm, shift, false, 4);
    }

    if hw1 == 0xE92D {
        let mask = multi_reg_mask_push(hw2);
        push_reg_mask(cpu, mask);
        return Ok(normal_outcome("push.w", reg_mask_to_string(mask), 1 + mask.count_ones(), 4));
    }

    if hw1 == 0xE8BD {
        let mask = multi_reg_mask_pop(hw2);
        let wrote_pc = pop_reg_mask(cpu, mask);
        let count = mask.count_ones();
        return Ok(ExecOutcome {
            mnemonic: "pop.w",
            op_str: reg_mask_to_string(mask),
            execute_cycles: 1 + count + if wrote_pc { 1 } else { 0 },
            pc_update: if wrote_pc { 0 } else { 4 },
        });
    }

    if (hw1 & 0xFFD0) == 0xE890 {
        let rn = (hw1 & 0xF) as u32;
        let writeback = ((hw1 >> 5) & 1) != 0;
        let mask = hw2;
        let wrote_pc = ldm_mask(cpu, rn, mask, writeback);
        return Ok(ExecOutcome {
            mnemonic: "ldm.w",
            op_str: format!("r{rn}, {}", reg_mask_to_string(mask)),
            execute_cycles: 1 + mask.count_ones() + if wrote_pc { 1 } else { 0 },
            pc_update: if wrote_pc { 0 } else { 4 },
        });
    }

    Err(format!("unsupported 32-bit instruction 0x{hw1:04X} 0x{hw2:04X} at PC 0x{current_pc:08X}"))
}

fn execute_data_proc_16(cpu: &mut Cpu, op: u16, rdn: u32, rm: u32) -> Result<ExecOutcome, String> {
    let lhs = cpu.read_reg(rdn);
    let rhs = cpu.read_reg(rm);
    let carry = ((cpu.read_apsr() >> 29) & 1) as u8;
    match op {
        0 => {
            let result = lhs & rhs;
            cpu.write_reg(rdn, result);
            UpdateApsr_N(cpu, result);
            UpdateApsr_Z(cpu, result);
            Ok(normal_outcome("ands", format!("r{rdn}, r{rm}"), 1, 2))
        }
        2 => execute_shift_reg(cpu, "lsls", ShiftKind::Lsl, rdn, rdn, rhs & 0xFF, true, 2),
        3 => execute_shift_reg(cpu, "lsrs", ShiftKind::Lsr, rdn, rdn, rhs & 0xFF, true, 2),
        4 => execute_shift_reg(cpu, "asrs", ShiftKind::Asr, rdn, rdn, rhs & 0xFF, true, 2),
        5 => {
            let result64 = lhs as u64 + rhs as u64 + carry as u64;
            let result = result64 as u32;
            cpu.write_reg(rdn, result);
            UpdateApsr_N(cpu, result);
            UpdateApsr_Z(cpu, result);
            UpdateApsr_C(cpu, if result64 > 0xFFFF_FFFF { 1 } else { 0 });
            Ok(normal_outcome("adcs", format!("r{rdn}, r{rm}"), 1, 2))
        }
        6 => {
            let borrow = 1u32.wrapping_sub(carry as u32);
            let result = lhs.wrapping_sub(rhs).wrapping_sub(borrow);
            cpu.write_reg(rdn, result);
            UpdateApsr_N(cpu, result);
            UpdateApsr_Z(cpu, result);
            UpdateApsr_C(cpu, if (lhs as u64) >= (rhs as u64 + borrow as u64) { 1 } else { 0 });
            Ok(normal_outcome("sbcs", format!("r{rdn}, r{rm}"), 1, 2))
        }
        7 => execute_shift_reg(cpu, "rors", ShiftKind::Ror, rdn, rdn, rhs & 0xFF, true, 2),
        8 => {
            let result = lhs & rhs;
            UpdateApsr_N(cpu, result);
            UpdateApsr_Z(cpu, result);
            Ok(normal_outcome("tst", format!("r{rdn}, r{rm}"), 1, 2))
        }
        9 => {
            let result = 0u32.wrapping_sub(rhs);
            cpu.write_reg(rdn, result);
            UpdateApsr_N(cpu, result);
            UpdateApsr_Z(cpu, result);
            UpdateApsr_C(cpu, if rhs == 0 { 1 } else { 0 });
            UpdateApsr_V(cpu, if rhs == 0x8000_0000 { 1 } else { 0 });
            Ok(normal_outcome("rsbs", format!("r{rdn}, r{rm}"), 1, 2))
        }
        10 => {
            update_sub_flags(cpu, lhs, rhs, lhs.wrapping_sub(rhs));
            Ok(normal_outcome("cmp", format!("r{rdn}, r{rm}"), 1, 2))
        }
        11 => {
            let result = lhs.wrapping_add(rhs);
            UpdateApsr_N(cpu, result);
            UpdateApsr_Z(cpu, result);
            UpdateApsr_C(cpu, if (lhs as u64) + (rhs as u64) > 0xFFFF_FFFF { 1 } else { 0 });
            Ok(normal_outcome("cmn", format!("r{rdn}, r{rm}"), 1, 2))
        }
        12 => {
            let result = lhs | rhs;
            cpu.write_reg(rdn, result);
            UpdateApsr_N(cpu, result);
            UpdateApsr_Z(cpu, result);
            Ok(normal_outcome("orrs", format!("r{rdn}, r{rm}"), 1, 2))
        }
        13 => {
            let result = lhs.wrapping_mul(rhs);
            cpu.write_reg(rdn, result);
            UpdateApsr_N(cpu, result);
            UpdateApsr_Z(cpu, result);
            Ok(normal_outcome("muls", format!("r{rdn}, r{rm}"), 1, 2))
        }
        14 => {
            let result = lhs & !rhs;
            cpu.write_reg(rdn, result);
            UpdateApsr_N(cpu, result);
            UpdateApsr_Z(cpu, result);
            Ok(normal_outcome("bics", format!("r{rdn}, r{rm}"), 1, 2))
        }
        15 => {
            let result = !rhs;
            cpu.write_reg(rdn, result);
            UpdateApsr_N(cpu, result);
            UpdateApsr_Z(cpu, result);
            Ok(normal_outcome("mvns", format!("r{rdn}, r{rm}"), 1, 2))
        }
        _ => Err(format!("unsupported 16-bit data-processing op={op}")),
    }
}

fn execute_add(
    cpu: &mut Cpu,
    rd: u32,
    rn: u32,
    rhs: u32,
    setflags: bool,
    op_str: String,
    size: u32,
) -> Result<ExecOutcome, String> {
    let lhs = cpu.read_reg(rn);
    let result = lhs.wrapping_add(rhs);
    cpu.write_reg(rd, result);
    if setflags {
        UpdateApsr_N(cpu, result);
        UpdateApsr_Z(cpu, result);
        UpdateApsr_C(cpu, if (lhs as u64) + (rhs as u64) > 0xFFFF_FFFF { 1 } else { 0 });
        let overflow = (((lhs ^ result) & (rhs ^ result)) >> 31) & 1;
        UpdateApsr_V(cpu, overflow as u8);
    }
    Ok(normal_outcome(if setflags { "adds" } else { "add.w" }, op_str, 1, if rd == 15 { 0 } else { size }))
}

fn execute_sub(
    cpu: &mut Cpu,
    rd: u32,
    rn: u32,
    rhs: u32,
    setflags: bool,
    op_str: String,
    size: u32,
) -> Result<ExecOutcome, String> {
    let lhs = cpu.read_reg(rn);
    let result = lhs.wrapping_sub(rhs);
    cpu.write_reg(rd, result);
    if setflags {
        update_sub_flags(cpu, lhs, rhs, result);
    }
    Ok(normal_outcome(if setflags { "subs" } else { "sub.w" }, op_str, 1, if rd == 15 { 0 } else { size }))
}

fn update_sub_flags(cpu: &mut Cpu, lhs: u32, rhs: u32, result: u32) {
    UpdateApsr_N(cpu, result);
    UpdateApsr_Z(cpu, result);
    UpdateApsr_C(cpu, if lhs >= rhs { 1 } else { 0 });
    let overflow = (((lhs ^ rhs) & (lhs ^ result)) >> 31) & 1;
    UpdateApsr_V(cpu, overflow as u8);
}

fn execute_shift_imm(
    cpu: &mut Cpu,
    mnemonic: &'static str,
    kind: ShiftKind,
    rd: u32,
    value: u32,
    amount: u32,
    setflags: bool,
) -> Result<ExecOutcome, String> {
    let (result, carry) = apply_shift(value, kind, amount, ((cpu.read_apsr() >> 29) & 1) as u8);
    cpu.write_reg(rd, result);
    if setflags {
        UpdateApsr_N(cpu, result);
        UpdateApsr_Z(cpu, result);
        if amount != 0 {
            UpdateApsr_C(cpu, carry);
        }
    }
    Ok(normal_outcome(mnemonic, format!("r{rd}, #{}", amount), 1, if rd == 15 { 0 } else { 2 }))
}

fn execute_shift_reg(
    cpu: &mut Cpu,
    mnemonic: &'static str,
    kind: ShiftKind,
    rd: u32,
    rm: u32,
    amount: u32,
    setflags: bool,
    size: u32,
) -> Result<ExecOutcome, String> {
    let (result, carry) = apply_shift(cpu.read_reg(rm), kind, amount, ((cpu.read_apsr() >> 29) & 1) as u8);
    cpu.write_reg(rd, result);
    if setflags {
        UpdateApsr_N(cpu, result);
        UpdateApsr_Z(cpu, result);
        if amount != 0 {
            UpdateApsr_C(cpu, carry);
        }
    }
    Ok(normal_outcome(mnemonic, format!("r{rd}, r{rm}, r{}", amount), 1, if rd == 15 { 0 } else { size }))
}

fn apply_shift(value: u32, kind: ShiftKind, amount: u32, current_c: u8) -> (u32, u8) {
    match kind {
        ShiftKind::Lsl => match amount {
            0 => (value, current_c),
            1..=31 => (value.wrapping_shl(amount), ((value >> (32 - amount)) & 1) as u8),
            32 => (0, (value & 1) as u8),
            _ => (0, 0),
        },
        ShiftKind::Lsr => match amount {
            0 => (value, current_c),
            1..=31 => (value >> amount, ((value >> (amount - 1)) & 1) as u8),
            32 => (0, ((value >> 31) & 1) as u8),
            _ => (0, 0),
        },
        ShiftKind::Asr => match amount {
            0 => (value, current_c),
            1..=31 => (((value as i32) >> amount) as u32, ((value >> (amount - 1)) & 1) as u8),
            _ => {
                let carry = ((value >> 31) & 1) as u8;
                (if (value as i32) < 0 { u32::MAX } else { 0 }, carry)
            }
        },
        ShiftKind::Ror => {
            if amount == 0 {
                return (value, current_c);
            }
            let shift = amount & 31;
            if shift == 0 {
                (value, ((value >> 31) & 1) as u8)
            } else {
                let result = value.rotate_right(shift);
                (result, ((result >> 31) & 1) as u8)
            }
        }
    }
}

fn read_u8(cpu: &mut Cpu, addr: u32) -> u32 {
    let word = cpu.read_mem(addr & !3);
    let shift = (addr & 3) * 8;
    (word >> shift) & 0xFF
}

fn read_u16(cpu: &mut Cpu, addr: u32) -> u32 {
    let word = cpu.read_mem(addr & !3);
    let shift = (addr & 2) * 8;
    (word >> shift) & 0xFFFF
}

fn write_u8(cpu: &mut Cpu, addr: u32, val: u32) {
    let aligned = addr & !3;
    let word = cpu.read_mem(aligned);
    let shift = (addr & 3) * 8;
    let mask = !(0xFF << shift);
    cpu.write_mem(aligned, (word & mask) | ((val & 0xFF) << shift));
}

fn write_u16(cpu: &mut Cpu, addr: u32, val: u32) {
    let aligned = addr & !3;
    let word = cpu.read_mem(aligned);
    let shift = (addr & 2) * 8;
    let mask = !(0xFFFF << shift);
    cpu.write_mem(aligned, (word & mask) | ((val & 0xFFFF) << shift));
}

fn push_reg_mask(cpu: &mut Cpu, mask: u16) {
    let count = mask.count_ones();
    let mut addr = cpu.read_sp().wrapping_sub(4 * count);
    for reg in 0..16u32 {
        if (mask & (1u16 << reg)) != 0 {
            cpu.write_mem(addr, cpu.read_reg(reg));
            addr = addr.wrapping_add(4);
        }
    }
    cpu.write_sp(cpu.read_sp().wrapping_sub(4 * count));
}

fn pop_reg_mask(cpu: &mut Cpu, mask: u16) -> bool {
    let mut sp = cpu.read_sp();
    let mut wrote_pc = false;
    let mut pc_value = 0u32;
    for reg in 0..16u32 {
        if (mask & (1u16 << reg)) != 0 {
            let value = cpu.read_mem(sp);
            sp = sp.wrapping_add(4);
            if reg == 15 {
                wrote_pc = true;
                pc_value = value;
            } else {
                cpu.write_reg(reg, value);
            }
        }
    }
    cpu.write_sp(sp);
    if wrote_pc {
        if !cpu.try_exception_return(pc_value) {
            cpu.write_pc(pc_value & !1);
        }
    }
    wrote_pc
}

fn ldm_mask(cpu: &mut Cpu, rn: u32, mask: u16, writeback: bool) -> bool {
    let mut addr = cpu.read_reg(rn);
    let mut wrote_pc = false;
    for reg in 0..16u32 {
        if (mask & (1u16 << reg)) != 0 {
            let value = cpu.read_mem(addr);
            addr = addr.wrapping_add(4);
            if reg == 15 {
                cpu.write_pc(value & !1);
                wrote_pc = true;
            } else {
                cpu.write_reg(reg, value);
            }
        }
    }
    if writeback {
        cpu.write_reg(rn, addr);
    }
    wrote_pc
}

fn stm_mask(cpu: &mut Cpu, rn: u32, mask: u16, writeback: bool) {
    let mut addr = cpu.read_reg(rn);
    for reg in 0..16u32 {
        if (mask & (1u16 << reg)) != 0 {
            cpu.write_mem(addr, cpu.read_reg(reg));
            addr = addr.wrapping_add(4);
        }
    }
    if writeback {
        cpu.write_reg(rn, addr);
    }
}

fn multi_reg_mask_push(hw2: u16) -> u16 {
    let mut mask = hw2 & 0x1FFF;
    if (hw2 & 0x4000) != 0 {
        mask |= 1 << 14;
    }
    mask
}

fn multi_reg_mask_pop(hw2: u16) -> u16 {
    let mut mask = hw2 & 0x1FFF;
    if (hw2 & 0x8000) != 0 {
        mask |= 1 << 15;
    }
    mask
}

fn reg_mask_to_string(mask: u16) -> String {
    let mut regs = Vec::new();
    for reg in 0..16u32 {
        if (mask & (1u16 << reg)) != 0 {
            regs.push(reg_name(reg));
        }
    }
    format!("{{{}}}", regs.join(", "))
}

fn reg_name(reg: u32) -> String {
    match reg {
        13 => "sp".to_string(),
        14 => "lr".to_string(),
        15 => "pc".to_string(),
        12 => "ip".to_string(),
        _ => format!("r{reg}"),
    }
}

fn thumb_expand_imm12(imm12: u16) -> u32 {
    let imm8 = (imm12 & 0xFF) as u32;
    if (imm12 >> 10) == 0 {
        match (imm12 >> 8) & 0x3 {
            0 => imm8,
            1 => (imm8 << 16) | imm8,
            2 => (imm8 << 24) | (imm8 << 8),
            3 => (imm8 << 24) | (imm8 << 16) | (imm8 << 8) | imm8,
            _ => unreachable!(),
        }
    } else {
        let unrotated = 0x80 | (imm8 & 0x7F);
        let rot = ((imm12 >> 7) & 0x1F) as u32;
        unrotated.rotate_right(rot)
    }
}

fn decode_condition(bits: u16) -> Result<ArmCC, String> {
    Ok(match bits {
        0x0 => ArmCC::ARM_CC_EQ,
        0x1 => ArmCC::ARM_CC_NE,
        0x2 => ArmCC::ARM_CC_HS,
        0x3 => ArmCC::ARM_CC_LO,
        0x4 => ArmCC::ARM_CC_MI,
        0x5 => ArmCC::ARM_CC_PL,
        0x6 => ArmCC::ARM_CC_VS,
        0x7 => ArmCC::ARM_CC_VC,
        0x8 => ArmCC::ARM_CC_HI,
        0x9 => ArmCC::ARM_CC_LS,
        0xA => ArmCC::ARM_CC_GE,
        0xB => ArmCC::ARM_CC_LT,
        0xC => ArmCC::ARM_CC_GT,
        0xD => ArmCC::ARM_CC_LE,
        0xE => ArmCC::ARM_CC_AL,
        _ => return Err(format!("invalid condition bits {bits}")),
    })
}

fn cond_suffix(cond: ArmCC) -> &'static str {
    match cond {
        ArmCC::ARM_CC_EQ => "eq",
        ArmCC::ARM_CC_NE => "ne",
        ArmCC::ARM_CC_HS => "hs",
        ArmCC::ARM_CC_LO => "lo",
        ArmCC::ARM_CC_MI => "mi",
        ArmCC::ARM_CC_PL => "pl",
        ArmCC::ARM_CC_VS => "vs",
        ArmCC::ARM_CC_VC => "vc",
        ArmCC::ARM_CC_HI => "hi",
        ArmCC::ARM_CC_LS => "ls",
        ArmCC::ARM_CC_GE => "ge",
        ArmCC::ARM_CC_LT => "lt",
        ArmCC::ARM_CC_GT => "gt",
        ArmCC::ARM_CC_LE => "le",
        _ => "",
    }
}

fn cond_branch_mnemonic(suffix: &'static str) -> &'static str {
    match suffix {
        "eq" => "beq",
        "ne" => "bne",
        "hs" => "bhs",
        "lo" => "blo",
        "mi" => "bmi",
        "pl" => "bpl",
        "vs" => "bvs",
        "vc" => "bvc",
        "hi" => "bhi",
        "ls" => "bls",
        "ge" => "bge",
        "lt" => "blt",
        "gt" => "bgt",
        "le" => "ble",
        _ => "b",
    }
}

fn shift_type_to_kind(bits: u32) -> Result<ShiftKind, String> {
    match bits {
        0 => Ok(ShiftKind::Lsl),
        1 => Ok(ShiftKind::Lsr),
        2 => Ok(ShiftKind::Asr),
        3 => Ok(ShiftKind::Ror),
        _ => Err(format!("invalid shift type {bits}")),
    }
}

fn sign_extend(value: u32, bits: u32) -> i32 {
    let shift = 32 - bits;
    ((value << shift) as i32) >> shift
}

fn normal_outcome(mnemonic: &'static str, op_str: String, execute_cycles: u32, pc_update: u32) -> ExecOutcome {
    ExecOutcome {
        mnemonic,
        op_str,
        execute_cycles,
        pc_update,
    }
}

fn load_outcome(mnemonic: &'static str, op_str: String, rt: u32, size: u32) -> ExecOutcome {
    ExecOutcome {
        mnemonic,
        op_str,
        execute_cycles: 2,
        pc_update: if rt == 15 { 0 } else { size },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peripheral::bus::Bus;
    use std::sync::Arc;
    use std::sync::atomic::AtomicU32;

    fn make_cpu() -> Cpu {
        Cpu::new(Arc::new(AtomicU32::new(8_000_000)), 1, Bus::new(), Bus::new())
    }

    #[test]
    fn decode_cmp_immediate_t1() {
        let mut cpu = make_cpu();
        cpu.write_reg(0, 1);

        let outcome = decode_execute_16(&mut cpu, 0x0800_0474, 0x2801)
            .expect("cmp immediate should decode");

        assert_eq!(outcome.mnemonic, "cmp");
        assert_eq!(outcome.op_str, "r0, #1");
        assert_eq!(outcome.execute_cycles, 1);
        assert_eq!(outcome.pc_update, 2);
        assert_eq!((cpu.read_apsr() >> 30) & 1, 1, "Z flag should be set");
        assert_eq!((cpu.read_apsr() >> 29) & 1, 1, "C flag should be set");
    }

    #[test]
    fn decode_sub_sp_immediate_t1() {
        let mut cpu = make_cpu();
        cpu.write_sp(0x2000_0100);

        let outcome = decode_execute_16(&mut cpu, 0x0800_039A, 0xB085)
            .expect("sub sp immediate should decode");

        assert_eq!(outcome.mnemonic, "sub");
        assert_eq!(outcome.op_str, "sp, #0x14");
        assert_eq!(cpu.read_sp(), 0x2000_00EC);
        assert_eq!(outcome.pc_update, 2);
    }

    #[test]
    fn decode_strb_w_immediate() {
        let mut cpu = make_cpu();
        cpu.write_sp(0x2000_0100);
        cpu.write_reg(0, 0xA5);

        let outcome = decode_execute_32(&mut cpu, 0x0800_03AE, 0xF88D, 0x0013)
            .expect("strb.w immediate should decode");

        assert_eq!(outcome.mnemonic, "strb.w");
        assert_eq!(outcome.op_str, "r0, [r13, #0x13]");
        assert_eq!(read_u8(&mut cpu, 0x2000_0113), 0xA5);
        assert_eq!(outcome.pc_update, 4);
    }

    #[test]
    fn decode_strh_w_immediate() {
        let mut cpu = make_cpu();
        cpu.write_sp(0x2000_0100);
        cpu.write_reg(0, 0xBEEF);

        let outcome = decode_execute_32(&mut cpu, 0x0800_03B6, 0xF8AD, 0x0010)
            .expect("strh.w immediate should decode");

        assert_eq!(outcome.mnemonic, "strh.w");
        assert_eq!(outcome.op_str, "r0, [r13, #0x10]");
        assert_eq!(read_u16(&mut cpu, 0x2000_0110), 0xBEEF);
        assert_eq!(outcome.pc_update, 4);
    }

    #[test]
    fn decode_add_sp_immediate_t1() {
        let mut cpu = make_cpu();
        cpu.write_sp(0x2000_0100);

        let outcome = decode_execute_16(&mut cpu, 0x0800_03C0, 0xA904)
            .expect("add sp relative immediate should decode");

        assert_eq!(outcome.mnemonic, "add");
        assert_eq!(outcome.op_str, "r1, sp, #0x10");
        assert_eq!(cpu.read_reg(1), 0x2000_0110);
        assert_eq!(outcome.pc_update, 2);
    }

    #[test]
    fn decode_movw_immediate() {
        let mut cpu = make_cpu();

        let outcome = decode_execute_32(&mut cpu, 0x0800_05C2, 0xF64C, 0x70FF)
            .expect("movw immediate should decode");

        assert_eq!(outcome.mnemonic, "movw");
        assert_eq!(outcome.op_str, "r0, #0xCFFF");
        assert_eq!(cpu.read_reg(0), 0xCFFF);
        assert_eq!(outcome.pc_update, 4);
    }

    #[test]
    fn decode_udiv_t1() {
        let mut cpu = make_cpu();
        cpu.write_reg(5, 100);
        cpu.write_reg(3, 4);

        let outcome = decode_execute_32(&mut cpu, 0x0800_0378, 0xFBB5, 0xF5F3)
            .expect("udiv should decode");

        assert_eq!(outcome.mnemonic, "udiv");
        assert_eq!(outcome.op_str, "r5, r5, r3");
        assert_eq!(cpu.read_reg(5), 25);
        assert_eq!(outcome.pc_update, 4);
    }

    #[test]
    fn decode_mls_t1() {
        let mut cpu = make_cpu();
        cpu.write_reg(1, 6);
        cpu.write_reg(0, 7);
        cpu.write_reg(8, 100);

        let outcome = decode_execute_32(&mut cpu, 0x0800_063E, 0xFB01, 0x8910)
            .expect("mls should decode");

        assert_eq!(outcome.mnemonic, "mls");
        assert_eq!(outcome.op_str, "r9, r1, r0, r8");
        assert_eq!(cpu.read_reg(9), 58);
        assert_eq!(outcome.pc_update, 4);
    }

    #[test]
    fn decode_ubfx_t1() {
        let mut cpu = make_cpu();
        cpu.write_reg(1, 0xFFFF_03AB);

        let outcome = decode_execute_32(&mut cpu, 0x0800_067C, 0xF3C1, 0x0208)
            .expect("ubfx should decode");

        assert_eq!(outcome.mnemonic, "ubfx");
        assert_eq!(outcome.op_str, "r2, r1, #0, #9");
        assert_eq!(cpu.read_reg(2), 0x1AB);
        assert_eq!(outcome.pc_update, 4);
    }

    #[test]
    fn decode_ldrb_register_offset_t1() {
        let mut cpu = make_cpu();
        cpu.write_reg(5, 0x2000_0100);
        cpu.write_reg(1, 0x3);
        write_u8(&mut cpu, 0x2000_0103, 0x7A);

        let outcome = decode_execute_16(&mut cpu, 0x0800_0338, 0x5C6B)
            .expect("ldrb register offset should decode");

        assert_eq!(outcome.mnemonic, "ldrb");
        assert_eq!(outcome.op_str, "r3, [r5, r1]");
        assert_eq!(cpu.read_reg(3), 0x7A);
        assert_eq!(outcome.pc_update, 2);
    }
}

fn branch_outcome(mnemonic: &'static str, op_str: String, execute_cycles: u32) -> ExecOutcome {
    ExecOutcome {
        mnemonic,
        op_str,
        execute_cycles,
        pc_update: 0,
    }
}