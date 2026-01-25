use crate::context::CpuContext;
use crate::opcodes::Instructions::str;
use crate::opcodes::opcode::{
    ArmInstruction, Executable, MatchFn, Operand2_resolver, UpdateApsr_C, UpdateApsr_N,
    UpdateApsr_V, UpdateApsr_Z, check_condition, op2_imm_match, op2_reg_match,
};
use crate::opcodes::instruction::{InstrBuilder};
use capstone::arch::arm::ArmOperandType;

pub struct Calculate_builder;
impl InstrBuilder for Calculate_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Instruction> {
        add_calculate_def()
    }
}

pub fn add_calculate_def() -> Vec<crate::opcodes::opcode::Instruction> {
    vec![
        crate::opcodes::opcode::Instruction {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_ADD as u32,
            name: "ADD".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Add,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Instruction {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_ADC as u32,
            name: "ADC".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Adc,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Instruction {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_SUB as u32,
            name: "SUB".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Sub,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Instruction {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_SBC as u32,
            name: "SBC".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Sbc,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Instruction {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_RSB as u32,
            name: "RSB".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Rsb,
            adjust_cycles: None,
        },

        // Similarly for SUB, SBC, RSB...
    ]
}

// ADD, ADC, SUB, SBC, RSB
// op{S}{cond} {Rd,} Rn, Operand2

pub struct Op_Add;
impl Executable for Op_Add {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rd, rn, op2) = Operand2_resolver(cpu, data);
        calculate_add_core(cpu, data, rd, rn, op2);
    }
}

pub struct Op_Adc;
impl Executable for Op_Adc {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rd, rn, op2) = Operand2_resolver(cpu, data);
        calculate_adc_core(cpu, data, rd, rn, op2);
    }
}

pub  struct Op_Sub;
impl Executable for Op_Sub {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rd, rn, op2) = Operand2_resolver(cpu, data);
        calculate_sub_core(cpu, data, rd, rn, op2);
    }
}

pub struct Op_Sbc;
impl Executable for Op_Sbc {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rd, rn, op2) = Operand2_resolver(cpu, data);
        calculate_sbc_core(cpu, data, rd, rn, op2);
    }
}

pub struct Op_Rsb;
impl Executable for Op_Rsb {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rd, rn, op2) = Operand2_resolver(cpu, data);
        calculate_rsb_core(cpu, data, rd, rn, op2);
    }
}





