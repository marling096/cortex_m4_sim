use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, CycleInfo, Executable, Opcode, OperandResolver, check_condition,
};
use capstone::arch::arm::{ArmInsn, ArmOperandType};

pub struct Branch_builder;
impl InstrBuilder for Branch_builder {
    fn build(&self) -> Vec<Opcode> {
        add_branch_def()
    }
}

pub fn add_branch_def() -> Vec<Opcode> {
    vec![
        Opcode {
            insnid: ArmInsn::ARM_INS_B as u32,
            name: "B".to_string(),
            length: 32,
            cycles: CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_B::execute,
            operand_resolver: &OpBranchResolver,
            adjust_cycles: None,
        },
        Opcode {
            insnid: ArmInsn::ARM_INS_BL as u32,
            name: "BL".to_string(),
            length: 32,
            cycles: CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Bl::execute,
            operand_resolver: &OpBranchResolver,
            adjust_cycles: None,
        },
        Opcode {
            insnid: ArmInsn::ARM_INS_BX as u32,
            name: "BX".to_string(),
            length: 32,
            cycles: CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Bx::execute,
            operand_resolver: &OpBxResolver,
            adjust_cycles: None,
        },
        Opcode {
            insnid: ArmInsn::ARM_INS_BLX as u32,
            name: "BLX".to_string(),
            length: 32,
            cycles: CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Blx::execute,
            operand_resolver: &OpBxResolver,
            adjust_cycles: None,
        },
    ]
}

// B{cond} label
// BL{cond} label
// BX{cond} Rm
// BLX{cond} Rm

pub struct Op_B;
impl Executable for Op_B {
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        let label = resolve_branch_target(cpu, data);
        cpu.write_pc(label);
        0
    }
}

pub struct Op_Bl;
impl Executable for Op_Bl {
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        let label = resolve_branch_target(cpu, data);
        let pc = cpu.read_pc();
        let return_addr = pc;
        cpu.write_lr(return_addr | 1);
        cpu.write_pc(label);
        0
    }
}

pub struct Op_Bx;
impl Executable for Op_Bx {
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        let val = resolve_branch_target(cpu, data);
        if cpu.try_exception_return(val) {
            return 0;
        }
        let target = val & !1;
        cpu.write_pc(target);
        0
    }
}

pub struct Op_Blx;
impl Executable for Op_Blx {
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        let val = resolve_branch_target(cpu, data);
        let pc = cpu.read_pc();
        let insn_len = data.insn.len() as u32;
        let return_addr = pc.wrapping_sub_signed(4).wrapping_add(insn_len);
        cpu.write_lr(return_addr | 1);

        let target = val & !1;
        cpu.write_pc(target);
        0
    }
}

pub struct OpBranchResolver;
impl OperandResolver for OpBranchResolver {
    fn resolve(&self, data: &mut ArmOpcode) -> u32 {
        data.arm_operands.op2 = data.get_operand(0);
        0
    }
}

pub struct OpBxResolver;
impl OperandResolver for OpBxResolver {
    fn resolve(&self, data: &mut ArmOpcode) -> u32 {
        data.arm_operands.op2 = data.get_operand(0);
        0
    }
}

fn resolve_branch_target(cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
    match &data.arm_operands.op2 {
        Some(op) => match op.op_type {
            ArmOperandType::Imm(imm) => imm as u32,
            ArmOperandType::Reg(reg) => cpu.read_reg(data.resolve_reg(reg)),
            _ => 0,
        },
        None => 0,
    }
}
