use std::time::Instant;

use crate::context::CpuContext;
use capstone::prelude::*;
#[derive(Clone)]
pub struct Opcode {
    pub insnid: u32,
    pub name: String,
    pub length: u32,

    pub cycles: CycleInfo,

    pub exec: &'static dyn Executable,
    // pub operands: ArmOpcode<'a>,
    pub operand_resolver: &'static dyn OperandResolver,
    pub adjust_cycles: Option<CycleAdjustFn>,
}

impl Opcode {
    pub fn new(
        insnid: u32,
        name: String,
        length: u32,
        cycles: CycleInfo,

        exec: &'static dyn Executable,
        operand_resolver: &'static dyn OperandResolver,
        adjust_cycles: Option<CycleAdjustFn>,
    ) -> Opcode {
        Opcode {
            insnid,
            name,
            length,
            cycles,
            exec: exec,
            operand_resolver: operand_resolver,
            adjust_cycles: adjust_cycles,
        }
    }

    pub fn instantiate(&self) {}
}

pub trait Executable {
    fn execute(&self, cpu: &mut dyn crate::context::CpuContext, ops: &ArmOpcode) -> u32;
}

pub trait OperandResolver {
    fn resolve(&self, data: &mut ArmOpcode) -> u32;
}

#[derive(Clone, Copy)]
pub struct CycleInfo {
    pub fetch_cycles: u32,
    pub decode_cycles: u32,
    pub execute_cycles: u32,
}

impl CycleInfo {
    pub fn new(fetch: u32, decode: u32, execute: u32) -> CycleInfo {
        CycleInfo {
            fetch_cycles: fetch,
            decode_cycles: decode,
            execute_cycles: execute,
        }
    }
}

pub trait MatchFn {
    fn op_match(&self, ops: &ArmOpcode) -> bool;
}
pub type CycleAdjustFn = fn(&mut CycleInfo, &[ArmOperand]);

use capstone::arch::arm::{ArmInsnDetail, ArmOperand, ArmOperandType, ArmReg, ArmShift};
use capstone::{Insn, InsnDetail};

/// 封装了 ARM 指令及其详细信息的结构体

pub struct ArmOpcode<'a> {
    /// 原始指令对象
    pub insn: &'a Insn<'a>,
    pub detail: InsnDetail<'a>,
    /// Capstone 句柄引用，用于解析寄存器名
    cs: &'a Capstone,

    /// 转换后的操作数列表 (如寄存器编号、立即数值等)，由 OperandResolver 填充
    pub transed_operands: Vec<u32>,
    pub arm_operands: ArmOperands,
    /// 指令详细信息 (含操作数、CC、Writeback 等)
    ///
    /// 注意：这个字段在创建 ArmOpcode 时就被解析并存储，避免了后续多次调用 cs.insn_detail() 的性能开销
    pub update_flags: bool,

    pub update_carry: u8,
}

impl<'a> ArmOpcode<'a> {
    /// Helper to resolve register to u32
    pub fn resolve_reg(&self, reg: capstone::RegId) -> u32 {
        match reg.0 {
            x if x == ArmReg::ARM_REG_R0 as u16 => 0,
            x if x == ArmReg::ARM_REG_R1 as u16 => 1,
            x if x == ArmReg::ARM_REG_R2 as u16 => 2,
            x if x == ArmReg::ARM_REG_R3 as u16 => 3,
            x if x == ArmReg::ARM_REG_R4 as u16 => 4,
            x if x == ArmReg::ARM_REG_R5 as u16 => 5,
            x if x == ArmReg::ARM_REG_R6 as u16 => 6,
            x if x == ArmReg::ARM_REG_R7 as u16 => 7,
            x if x == ArmReg::ARM_REG_R8 as u16 => 8,
            x if x == ArmReg::ARM_REG_R9 as u16 => 9,
            x if x == ArmReg::ARM_REG_R10 as u16 => 10,
            x if x == ArmReg::ARM_REG_R11 as u16 => 11,
            x if x == ArmReg::ARM_REG_R12 as u16 => 12,
            x if x == ArmReg::ARM_REG_R13 as u16 => 13,
            x if x == ArmReg::ARM_REG_R14 as u16 => 14,
            x if x == ArmReg::ARM_REG_R15 as u16 => 15,

            x if x == ArmReg::ARM_REG_SP as u16 => 13,
            x if x == ArmReg::ARM_REG_LR as u16 => 14,
            x if x == ArmReg::ARM_REG_PC as u16 => 15,

            x if x == ArmReg::ARM_REG_SB as u16 => 9,
            x if x == ArmReg::ARM_REG_SL as u16 => 10,
            x if x == ArmReg::ARM_REG_FP as u16 => 11,
            x if x == ArmReg::ARM_REG_IP as u16 => 12,

            _ => 0,
        }
    }

