use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, Operand2_resolver, OperandResolver, UpdateApsr_C, UpdateApsr_N,
    UpdateApsr_Z, check_condition,
};

// MOV{S}{cond} Rd, Operand2
// MOV{cond} Rd, #imm16
// MVN{S}{cond} Rd, Operand2
pub struct Op_Movs;
impl Executable for Op_Movs {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        let rd = data.transed_operands.get(0).copied().unwrap_or(0);
        let imm = data.transed_operands.get(1).copied().unwrap_or(0);

        cpu.write_reg(rd, imm);

        if data.update_flags() {
            UpdateApsr_N(cpu, imm);
            UpdateApsr_Z(cpu, imm);
            UpdateApsr_C(cpu, data.update_carry as u8);
            // Note: C flag update logic should be added here based on Operand2 specifics
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Mvns;
impl Executable for Op_Mvns {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        let rd = data.transed_operands.get(0).copied().unwrap_or(0);
        let val = data.transed_operands.get(1).copied().unwrap_or(0);
        let result = !val;

        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_N(cpu, result);
            UpdateApsr_Z(cpu, result);
            UpdateApsr_C(cpu, data.update_carry as u8);
            // Note: C flag update logic should be added here based on Operand2 specifics
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

fn get_ops(cpu: &mut dyn crate::context::CpuContext, data: &mut ArmOpcode) -> (u32, u32) {
    let (rn, rd, op2) = Operand2_resolver(cpu, data);
    (rn, op2)
}

pub struct OpMovsResolver;
impl OperandResolver for OpMovsResolver {
    fn resolve(&self, cpu: &mut dyn crate::context::CpuContext, data: &mut ArmOpcode) -> u32 {
        let (rd, _rn, op2) = Operand2_resolver(cpu, data);
        data.transed_operands.reserve(2);
        data.transed_operands.push(rd);
        data.transed_operands.push(op2);
        op2
    }
}

pub struct Movs_builder;
impl InstrBuilder for Movs_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_movs_def()
    }
}
pub fn add_movs_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![crate::opcodes::opcode::Opcode {
        insnid: capstone::arch::arm::ArmInsn::ARM_INS_MOVS as u32,
        name: "MOVS".to_string(),
        length: 32,
        cycles: crate::opcodes::opcode::CycleInfo {
            fetch_cycles: 1,
            decode_cycles: 0,
            execute_cycles: 1,
        },
        exec: &Op_Movs,
        operand_resolver: &OpMovsResolver,
        adjust_cycles: None,
    }]
}
