use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, OperandResolver, UpdateApsr_C, UpdateApsr_N,
    UpdateApsr_V, UpdateApsr_Z, check_condition,
};
use capstone::arch::arm::{ArmOperandType, ArmShift};

pub struct Cmp_builder;
impl InstrBuilder for Cmp_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_cmp_def()
    }
}

pub fn add_cmp_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_CMP as u32,
            name: "CMP".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Cmp::execute,
            operand_resolver: &OpCmpResolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_CMN as u32,
            name: "CMN".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Cmn::execute,
            operand_resolver: &OpCmpResolver,
            adjust_cycles: None,
        },
    ]
}

// CMP{cond} Rn, Operand2
// CMN{cond} Rn, Operand2

pub struct Op_Cmp;
impl Executable for Op_Cmp {
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        let rn = data.arm_operands.rn;
        let (op2, _shifter_carry) = resolve_op2_and_carry(cpu, data);
        cmp_core(cpu, data, rn, op2);
        data.size()
    }
}

pub struct Op_Cmn;
impl Executable for Op_Cmn {
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }
        let rn = data.arm_operands.rn;
        let (op2, _shifter_carry) = resolve_op2_and_carry(cpu, data);
        cmn_core(cpu, data, rn, op2);
        data.size()
    }
}

pub struct OpCmpResolver;
impl OperandResolver for OpCmpResolver {
    fn resolve(&self, data: &mut ArmOpcode) -> u32 {
        let rn = match data.get_operand(0) {
            Some(op) => match op.op_type {
                ArmOperandType::Reg(r) => data.resolve_reg(r),
                _ => 0,
            },
            None => 0,
        };
        data.arm_operands.rn = rn;
        data.arm_operands.op2 = data.get_operand(1);
        rn
    }
}

// === CMP ===
// CMP is effectively a SUBS Opcode with the result discarded.
fn cmp_core(cpu: &mut dyn CpuContext, _data: &ArmOpcode, rn: u32, op2_val: u32) {
    let rn_val = cpu.read_reg(rn);
    // Rn - Op2
    let result = rn_val.wrapping_sub(op2_val);

    // print!("Comparing R{} (0x{:08X}) with Op2 (0x{:08X}): Result = 0x{:08X}\n", rn, rn_val, op2_val, result);

    UpdateApsr_Z(cpu, result);
    UpdateApsr_N(cpu, result);

    // Carry: set if no borrow (Rn >= Op2)
    let carry = if rn_val >= op2_val { 1u8 } else { 0u8 };
    UpdateApsr_C(cpu, carry);

    // Overflow: signed overflow for subtraction
    let rn_i = rn_val as i32;
    let op2_i = op2_val as i32;
    let res_i = result as i32;
    let v = if ((rn_i ^ op2_i) & (rn_i ^ res_i) & (1i32 << 31)) != 0 {
        1u8
    } else {
        0u8
    };
    UpdateApsr_V(cpu, v);
}

// === CMN ===
// CMN is effectively an ADDS Opcode with the result discarded.
fn cmn_core(cpu: &mut dyn CpuContext, _data: &ArmOpcode, rn: u32, op2_val: u32) {
    let rn_val = cpu.read_reg(rn);
    // Rn + Op2
    let result = rn_val.wrapping_add(op2_val);

    UpdateApsr_Z(cpu, result);
    UpdateApsr_N(cpu, result);

    // Carry: unsigned overflow for addition
    let carry = if (rn_val as u64) + (op2_val as u64) > 0xffff_ffffu64 {
        1u8
    } else {
        0u8
    };
    UpdateApsr_C(cpu, carry);

    // Overflow: signed overflow for addition
    let rn_i = rn_val as i32;
    let op2_i = op2_val as i32;
    let res_i = result as i32;
    let v = if (rn_i > 0 && op2_i > 0 && res_i < 0) || (rn_i < 0 && op2_i < 0 && res_i >= 0) {
        1u8
    } else {
        0u8
    };
    UpdateApsr_V(cpu, v);
}

fn resolve_op2_and_carry(cpu: &mut dyn CpuContext, data: &ArmOpcode) -> (u32, u8) {
    let current_c = ((cpu.read_apsr() >> 29) & 1) as u8;
    let Some(op2) = &data.arm_operands.op2 else {
        return (0, current_c);
    };

    match op2.op_type {
        ArmOperandType::Reg(reg) => {
            let value = cpu.read_reg(data.resolve_reg(reg));
            op_shift_match(op2.shift, value, current_c)
        }
        ArmOperandType::Imm(imm) => (imm as u32, current_c),
        _ => (0, current_c),
    }
}

fn op_shift_match(shift: ArmShift, value: u32, current_c: u8) -> (u32, u8) {
    match shift {
        ArmShift::Lsl(amount) => match amount {
            0 => (value, current_c),
            1..=31 => {
                let carry = ((value >> (32 - amount)) & 1) as u8;
                (value.wrapping_shl(amount), carry)
            }
            32 => (0, (value & 1) as u8),
            _ => (0, 0),
        },
        ArmShift::Lsr(amount) => match amount {
            0 => (value, current_c),
            1..=31 => {
                let carry = ((value >> (amount - 1)) & 1) as u8;
                (value >> amount, carry)
            }
            32 => (0, (value >> 31) as u8),
            _ => (0, 0),
        },
        ArmShift::Asr(amount) => match amount {
            0 => (value, current_c),
            1..=31 => {
                let carry = ((value >> (amount - 1)) & 1) as u8;
                (((value as i32) >> amount) as u32, carry)
            }
            _ => {
                let carry = ((value >> 31) & 1) as u8;
                let result = if (value as i32) < 0 { 0xFFFF_FFFF } else { 0 };
                (result, carry)
            }
        },
        ArmShift::Ror(amount) => {
            if amount == 0 {
                (value, current_c)
            } else {
                let shift_mod = amount % 32;
                if shift_mod == 0 {
                    (value, (value >> 31) as u8)
                } else {
                    let result = value.rotate_right(shift_mod);
                    (result, ((result >> 31) & 1) as u8)
                }
            }
        }
        ArmShift::Rrx(_) => {
            let carry = (value & 1) as u8;
            let result = (value >> 1) | ((current_c as u32) << 31);
            (result, carry)
        }
        _ => (value, current_c),
    }
}
