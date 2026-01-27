use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, MatchFn, Operand2_resolver, UpdateApsr_C, UpdateApsr_N,
    UpdateApsr_V, UpdateApsr_Z, check_condition, op2_imm_match, op2_reg_match,
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
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_CMP as u32,
            name: "CMP".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Cmp,
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
            exec: &Op_Cmn,
            adjust_cycles: None,
        },
    ]
}

// CMP{cond} Rn, Operand2
// CMN{cond} Rn, Operand2

pub struct Op_Cmp;
impl Executable for Op_Cmp {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rn, op2) = get_ops(cpu, data);
        cmp_core(cpu, data, rn, op2);
    }
}

pub struct Op_Cmn;
impl Executable for Op_Cmn {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rn, op2) = get_ops(cpu, data);
        cmn_core(cpu, data, rn, op2);
    }
}

fn get_ops(cpu: &mut dyn crate::context::CpuContext, data: &ArmOpcode) -> (u32, u32) {
    let (rn, rd, op2) = Operand2_resolver(cpu, data);
    (rn, op2)
}

// === CMP ===
// CMP is effectively a SUBS Opcode with the result discarded.
fn cmp_core(cpu: &mut dyn CpuContext, _data: &ArmOpcode, rn: u32, op2_val: u32) {
    let rn_val = cpu.read_reg(rn);
    // Rn - Op2
    let result = rn_val.wrapping_sub(op2_val);

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

// pub struct Cmp_Imm;
// impl Executable for Cmp_Imm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         if !check_condition(cpu, data.condition()) {
//             return;
//         }
//         let (rn, imm) = get_ops(cpu, data);
//         cmp_core(cpu, data, rn, imm);
//     }
// }
// pub struct Cmp_Imm_match;
// impl MatchFn for Cmp_Imm_match {
//     fn op_match(&self, data: &ArmOpcode) -> bool {
//         op2_imm_match(data)
//     }
// }

// pub struct Cmp_Reg;
// impl Executable for Cmp_Reg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         if !check_condition(cpu, data.condition()) {
//             return;
//         }
//         let (rn, rm) = get_ops(cpu, data);
//         let val = cpu.read_reg(rm);
//         cmp_core(cpu, data, rn, val);
//     }
// }
// pub struct Cmp_Reg_match;
// impl MatchFn for Cmp_Reg_match {
//     fn op_match(&self, data: &ArmOpcode) -> bool {
//         op2_reg_match(data)
//     }
// }

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

// pub struct Cmn_Imm;
// impl Executable for Cmn_Imm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         if !check_condition(cpu, data.condition()) {
//             return;
//         }
//         let (rn, imm) = get_ops(cpu, data);
//         cmn_core(cpu, data, rn, imm);
//     }
// }
// pub struct Cmn_Imm_match;
// impl MatchFn for Cmn_Imm_match {
//     fn op_match(&self, data: &ArmOpcode) -> bool {
//         op2_imm_match(data)
//     }
// }

// pub struct Cmn_Reg;
// impl Executable for Cmn_Reg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         if !check_condition(cpu, data.condition()) {
//             return;
//         }
//         let (rn, rm) = get_ops(cpu, data);
//         let val = cpu.read_reg(rm);
//         cmn_core(cpu, data, rn, val);
//     }
// }
// pub struct Cmn_Reg_match;
// impl MatchFn for Cmn_Reg_match {
//     fn op_match(&self, data: &ArmOpcode) -> bool {
//         op2_reg_match(data)
//     }
// }
