use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, Operand_resolver_two, OperandResolver, check_condition,
};

// ADR{cond} Rd, label
pub struct OpAdr;
impl Executable for OpAdr {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        // Use safe access to avoid panics if resolver failed to populate operands
        let rd = data.transed_operands.get(0).copied().unwrap_or(0);
        let address = data.transed_operands.get(1).copied().unwrap_or(0);

        cpu.write_reg(rd, address);

        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct OpAdrResolver;
impl OperandResolver for OpAdrResolver {
    fn resolve(&self, data: &mut ArmOpcode) -> u32 {
        let (rd, label) = Operand_resolver_two(cpu, data);

        data.transed_operands.reserve(2);
        data.transed_operands.push(rd);
        data.transed_operands.push(label);

        label
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
        exec: &OpAdr,
        operand_resolver: &OpAdrResolver,
        adjust_cycles: None,
    }]
}
