use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, OperandResolver, UpdateApsr_C, UpdateApsr_N, UpdateApsr_V,
    UpdateApsr_Z, check_condition, resolve_op2_runtime,
};
use capstone::arch::arm::ArmOperandType;

pub struct Calculate_builder;
impl InstrBuilder for Calculate_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_calculate_def()
    }
}

pub fn add_calculate_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_ADD as u32,
            name: "ADD".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Add::execute,
            operand_resolver: &OpCalculateResolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_ADC as u32,
            name: "ADC".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Adc::execute,
            operand_resolver: &OpCalculateResolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_SUB as u32,
            name: "SUB".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Sub::execute,
            operand_resolver: &OpCalculateResolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_SBC as u32,
            name: "SBC".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Sbc::execute,
            operand_resolver: &OpCalculateResolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_RSB as u32,
            name: "RSB".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Rsb::execute,
            operand_resolver: &OpCalculateResolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_MUL as u32,
            name: "MUL".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Mul::execute,
            operand_resolver: &OpCalculateResolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_UDIV as u32,
            name: "UDIV".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Udiv::execute,
            operand_resolver: &OpCalculateResolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_MLS as u32,
            name: "MLS".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Mls::execute,
            operand_resolver: &OpCalculateResolver,
            adjust_cycles: None,
        },
        // Similarly for SUB, SBC, RSB...
    ]
}

pub struct OpCalculateResolver;
impl OperandResolver for OpCalculateResolver {
    fn resolve(&self, data: &mut ArmOpcode) -> u32 {
        let rd = match data.get_operand(0) {
            Some(op) => match op.op_type {
                ArmOperandType::Reg(r) => data.resolve_reg(r),
                _ => 0,
            },
            None => 0,
        };
        let rn = match data.get_operand(1) {
            Some(op) => match op.op_type {
                ArmOperandType::Reg(r) => data.resolve_reg(r),
                _ => rd,
            },
            None => rd,
        };
        data.arm_operands.condition = data.condition();
        data.arm_operands.rd = rd;
        data.arm_operands.rn = rn;
        data.arm_operands.op2 = data.get_operand(2).or_else(|| data.get_operand(1));
        rd
    }
}

// ADD, ADC, SUB, SBC, RSB
// op{S}{cond} {Rd,} Rn, Operand2

