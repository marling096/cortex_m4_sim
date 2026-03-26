use crate::arch::ArmInsn;
use crate::context::CpuContext;
use crate::opcodes::decoded::{DecodedInstructionBuilder, DecodedOperandKind};
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, OperandResolver, check_condition,
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
            insnid: ArmInsn::ARM_INS_CBZ as u32,
            name: "CBZ".to_string(),
            length: 16,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Cbz::execute,
            operand_resolver: &OpCompareBranchResolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: ArmInsn::ARM_INS_CBNZ as u32,
            name: "CBNZ".to_string(),
            length: 16,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Cbnz::execute,
            operand_resolver: &OpCompareBranchResolver,
            adjust_cycles: None,
        },
    ]
}

// CBZ Rn, label
// CBNZ Rn, label

pub struct Op_Cbz;
impl Executable for Op_Cbz {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }

        let rn = data.arm_operands.rn;
        let label = resolve_compare_branch_target(cpu, data);

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
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }

        let rn = data.arm_operands.rn;
        let label = resolve_compare_branch_target(cpu, data);

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
    fn resolve(&self, raw: &ArmOpcode, decoded: &mut DecodedInstructionBuilder) -> u32 {
        decoded.arm_operands.condition = raw.condition();
        decoded.arm_operands.rn = match decoded.get_operand(0) {
            Some(op) => match op.op_type {
                DecodedOperandKind::Reg(reg) => reg,
                _ => 0,
            },
            None => 0,
        };
        decoded.arm_operands.op2 = decoded.get_operand(1).cloned();
        decoded.arm_operands.rn
    }
}

fn resolve_compare_branch_target(cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
    match &data.arm_operands.op2 {
        Some(op) => match op.op_type {
            DecodedOperandKind::Imm(imm) => imm as u32,
            DecodedOperandKind::Reg(reg) => cpu.read_reg(reg),
            _ => 0,
        },
        None => 0,
    }
}