    /// 从原始 Insn 创建 ArmOpcode
    pub fn new(cs: &'a Capstone, insn: &'a Insn<'a>) -> Option<Self> {
        let start = Instant::now();
        let detail = cs.insn_detail(insn).ok()?;

        if let arch::ArchDetail::ArmDetail(_) = detail.arch_detail() {
            let _duration = start.elapsed();
            // println!("ArmOpcode::new execution time: {:?}", duration);
            Some(ArmOpcode {
                insn,
                detail,
                cs,
                transed_operands: Vec::new(),
                arm_operands: ArmOperands::new(),
                update_flags: false,
                update_carry: 0,
            })
        } else {
            None
        }
    }

    pub fn trans_operands(&mut self) {
        let start = Instant::now();
        // 解构 self 以避免借用冲突
        let detail = &self.detail;
        let cs = &self.cs;

        let arch_detail = if let arch::ArchDetail::ArmDetail(arm) = detail.arch_detail() {
            arm
        } else {
            panic!("ArmOpcode has invalid detail");
        };

        // Helper closure to resolve register to u32
        let get_reg_val = |reg: capstone::RegId| -> Option<u32> {
            if let Some(reg_name) = cs.reg_name(reg) {
                if reg_name.starts_with('r') {
                    if let Ok(reg_num) = reg_name[1..].parse::<u32>() {
                        return Some(reg_num);
                    }
                } else if reg_name == "sp" {
                    return Some(13);
                } else if reg_name == "lr" {
                    return Some(14);
                } else if reg_name == "pc" {
                    return Some(15);
                }
            }
            None
        };

        self.transed_operands.clear();
        for op in arch_detail.operands() {
            match op.op_type {
                ArmOperandType::Reg(reg) => {
                    if let Some(reg_num) = get_reg_val(reg) {
                        self.transed_operands.push(reg_num);
                    }
                }
                ArmOperandType::Imm(val) => {
                    self.transed_operands.push(val as u32);
                }
                ArmOperandType::Mem(mem) => {
                    if let Some(reg_num) = get_reg_val(mem.base()) {
                        self.transed_operands.push(reg_num);
                    }
                    if let Some(reg_num) = get_reg_val(mem.index()) {
                        self.transed_operands.push(reg_num);
                    }
                    self.transed_operands.push(mem.disp() as u32);
                }
                ArmOperandType::Pimm(val) | ArmOperandType::Cimm(val) => {
                    self.transed_operands.push(val as u32);
                }
                _ => {}
            }
        }
        let duration = start.elapsed();
        println!("ArmOpcode::trans_operands execution time: {:?}", duration);
    }

    pub fn op_writer(&self) {
        // println!("op_str: {}", self.insn.op_str().unwrap_or(""));
    }

    /// 获取指令 ID (u32)
    pub fn id(&self) -> u32 {
        self.insn.id().0
    }

    pub fn address(&self) -> u32 {
        self.insn.address() as u32
    }

    pub fn size(&self) -> u32 {
        self.insn.len() as u32
    }

    /// 获取助记符 (如 "ldr")
    pub fn mnemonic(&self) -> &str {
        self.insn.mnemonic().unwrap_or("")
    }

    /// 获取操作数字符串 (如 "r0, [pc, #0x20]")
    pub fn op_str(&self) -> &str {
        self.insn.op_str().unwrap_or("")
    }

    fn arm_detail(&self) -> ArmInsnDetail {
        if let arch::ArchDetail::ArmDetail(arm) = self.detail.arch_detail() {
            arm
        } else {
            panic!("ArmOpcode has invalid detail");
        }
    }

    /// 获取所有操作数的迭代器
    pub fn operands(&self) -> impl Iterator<Item = ArmOperand> {
        self.arm_detail().operands().collect::<Vec<_>>().into_iter()
    }

    /// 快捷方法：获取特定索引的操作数
    pub fn get_operand(&self, index: usize) -> Option<ArmOperand> {
        self.operands().nth(index)
    }

    pub fn update_flags(&self) -> bool {
        self.arm_detail().update_flags()
    }

    pub fn writeback(&self) -> bool {
        self.arm_detail().writeback()
    }

    pub fn post_index(&self) -> bool {
        self.writeback() && self.op_str().contains("],")
    }

    pub fn condition(&self) -> capstone::arch::arm::ArmCC {
        self.arm_detail().cc()
    }

    pub fn it_mask(&self) -> u8 {
        // self.arm_detail().it_mask()
        0
    }
}

pub type Op2_ResolverFn = fn(&mut dyn CpuContext, &ArmOpcode) -> u32;
pub struct ArmOperands {
    pub rd: u32,
    pub rn: u32,
    pub op2: Option<ArmOperand>,
    pub op2_value: u32,

