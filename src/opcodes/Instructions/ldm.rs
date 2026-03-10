use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, CycleInfo, Executable, OperandResolver, check_condition, count_reg_operands,
    reg_list_contains, resolve_multi_reg_operands,
};
use capstone::arch::arm::{ArmOperand, ArmReg};

// op{addr_mode}{cond} Rn{!}, reglist
pub struct Op_Ldm;
impl Executable for Op_Ldm {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        ldm(cpu, data)
    }
}
pub fn ldm(cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
    if !check_condition(cpu, data.arm_operands.condition) {
        return data.size();
    }

    if data.transed_operands.is_empty() {
        return data.size();
    }

    let base_reg_id = data.transed_operands[0];
    let reg_list_id = &data.transed_operands[1..];

    let base_addr = cpu.read_reg(base_reg_id);

    let mut addr = base_addr;
    for &reg_id in reg_list_id {
        let value = cpu.read_mem(addr);
        cpu.write_reg(reg_id, value);
        addr = addr.wrapping_add(4);
    }

    if data.writeback() {
        cpu.write_reg(base_reg_id, addr);
    }

    if reg_list_id.iter().any(|&r| r == 15) {
        0
    } else {
        data.size()
    }
}

fn adjust_ldm_cycles(cycles: &mut CycleInfo, operands: &[ArmOperand]) {
    let reg_count = count_reg_operands(operands).saturating_sub(1);
    let mut execute = 1u32.saturating_add(reg_count);
    if reg_list_contains(operands, ArmReg::ARM_REG_PC as u16, true) {
        execute = execute.saturating_add(1);
    }
    cycles.execute_cycles = execute;
}

pub struct OpLdm_resolver;
impl OperandResolver for OpLdm_resolver {
    fn resolve(&self, data: &mut ArmOpcode) -> u32 {
        resolve_multi_reg_operands(data, true)
    }
}

pub struct Ldm_builder;
impl InstrBuilder for Ldm_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_ldm_def()
    }
}
pub fn add_ldm_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![crate::opcodes::opcode::Opcode {
        insnid: capstone::arch::arm::ArmInsn::ARM_INS_LDM as u32,
        name: "LDM".to_string(),
        length: 32,
        cycles: crate::opcodes::opcode::CycleInfo {
            fetch_cycles: 1,
            decode_cycles: 0,
            execute_cycles: 1,
        },
        exec: Op_Ldm::execute,
        operand_resolver: &OpLdm_resolver,
        adjust_cycles: Some(adjust_ldm_cycles),
    }]
}