pub struct Op_Add;
impl Executable for Op_Add {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        let rd = data.arm_operands.rd;
        let rn = data.arm_operands.rn;
        let (op2, _) = resolve_op2_runtime(cpu, data);
        calculate_add_core(cpu, data, rd, rn, op2);
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Adc;
impl Executable for Op_Adc {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        let rd = data.arm_operands.rd;
        let rn = data.arm_operands.rn;
        let (op2, _) = resolve_op2_runtime(cpu, data);
        calculate_adc_core(cpu, data, rd, rn, op2);
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Sub;
impl Executable for Op_Sub {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        let rd = data.arm_operands.rd;
        let rn = data.arm_operands.rn;
        let (op2, _) = resolve_op2_runtime(cpu, data);
        calculate_sub_core(cpu, data, rd, rn, op2);
        // print!("SUB: Rn={:#X}, Op2={:#X}\n", cpu.read_reg(rn), op2);
        // print!("SUB Result: {:#X}\n", cpu.read_reg(rd));
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Sbc;
impl Executable for Op_Sbc {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        let rd = data.arm_operands.rd;
        let rn = data.arm_operands.rn;
        let (op2, _) = resolve_op2_runtime(cpu, data);
        calculate_sbc_core(cpu, data, rd, rn, op2);
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Rsb;
impl Executable for Op_Rsb {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        let rd = data.arm_operands.rd;
        let rn = data.arm_operands.rn;
        let (op2, _) = resolve_op2_runtime(cpu, data);
        calculate_rsb_core(cpu, data, rd, rn, op2);
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Mul;
impl Executable for Op_Mul {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        let rd = data.arm_operands.rd;
        let rn = data.arm_operands.rn;
        let (op2, _) = resolve_op2_runtime(cpu, data);
        calculate_mul_core(cpu, data, rd, rn, op2);
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Udiv;
impl Executable for Op_Udiv {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        let rd = data.arm_operands.rd;
        let rn = data.arm_operands.rn;
        let (op2, _) = resolve_op2_runtime(cpu, data);
        calculate_udiv_core(cpu, rd, rn, op2);
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Mls;
impl Executable for Op_Mls {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }
        let rd = data.arm_operands.rd;
        let rn = data.arm_operands.rn;
        let (op2, _) = resolve_op2_runtime(cpu, data);
        let ra = match data.get_operand(3) {
            Some(op) => match op.op_type {
                ArmOperandType::Reg(r) => data.resolve_reg(r),
                _ => 0,
            },
            None => 0,
        };
        calculate_mls_core(cpu, rd, rn, op2, ra);
        if rd == 15 { 0 } else { data.size() }
    }
}

fn calculate_add_core(cpu: &mut dyn CpuContext, data: &ArmOpcode, rd: u32, rn: u32, op2_val: u32) {
    let rn_val = cpu.read_reg(rn);

    let result = rn_val.wrapping_add(op2_val);
    cpu.write_reg(rd, result);

    if data.update_flags() {
        UpdateApsr_Z(cpu, result);
        UpdateApsr_N(cpu, result);

        // Carry: unsigned overflow
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
}

// === ADC ===

fn calculate_adc_core(cpu: &mut dyn CpuContext, data: &ArmOpcode, rd: u32, rn: u32, op2_val: u32) {
    let rn_val = cpu.read_reg(rn);
    let apsr = cpu.read_apsr();
    let carry_in = if (apsr & (1u32 << 29)) != 0 {
        1u32
    } else {
        0u32
    };

    let wide = (rn_val as u64) + (op2_val as u64) + (carry_in as u64);
    let result = (wide & 0xffff_ffff) as u32;
    let carry_out = if wide > 0xffff_ffffu64 { 1u8 } else { 0u8 };

    cpu.write_reg(rd, result);

    if data.update_flags() {
        UpdateApsr_Z(cpu, result);
        UpdateApsr_N(cpu, result);
        UpdateApsr_C(cpu, carry_out);

        // Overflow for ADC
        let rn_i = rn_val as i32 as i64;
        let op2_i = op2_val as i32 as i64;
        let v = if (rn_i > 0 && op2_i > 0 && (rn_i + op2_i + (carry_in as i64)) < 0)
            || (rn_i < 0 && op2_i < 0 && (rn_i + op2_i + (carry_in as i64)) >= 0)
        {
            1u8
        } else {
            0u8
        };
        UpdateApsr_V(cpu, v);
    }
}

// === SUB ===

fn calculate_sub_core(cpu: &mut dyn CpuContext, data: &ArmOpcode, rd: u32, rn: u32, op2_val: u32) {
    let rn_val = cpu.read_reg(rn);

    let result = rn_val.wrapping_sub(op2_val);
    cpu.write_reg(rd, result);

    if data.update_flags() {
        UpdateApsr_Z(cpu, result);
        UpdateApsr_N(cpu, result);

        // Carry for subtraction: set if no borrow (Rn >= Op2)
        let carry = if rn_val >= op2_val { 1u8 } else { 0u8 };
        UpdateApsr_C(cpu, carry);

        // Overflow
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
}

// === SBC ===

fn calculate_sbc_core(cpu: &mut dyn CpuContext, data: &ArmOpcode, rd: u32, rn: u32, op2_val: u32) {
    let rn_val = cpu.read_reg(rn);
    let apsr = cpu.read_apsr();
    let carry_in = if (apsr & (1u32 << 29)) != 0 {
        1u32
    } else {
        0u32
    };

    // SBC = Rn - Op2 - (1 - C)
    let borrow = 1u32 - carry_in;
    let wide = (rn_val as u64).wrapping_sub((op2_val as u64) + (borrow as u64));
    let result = wide as u32;

    cpu.write_reg(rd, result);

    if data.update_flags() {
        UpdateApsr_Z(cpu, result);
        UpdateApsr_N(cpu, result);

        // Carry set if no borrow
        let carry = if (rn_val as u64) >= ((op2_val as u64) + (borrow as u64)) {
            1u8
        } else {
            0u8
        };
        UpdateApsr_C(cpu, carry);

        let rn_i = rn_val as i32 as i64;
        let op2_i = op2_val as i32 as i64;
        let res_i = result as i32 as i64;
        let v = if ((rn_i ^ op2_i) & (rn_i ^ res_i) & (1i64 << 31)) != 0 {
            1u8
        } else {
            0u8
        };
        UpdateApsr_V(cpu, v);
    }
}

// === RSB ===

fn calculate_rsb_core(cpu: &mut dyn CpuContext, data: &ArmOpcode, rd: u32, rn: u32, op2_val: u32) {
    let rn_val = cpu.read_reg(rn);

    // RSB: result = Op2 - Rn
    let result = op2_val.wrapping_sub(rn_val);
    cpu.write_reg(rd, result);

    if data.update_flags() {
        UpdateApsr_Z(cpu, result);
        UpdateApsr_N(cpu, result);

        // Carry: set if Op2 >= Rn
        let carry = if op2_val >= rn_val { 1u8 } else { 0u8 };
        UpdateApsr_C(cpu, carry);

        let op2_i = op2_val as i32;
        let rn_i = rn_val as i32;
        let res_i = result as i32;
        let v = if ((op2_i ^ rn_i) & (op2_i ^ res_i) & (1i32 << 31)) != 0 {
            1u8
        } else {
            0u8
        };

        UpdateApsr_V(cpu, v);
    }
}

// === MUL ===

fn calculate_mul_core(cpu: &mut dyn CpuContext, data: &ArmOpcode, rd: u32, rn: u32, op2_val: u32) {
    let rn_val = cpu.read_reg(rn);
    let result = rn_val.wrapping_mul(op2_val);
    cpu.write_reg(rd, result);

    if data.update_flags() {
        UpdateApsr_Z(cpu, result);
        UpdateApsr_N(cpu, result);
    }
}

// === UDIV ===

fn calculate_udiv_core(cpu: &mut dyn CpuContext, rd: u32, rn: u32, op2_val: u32) {
    let rn_val = cpu.read_reg(rn);
    let result = if op2_val == 0 { 0 } else { rn_val / op2_val };
    cpu.write_reg(rd, result);
}

// === MLS ===

fn calculate_mls_core(cpu: &mut dyn CpuContext, rd: u32, rn: u32, op2_val: u32, ra: u32) {
    let rn_val = cpu.read_reg(rn);
    let ra_val = cpu.read_reg(ra);
    let result = ra_val.wrapping_sub(rn_val.wrapping_mul(op2_val));
    cpu.write_reg(rd, result);
}
