use crate::context::CpuContext;
use crate::opcodes::opcode::{
    ArmOpcode, CycleInfo, Executable, Opcode, MatchFn, Operand2_resolver, UpdateApsr_C,
    UpdateApsr_N, UpdateApsr_Z, check_condition, op2_imm_match, op2_reg_match,
};
use crate::opcodes::instruction::{InstrBuilder};
use capstone::arch::arm::{ArmInsn, ArmOperandType};

pub struct Bitop_builder;
impl InstrBuilder for Bitop_builder {
    fn build(&self) -> Vec<Opcode> {
        add_bitop_def()
    }
}

pub fn add_bitop_def() -> Vec<Opcode> {
    vec![
        Opcode {
            insnid: ArmInsn::ARM_INS_AND as u32,
            name: "AND".to_string(),
            length: 16,
            cycles: CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_And,
            adjust_cycles: None,
        },
        Opcode {
            insnid: ArmInsn::ARM_INS_ORR as u32,
            name: "ORR".to_string(),
            length: 32,
            cycles: CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Orr,
            adjust_cycles: None,
        },
        Opcode {
            insnid: ArmInsn::ARM_INS_EOR as u32,
            name: "EOR".to_string(),
            length: 32,
            cycles: CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Eor,
            adjust_cycles: None,
        },
        Opcode {
            insnid: ArmInsn::ARM_INS_BIC as u32,
            name: "BIC".to_string(),
            length: 32,
            cycles: CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Bic,
            adjust_cycles: None,
        },
        Opcode {
            insnid: ArmInsn::ARM_INS_ORN as u32,
            name: "ORN".to_string(),
            length: 32,
            cycles: CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Orn,
            adjust_cycles: None,
        },
    ]
}

// AND, ORR, EOR, BIC, and ORN
// op{S}{cond} {Rd,} Rn, Operand2
// Operand2 can be a:
// • Constant
// • Register with optional shift

pub struct Op_And;
impl Executable for Op_And{
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        let (rd, rn, op2) = Operand2_resolver(cpu, data);
        let result = cpu.read_reg(rn) & op2;
        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
        }
    }
}


pub struct Op_Orr;
impl Executable for Op_Orr{
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        let (rd, rn, op2) = Operand2_resolver(cpu, data);
        let result = cpu.read_reg(rn) | op2;
        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
        }
    }
}

pub struct Op_Bic;
impl Executable for Op_Bic{
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        let (rd, rn, op2) = Operand2_resolver(cpu, data);
        let result = cpu.read_reg(rn) & !op2;
        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
        }
    }
}

pub struct Op_Orn;
impl Executable for Op_Orn{
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        let (rd, rn, op2) = Operand2_resolver(cpu, data);
        let result = cpu.read_reg(rn) | !op2;
        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
        }
    }
}

pub struct Op_Eor;
impl Executable for Op_Eor{
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        let (rd, rn, op2) = Operand2_resolver(cpu, data);
        let result = cpu.read_reg(rn) ^ op2;
        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
        }
    }
}



// pub struct Bit_And_Imm;

// impl Executable for Bit_And_Imm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         bit_and_imm(cpu, data);
//     }
// }
// fn bit_and_imm(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rn, imm) = Operand2_resolver(cpu, data);

//     let result = cpu.read_reg(rn) & imm;
//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//     }
// }

// pub struct Bit_And_Imm_match;

// impl MatchFn for Bit_And_Imm_match {
//     fn op_match(&self, data: &ArmOpcode) -> bool {
//         op2_imm_match(data)
//     }
// }

// pub struct Bit_And_Reg;

// impl Executable for Bit_And_Reg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         bit_and_reg(cpu, data);
//     }
// }

// fn bit_and_reg(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rn, rm) = Operand2_resolver(cpu, data);

//     let result = cpu.read_reg(rn) & cpu.read_reg(rm);
//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//     }
// }

// pub struct Bit_And_Reg_match;

// impl MatchFn for Bit_And_Reg_match {
//     fn op_match(&self, data: &ArmOpcode) -> bool {
//         op2_reg_match(data)
//     }
// }

// pub struct Bit_Eor_Imm;

// impl Executable for Bit_Eor_Imm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         bit_eor_imm(cpu, data);
//     }
// }

// fn bit_eor_imm(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rn, imm) = Operand2_resolver(cpu, data);