    pub mem_disp: i32,
    pub mem_has_index: bool,
    pub mem_writeback: bool,
    pub mem_post_index: bool,
    pub mem_post_imm: i32,

    pub op2_resolver: Option<Op2_ResolverFn>,
}
impl ArmOperands {
    pub fn new() -> Self {
        ArmOperands {
            rd: 0,
            rn: 0,
            op2: None,
            op2_value: 0,
            mem_disp: 0,
            mem_has_index: false,
            mem_writeback: false,
            mem_post_index: false,
            mem_post_imm: 0,
            op2_resolver: None,
        }
    }

}

pub fn UpdateApsr_N(cpu: &mut dyn crate::context::CpuContext, result: u32) {
    // N = bit31 of result
    let mut apsr = cpu.read_apsr();
    if (result & (1u32 << 31)) != 0 {
        apsr |= 1u32 << 31;
    } else {
        apsr &= !(1u32 << 31);
    }
    cpu.write_apsr(apsr);
}

pub fn UpdateApsr_Z(cpu: &mut dyn crate::context::CpuContext, result: u32) {
    // Z = 1 when result == 0
    let mut apsr = cpu.read_apsr();
    if result == 0 {
        apsr |= 1u32 << 30;
    } else {
        apsr &= !(1u32 << 30);
    }
    cpu.write_apsr(apsr);
}

pub fn UpdateApsr_C(_cpu: &mut dyn crate::context::CpuContext, flag: u8) {
    // C = carry flag (bit 29)
    let mut apsr = _cpu.read_apsr();
    if flag != 0 {
        apsr |= 1u32 << 29;
    } else {
        apsr &= !(1u32 << 29);
    }
    _cpu.write_apsr(apsr);
}

pub fn UpdateApsr_V(cpu: &mut dyn crate::context::CpuContext, flag: u8) {
    // V = overflow flag (bit 28)
    let mut apsr = cpu.read_apsr();
    if flag != 0 {
        apsr |= 1u32 << 28;
    } else {
        apsr &= !(1u32 << 28);
    }
    cpu.write_apsr(apsr);
}
use capstone::arch::arm::ArmCC;
pub fn check_condition(cpu: &dyn CpuContext, cc: ArmCC) -> bool {
    let apsr = cpu.read_apsr();
    let n = (apsr >> 31) & 1;
    let z = (apsr >> 30) & 1;
    let c = (apsr >> 29) & 1;
    let v = (apsr >> 28) & 1;

    match cc {
        ArmCC::ARM_CC_EQ => z == 1,
        ArmCC::ARM_CC_NE => z == 0,
        ArmCC::ARM_CC_HS => c == 1,
        ArmCC::ARM_CC_LO => c == 0,
        ArmCC::ARM_CC_MI => n == 1,
        ArmCC::ARM_CC_PL => n == 0,
        ArmCC::ARM_CC_VS => v == 1,
        ArmCC::ARM_CC_VC => v == 0,
        ArmCC::ARM_CC_HI => c == 1 && z == 0,
        ArmCC::ARM_CC_LS => c == 0 || z == 1,
        ArmCC::ARM_CC_GE => n == v,
        ArmCC::ARM_CC_LT => n != v,
        ArmCC::ARM_CC_GT => z == 0 && (n == v),
        ArmCC::ARM_CC_LE => z == 1 || (n != v),
        ArmCC::ARM_CC_AL => true,
        _ => true,
    }
}


pub fn operand_resolver_multi_runtime(
    cpu: &mut dyn crate::context::CpuContext,
    data: &ArmOpcode,
) -> (u32, u32) {
    let arch_detail = if let arch::ArchDetail::ArmDetail(arm) = data.detail.arch_detail() {
        arm
    } else {
        panic!("ArmOpcode has invalid detail");
    };
    let mut operands = arch_detail.operands();
    let op_rt = operands.next().expect("missing rt operand");
    let op2 = operands.next().expect("missing mem operand");
    let op3 = operands.next();

    let rt = match op_rt.op_type {
        ArmOperandType::Reg(r) => data.resolve_reg(r),
        _ => panic!("first operand is not a register"),
    };
    let writeback = data.writeback();
    let post_index = op3.is_some();

    if !writeback {
        let addr = match op2.op_type {
            ArmOperandType::Mem(mem) => {
                let base = cpu.read_reg(data.resolve_reg(mem.base()));

                let disp = mem.disp();
                if mem.index() != capstone::RegId::INVALID_REG {
                    let val = cpu.read_reg(data.resolve_reg(mem.index()));
                    let current_c = (cpu.read_apsr() >> 29) as u8 & 1;
                    let (r2_val, _carry) = op_shift_match_by_shift(op2.shift, val, current_c);
                    base.wrapping_add(r2_val)
                } else {
                    base.wrapping_add_signed(disp)
                }
            }
            _ => panic!("operand2 is not a memory operand"),
        };
        (rt, addr)
    } else {
        let addr = match op2.op_type {
            ArmOperandType::Mem(mem) => {
                let base = data.resolve_reg(mem.base());
                let disp = mem.disp();
                if post_index {
                    let op3 = op3.expect("missing post-index immediate");
                    let op3_offset = match op3.op_type {
                        ArmOperandType::Imm(imm) => imm,
                        _ => panic!("third operand is not an immediate"),
                    };
                    base.wrapping_add_signed(op3_offset)
                } else {
                    base.wrapping_add_signed(disp)
                }
            }
            _ => panic!("operand2 is not a memory operand"),
        };
        (rt, addr)
    }
}

