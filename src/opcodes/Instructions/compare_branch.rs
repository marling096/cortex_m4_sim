use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, Operand_resolver_two, OperandResolver, check_condition,
};

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
            operand_resolver: &OpCompareBranchResolver,
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
            operand_resolver: &OpCompareBranchResolver,
            adjust_cycles: None,
        },
    ]
}

// CBZ Rn, label
// CBNZ Rn, label

pub struct Op_Cbz;
impl Executable for Op_Cbz {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        // CBZ Rn, label
        let rn = data.transed_operands.get(0).copied().unwrap_or(0);
        let label = data.transed_operands.get(1).copied().unwrap_or(0);

        let val = cpu.read_reg(rn);
        // print!("CBZ R{}:0x{:08X}, label 0x{:08X}\n", rn, val, label);
        if val == 0 {
            cpu.write_pc(label);
            0
        } else {
            data.size()
        }
    }
}

pub struct Op_Cbnz;
impl Executable for Op_Cbnz {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        // CBNZ Rn, label
        let rn = data.transed_operands.get(0).copied().unwrap_or(0);
        let label = data.transed_operands.get(1).copied().unwrap_or(0);

        let val = cpu.read_reg(rn);
        if val != 0 {
            cpu.write_pc(label);
            0
        } else {
            data.size()
        }
    }
}

pub struct OpCompareBranchResolver;
impl OperandResolver for OpCompareBranchResolver {
    fn resolve(&self, cpu: &mut dyn crate::context::CpuContext, data: &mut ArmOpcode) -> u32 {
        let (rn, label) = Operand_resolver_two(cpu, data);
        data.transed_operands.reserve(2);
        data.transed_operands.push(rn);
        data.transed_operands.push(label);
        label
    }
}