fn calculate_add_core(
    cpu: &mut dyn CpuContext,
    data: &ArmInstruction,
    rd: u32,
    rn: u32,
    op2_val: u32,
) {
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



// pub struct Calculate_Add_Imm;
// impl Executable for Calculate_Add_Imm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
//         if !check_condition(cpu, data.condition()) {
//             return;
//         }
//         let (rd, rn, imm) = Operand2_resolver(cpu, data);
//         calculate_add_core(cpu, data, rd, rn, imm);
//     }
// }
// pub struct Calculate_Add_Imm_match;
// impl MatchFn for Calculate_Add_Imm_match {
//     fn op_match(&self, data: &ArmInstruction) -> bool {
//         op2_imm_match(data)
//     }
// }

// pub struct Calculate_Add_Reg;
// impl Executable for Calculate_Add_Reg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
//         if !check_condition(cpu, data.condition()) {
//             return;
//         }
//         let (rd, rn, rm) = Operand2_resolver(cpu, data);
//         let val = cpu.read_reg(rm);
//         calculate_add_core(cpu, data, rd, rn, val);
//     }
// }
// pub struct Calculate_Add_Reg_match;
// impl MatchFn for Calculate_Add_Reg_match {
//     fn op_match(&self, data: &ArmInstruction) -> bool {
//         op2_reg_match(data)
//     }
// }

// === ADC ===

fn calculate_adc_core(
    cpu: &mut dyn CpuContext,
    data: &ArmInstruction,
    rd: u32,
    rn: u32,
    op2_val: u32,
) {
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



// pub struct Calculate_Adc_Imm;
// impl Executable for Calculate_Adc_Imm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
//         if !check_condition(cpu, data.condition()) {
//             return;
//         }
//         let (rd, rn, imm) = Operand2_resolver(cpu, data);
//         calculate_adc_core(cpu, data, rd, rn, imm);
//     }
// }
// pub struct Calculate_Adc_Imm_match;
// impl MatchFn for Calculate_Adc_Imm_match {
//     fn op_match(&self, data: &ArmInstruction) -> bool {
//         op2_imm_match(data)
//     }
// }

// pub struct Calculate_Adc_Reg;
// impl Executable for Calculate_Adc_Reg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
//         if !check_condition(cpu, data.condition()) {
//             return;
//         }
//         let (rd, rn, rm) = Operand2_resolver(cpu, data);
//         let val = cpu.read_reg(rm);
//         calculate_adc_core(cpu, data, rd, rn, val);
//     }
// }
// pub struct Calculate_Adc_Reg_match;
// impl MatchFn for Calculate_Adc_Reg_match {
//     fn op_match(&self, data: &ArmInstruction) -> bool {
//         op2_reg_match(data)
//     }
// }

// === SUB ===

fn calculate_sub_core(
    cpu: &mut dyn CpuContext,
    data: &ArmInstruction,
    rd: u32,
    rn: u32,
    op2_val: u32,
) {
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

// pub struct Calculate_Sub_Imm;
// impl Executable for Calculate_Sub_Imm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
//         if !check_condition(cpu, data.condition()) {
//             return;
//         }
//         let (rd, rn, imm) = Operand2_resolver(cpu, data);
//         calculate_sub_core(cpu, data, rd, rn, imm);
//     }
// }
// pub struct Calculate_Sub_Imm_match;
// impl MatchFn for Calculate_Sub_Imm_match {
//     fn op_match(&self, data: &ArmInstruction) -> bool {
//         op2_imm_match(data)
//     }
// }

// pub struct Calculate_Sub_Reg;
// impl Executable for Calculate_Sub_Reg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
//         pub struct Calculate_Sub_Reg_match;
//         impl MatchFn for Calculate_Sub_Reg_match {
//             fn op_match(&self, data: &ArmInstruction) -> bool {
//                 op2_reg_match(data)
//             }
//         }
//         if !check_condition(cpu, data.condition()) {
//             return;
//         }
//         let (rd, rn, rm) = Operand2_resolver(cpu, data);
//         let val = cpu.read_reg(rm);
//         calculate_sub_core(cpu, data, rd, rn, val);
//     }
// }

// === SBC ===

fn calculate_sbc_core(
    cpu: &mut dyn CpuContext,
    data: &ArmInstruction,
    rd: u32,
    rn: u32,
    op2_val: u32,
) {
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

// pub struct Calculate_Sbc_Imm;
// impl Executable for Calculate_Sbc_Imm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
//         if !check_condition(cpu, data.condition()) {
//             return;
//         }
//         let (rd, rn, imm) = Operand2_resolver(cpu, data);
//         calculate_sbc_core(cpu, data, rd, rn, imm);
//     }
// }
// pub struct Calculate_Sbc_Imm_match;
// impl MatchFn for Calculate_Sbc_Imm_match {
//     fn op_match(&self, data: &ArmInstruction) -> bool {
//         op2_imm_match(data)
//     }
// }

// pub struct Calculate_Sbc_Reg;
// impl Executable for Calculate_Sbc_Reg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
//         if !check_condition(cpu, data.condition()) {
//             return;
//         }
//         let (rd, rn, rm) = Operand2_resolver(cpu, data);
//         let val = cpu.read_reg(rm);
//         calculate_sbc_core(cpu, data, rd, rn, val);
//     }
// }
// pub struct Calculate_Sbc_Reg_match;
// impl MatchFn for Calculate_Sbc_Reg_match {
//     fn op_match(&self, data: &ArmInstruction) -> bool {
//         op2_reg_match(data)
//     }
// }

// === RSB ===

fn calculate_rsb_core(
    cpu: &mut dyn CpuContext,
    data: &ArmInstruction,
    rd: u32,
    rn: u32,
    op2_val: u32,
) {
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

// pub struct Calculate_Rsb_Imm;
// impl Executable for Calculate_Rsb_Imm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
//         if !check_condition(cpu, data.condition()) {
//             return;
//         }
//         let (rd, rn, imm) = Operand2_resolver(cpu, data);

//         calculate_rsb_core(cpu, data, rd, rn, imm);
//     }
// }
// pub struct Calculate_Rsb_Imm_match;
// impl MatchFn for Calculate_Rsb_Imm_match {
//     fn op_match(&self, data: &ArmInstruction) -> bool {
//         op2_imm_match(data)
//     }
// }

// pub struct Calculate_Rsb_Reg;
// impl Executable for Calculate_Rsb_Reg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmInstruction) {
//         if !check_condition(cpu, data.condition()) {
//             return;
//         }
//         let (rd, rn, rm) = Operand2_resolver(cpu, data);
//         let val = cpu.read_reg(rm);
//         calculate_rsb_core(cpu, data, rd, rn, val);
//     }
// }
// pub struct Calculate_Rsb_Reg_match;
// impl MatchFn for Calculate_Rsb_Reg_match {
//     fn op_match(&self, data: &ArmInstruction) -> bool {
//         op2_reg_match(data)
//     }
// }
