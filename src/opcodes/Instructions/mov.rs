use crate::context::CpuContext;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, Operand2_resolver, UpdateApsr_C, UpdateApsr_N, UpdateApsr_Z,
    check_condition,
};
use crate::opcodes::instruction::{InstrBuilder};

pub struct Mov_builder;
impl InstrBuilder for Mov_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_mov_def()
    }
}

pub fn add_mov_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_MOV as u32,
            name: "MOV".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Mov,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_MVN as u32,
            name: "MVN".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Mvn,
            adjust_cycles: None,
        },
    ]
}

// MOV{S}{cond} Rd, Operand2
// MOV{cond} Rd, #imm16
// MVN{S}{cond} Rd, Operand2

fn get_ops(cpu: &mut dyn crate::context::CpuContext, data: &ArmOpcode) -> (u32, u32) {
    let (rn, rd, op2) = Operand2_resolver(cpu, data);
    (rn, op2)
}

pub struct Op_Mov;
impl Executable for Op_Mov {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        if !check_condition(cpu, data.condition()) {
            return;
        }

        let (rd, imm) = get_ops(cpu, data);

        cpu.write_reg(rd, imm);

        if data.update_flags() {
            UpdateApsr_N(cpu, imm);
            UpdateApsr_Z(cpu, imm);
        }
    }
}

pub struct Op_Mvn;
impl Executable for Op_Mvn {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        if !check_condition(cpu, data.condition()) {
            return;
        }

        let (rd, val) = get_ops(cpu, data);
        let result = !val;

        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_N(cpu, result);
            UpdateApsr_Z(cpu, result);
        }
    }
}

// pub struct Mov_Imm;

// impl Executable for Mov_Imm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         mov_imm(cpu, data);
//     }
// }

// fn mov_imm(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }

//     let (rd, imm) = get_ops(cpu, data);

//     cpu.write_reg(rd, imm);

//     if data.update_flags() {
//         UpdateApsr_N(cpu, imm);
//         UpdateApsr_Z(cpu, imm);
//     }
// }

// pub struct Mov_Reg;

// impl Executable for Mov_Reg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         mov_reg(cpu, data);
//     }
// }
// fn mov_reg(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }

//     let (rd, val) = get_ops(cpu, data);

//     cpu.write_reg(rd, val);

//     if data.update_flags() {
//         UpdateApsr_N(cpu, val);
//         UpdateApsr_Z(cpu, val);
//     }
// }

// pub struct Mvn_Reg;

// impl Executable for Mvn_Reg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         mvn_reg(cpu, data);
//     }
// }

// fn mvn_reg(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }

//     let (rd, val) = get_ops(cpu, data);
//     let result = !val;

//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_N(cpu, result);
//         UpdateApsr_Z(cpu, result);
//     }
// }