//     let result = cpu.read_reg(rn) ^ imm;
//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//     }
// }

// pub struct Bit_Eor_Imm_match;

// impl MatchFn for Bit_Eor_Imm_match {
//     fn op_match(&self, data: &ArmOpcode) -> bool {
//         op2_imm_match(data)
//     }
// }

// pub struct Bit_Eor_Reg;

// impl Executable for Bit_Eor_Reg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         bit_eor_reg(cpu, data);
//     }
// }

// fn bit_eor_reg(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rn, rm) = Operand2_resolver(cpu, data);

//     let result = cpu.read_reg(rn) ^ cpu.read_reg(rm);
//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//     }
// }

// pub struct Bit_Eor_Reg_match;

// impl MatchFn for Bit_Eor_Reg_match {
//     fn op_match(&self, data: &ArmOpcode) -> bool {
//         op2_reg_match(data)
//     }
// }

// pub struct Bit_Orr_Imm;

// impl Executable for Bit_Orr_Imm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         bit_orr_imm(cpu, data);
//     }
// }

// fn bit_orr_imm(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rn, imm) = Operand2_resolver(cpu, data);

//     let result = cpu.read_reg(rn) | imm;
//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//     }
// }

// pub struct Bit_Orr_Imm_match;

// impl MatchFn for Bit_Orr_Imm_match {
//     fn op_match(&self, data: &ArmOpcode) -> bool {
//         op2_imm_match(data)
//     }
// }

// pub struct Bit_Orr_Reg;

// impl Executable for Bit_Orr_Reg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         bit_orr_reg(cpu, data);
//     }
// }

// fn bit_orr_reg(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rn, rm) = Operand2_resolver(cpu, data);

//     let result = cpu.read_reg(rn) | cpu.read_reg(rm);
//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//     }
// }

// pub struct Bit_Orr_Reg_match;

// impl MatchFn for Bit_Orr_Reg_match {
//     fn op_match(&self, data: &ArmOpcode) -> bool {
//         op2_reg_match(data)
//     }
// }

// pub struct Bit_Bic_Imm;

// impl Executable for Bit_Bic_Imm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         bit_bic_imm(cpu, data);
//     }
// }

// fn bit_bic_imm(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rn, imm) = Operand2_resolver(cpu, data);

//     let result = cpu.read_reg(rn) & !imm;
//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//     }
// }

// pub struct Bit_Bic_Imm_match;

// impl MatchFn for Bit_Bic_Imm_match {
//     fn op_match(&self, data: &ArmOpcode) -> bool {
//         op2_imm_match(data)
//     }
// }

// pub struct Bit_Bic_Reg;

// impl Executable for Bit_Bic_Reg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         bit_bic_reg(cpu, data);
//     }
// }

// fn bit_bic_reg(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rn, rm) = Operand2_resolver(cpu, data);

//     let result = cpu.read_reg(rn) & !cpu.read_reg(rm);
//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//     }
// }

// pub struct Bit_Bic_Reg_match;

// impl MatchFn for Bit_Bic_Reg_match {
//     fn op_match(&self, data: &ArmOpcode) -> bool {
//         op2_reg_match(data)
//     }
// }

// pub struct Bit_Orn_Imm;

// impl Executable for Bit_Orn_Imm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         bit_orn_imm(cpu, data);
//     }
// }

// fn bit_orn_imm(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rn, imm) = Operand2_resolver(cpu, data);

//     let result = cpu.read_reg(rn) | !imm;
//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//     }
// }

// pub struct Bit_Orn_Imm_match;

// impl MatchFn for Bit_Orn_Imm_match {
//     fn op_match(&self, data: &ArmOpcode) -> bool {
//         op2_imm_match(data)
//     }
// }

// pub struct Bit_Orn_Reg;

// impl Executable for Bit_Orn_Reg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         bit_orn_reg(cpu, data);
//     }
// }

// fn bit_orn_reg(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rn, rm) = Operand2_resolver(cpu, data);

//     let result = cpu.read_reg(rn) | !cpu.read_reg(rm);
//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//     }
// }

// pub struct Bit_Orn_Reg_match;

// impl MatchFn for Bit_Orn_Reg_match {
//     fn op_match(&self, data: &ArmOpcode) -> bool {
//         op2_reg_match(data)
//     }
// }
