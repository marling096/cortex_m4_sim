use crate::context::CpuContext;
use crate::opcodes::opcode::{ArmOpcode, Executable, Operand_resolver_two, check_condition};
use crate::opcodes::instruction::{InstrBuilder};

pub struct Compare_branch_builder;
impl InstrBuilder for Compare_branch_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_compare_branch_def()
    }
}

pub fn add_compare_branch_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_CBZ as u32,
            name: "CBZ".to_string(),
            length: 16,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Cbz,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_CBNZ as u32,
            name: "CBNZ".to_string(),
            length: 16,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Cbnz,
            adjust_cycles: None,
        },
    ]
}

// CBZ Rn, label
// CBNZ Rn, label

pub struct Op_Cbz;
impl Executable for Op_Cbz {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        if !check_condition(cpu, data.condition()) {
            return;
        }

        // CBZ Rn, label
        let (rn, label) = Operand_resolver_two(cpu, data);

        let val = cpu.read_reg(rn);
        if val == 0 {
            cpu.write_pc(label);
        }
    }
}

pub struct Op_Cbnz;
impl Executable for Op_Cbnz {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        if !check_condition(cpu, data.condition()) {
            return;
        }

        // CBNZ Rn, label
        let (rn, label) = Operand_resolver_two(cpu, data);

        let val = cpu.read_reg(rn);
        if val != 0 {
            cpu.write_pc(label);
        }
    }
}

// pub struct Compare_Branch_Zero;

// impl Executable for Compare_Branch_Zero {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         compare_branch_zero(cpu, data);
//     }
// }

// fn compare_branch_zero(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }

//     let rn = data.operands[0];
//     let label = data.operands[1];

//     let val = cpu.read_reg(rn);
//     if val == 0 {
//         cpu.write_pc(label);
//     }
// }

// pub struct Compare_Branch_NotZero;

// impl Executable for Compare_Branch_NotZero {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         compare_branch_not_zero(cpu, data);
//     }
// }

// fn compare_branch_not_zero(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }

//     let rn = data.operands[0];
//     let label = data.operands[1];

//     let val = cpu.read_reg(rn);
//     if val != 0 {
//         cpu.write_pc(label);
//     }
// }