pub fn op2_imm_match(data: &ArmOpcode) -> bool {
    let len = data.transed_operands.len();

    // AND: 只允许 2 或 3 个 operand
    if len != 2 && len != 3 {
        return false;
    }

    // Operand2 在最后一位
    let op2 = match data.get_operand(len - 1) {
        Some(op) => op,
        None => return false,
    };

    match &op2.op_type {
        // Operand2 = immediate
        ArmOperandType::Imm(_) => true,

        // 其他一律非法
        _ => false,
    }
}

pub fn op2_reg_match(data: &ArmOpcode) -> bool {
    let len = data.transed_operands.len();

    // AND: 只允许 2 或 3 个 operand
    if len != 2 && len != 3 {
        return false;
    }

    // Operand2 在最后一位
    let op2 = match data.get_operand(len - 1) {
        Some(op) => op,
        None => return false,
    };

    match &op2.op_type {
        // Operand2 = register (with optional shift)
        ArmOperandType::Reg(_) => true,

        // 其他一律非法
        _ => false,
    }
}


//return (value after shift , carry)
fn op_shift_match(op2: ArmOperand, val: u32, current_c: u8) -> (u32, u8) {
    op_shift_match_by_shift(op2.shift, val, current_c)
}

fn op_shift_match_by_shift(shift_kind: ArmShift, val: u32, current_c: u8) -> (u32, u8) {
    match shift_kind {
        ArmShift::Lsl(shift) => {
            // LSL, Logical Shift Left
            match shift {
                0 => (val, current_c),
                1..=31 => {
                    let carry = (val >> (32 - shift)) & 1;
                    (val << shift, carry as u8)
                }
                32 => (0, (val & 1) as u8),
                _ => panic!("Lsl invalid shift amount"),
            }
        }
        ArmShift::Lsr(shift) => {
            // LSR, Logical Shift Right
            match shift {
                0 => (val, current_c),
                1..=31 => {
                    let carry = (val >> (shift - 1)) & 1;
                    (val >> shift, carry as u8)
                }
                32 => (0, (val >> 31) as u8),
                _ => panic!("Lsr invalid shift amount"),
            }
        }
        ArmShift::Asr(shift) => {
            // ASR, Arithmetic Shift Right
            match shift {
                0 => (val, current_c),
                1..=31 => {
                    let carry = (val >> (shift - 1)) & 1;
                    let res = ((val as i32) >> shift) as u32;
                    (res, carry as u8)
                }
                _ => {
                    // shift >= 32
                    let carry = (val >> 31) & 1;
                    let res = if (val as i32) < 0 { 0xFFFFFFFF } else { 0 };
                    (res, carry as u8)
                }
            }
        }
        ArmShift::Ror(shift) => {
            // ROR, Rotate Right
            if shift == 0 {
                (val, current_c)
            } else {
                let shift_mod = shift % 32;
                if shift_mod == 0 {
                    (val, (val >> 31) as u8)
                } else {
                    let res = val.rotate_right(shift_mod);
                    let carry = (res >> 31) & 1;
                    (res, carry as u8)
                }
            }
        }
        ArmShift::Rrx(_) => {
            // RRX, Rotate Right with Extend
            let c_out = (val & 1) as u8;
            let res = (val >> 1) | ((current_c as u32) << 31);
            (res, c_out)
        }
        _ => (val, current_c),
    }
}

pub fn resolve_op2_runtime(cpu: &mut dyn crate::context::CpuContext, data: &ArmOpcode) -> (u32, u8) {
    let current_c = ((cpu.read_apsr() >> 29) & 1) as u8;
    let Some(op2) = &data.arm_operands.op2 else {
        return (0, current_c);
    };

    match op2.op_type {
        ArmOperandType::Reg(reg) => {
            let value = cpu.read_reg(data.resolve_reg(reg));
            op_shift_match(op2.clone(), value, current_c)
        }
        ArmOperandType::Imm(imm) => (imm as u32, current_c),
        _ => (0, current_c),
    }
}
