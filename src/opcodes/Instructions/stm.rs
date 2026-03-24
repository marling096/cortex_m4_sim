use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, CycleInfo, Executable, OperandResolver, check_condition, count_reg_operands,
    resolve_multi_reg_operands,
};
use capstone::arch::arm::ArmOperand;

// op{addr_mode}{cond} Rn{!}, reglist
pub struct Op_Stm;
impl Executable for Op_Stm {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        stm(cpu, data)
    }
}

pub fn stm(cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
    if !check_condition(cpu, data.arm_operands.condition) {
        return data.size();
    }
    if data.transed_operands.is_empty() {
        return data.size();
    }

    let base_reg_id = data.transed_operands[0];
    let reg_list_ids = &data.transed_operands[1..];

    let base_addr = cpu.read_reg(base_reg_id);
    let mut addr = base_addr;
    for &reg_id in reg_list_ids {
        let value = cpu.read_reg(reg_id);
        cpu.write_mem(addr, value);
        addr = addr.wrapping_add(4);
    }

    if data.writeback() {
        cpu.write_reg(base_reg_id, addr);
    }

    data.size()
}

pub struct Stm_builder;
impl InstrBuilder for Stm_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_stm_def()
    }
}
pub fn add_stm_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![crate::opcodes::opcode::Opcode {
        insnid: capstone::arch::arm::ArmInsn::ARM_INS_STM as u32,
        name: "STM".to_string(),
        length: 32,
        cycles: crate::opcodes::opcode::CycleInfo {
            fetch_cycles: 1,
            decode_cycles: 0,
            execute_cycles: 1,
        },
        exec: Op_Stm::execute,
        operand_resolver: &OpStm_resolver,
        adjust_cycles: Some(adjust_stm_cycles),
    }]
}

fn adjust_stm_cycles(cycles: &mut CycleInfo, operands: &[ArmOperand]) {
    let reg_count = count_reg_operands(operands).saturating_sub(1);
    cycles.execute_cycles = 1u32.saturating_add(reg_count);
}

pub struct OpStm_resolver;
impl OperandResolver for OpStm_resolver {
    fn resolve(&self, data: &mut ArmOpcode) -> u32 {
        resolve_multi_reg_operands(data, true)
    }
}
