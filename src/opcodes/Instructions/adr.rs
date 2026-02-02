use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, CycleInfo, Executable, MatchFn, Opcode, Operand_resolver_two, Operand2_resolver,
    UpdateApsr_C, UpdateApsr_N, UpdateApsr_Z, check_condition, op2_imm_match, op2_reg_match,
};
use capstone::arch::arm::{ArmInsn, ArmOperandType};

//ADR{cond} Rd, label
pub struct Op_Adr;
impl Executable for Op_Adr {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        adr(cpu, data);
        data.size()
    }
}

pub fn adr(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
    if !check_condition(cpu, data.condition()) {
        return;
    }
    let (rd, label) = Operand_resolver_two(cpu, data);
    let pc_val = cpu.read_pc();
    let address = pc_val.wrapping_add(label);
    cpu.write_reg(rd, address);
}

pub struct Adr_builder;
impl InstrBuilder for Adr_builder {
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
        exec: &Op_Adr,
        adjust_cycles: None,
    }]
}
