use crate::arch::ArmInsn;
use crate::context::CpuContext;
use crate::opcodes::decoded::{DecodedInstructionBuilder, DecodedOperandKind};
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, OperandResolver, UpdateApsr_C, UpdateApsr_N,
    UpdateApsr_V, UpdateApsr_Z, check_condition, resolve_op2_runtime,
};

pub struct Cmp_builder;
impl InstrBuilder for Cmp_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_cmp_def()
    }
}

pub fn add_cmp_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![
        crate::opcodes::opcode::Opcode {
            insnid: ArmInsn::ARM_INS_CMP as u32,
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
            insnid: ArmInsn::ARM_INS_CMN as u32,
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
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
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
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
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
    fn resolve(&self, raw: &ArmOpcode, decoded: &mut DecodedInstructionBuilder) -> u32 {
        let rn = match decoded.get_operand(0) {
            Some(op) => match op.op_type {
                DecodedOperandKind::Reg(reg) => reg,
                _ => 0,
            },
            None => 0,
        };
        decoded.arm_operands.condition = raw.condition();
        decoded.arm_operands.rn = rn;
        decoded.arm_operands.op2 = decoded.get_operand(1).cloned();
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
    resolve_op2_runtime(cpu, data)
}
