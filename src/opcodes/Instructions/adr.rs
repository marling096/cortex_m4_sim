use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, OperandResolver, check_condition,
};
use capstone::arch::arm::ArmOperandType;

// ADR{cond} Rd, label
pub struct OpAdr;
impl Executable for OpAdr {
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        let rd = data.arm_operands.rd;
        let address = resolve_adr_target(cpu, data);

        cpu.write_reg(rd, address);

        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct OpAdrResolver;
impl OperandResolver for OpAdrResolver {
    fn resolve(&self, data: &mut ArmOpcode) -> u32 {
        let rd = match data.get_operand(0) {
            Some(op) => match op.op_type {
                ArmOperandType::Reg(r) => data.resolve_reg(r),
                _ => 0,
            },
            None => 0,
        };
        data.arm_operands.rd = rd;
        data.arm_operands.rn = 0;
        data.arm_operands.op2 = data.get_operand(1);

        rd
    }
}

fn resolve_adr_target(cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
    match &data.arm_operands.op2 {
        Some(op2) => match op2.op_type {
            ArmOperandType::Imm(imm) => imm as u32,
            ArmOperandType::Reg(r) => cpu.read_reg(data.resolve_reg(r)),
            _ => 0,
        },
        None => 0,
    }
}

pub struct AdrBuilder;
impl InstrBuilder for AdrBuilder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_adr_def()
    }
}

pub fn add_adr_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![crate::opcodes::opcode::Opcode {
        insnid: capstone::arch::arm::ArmInsn::ARM_INS_ADR as u32,
        name: "ADR".to_string(),
        length: 32,
        cycles: crate::opcodes::opcode::CycleInfo {
            fetch_cycles: 1,
            decode_cycles: 0,
            execute_cycles: 1,
        },
        exec: OpAdr::execute,
        operand_resolver: &OpAdrResolver,
        adjust_cycles: None,
    }]
}
