use crate::context::CpuContext;
use crate::opcodes::decoded::{DecodedInstructionBuilder, DecodedOperandKind};
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, OperandResolver, UpdateApsr_C, UpdateApsr_N,
    UpdateApsr_Z, check_condition, resolve_op2_runtime,
};

// MOV{S}{cond} Rd, Operand2
// MOV{cond} Rd, #imm16
// MVN{S}{cond} Rd, Operand2
pub struct Op_Movs;
impl Executable for Op_Movs {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }

        let rd = data.arm_operands.rd;
        let (imm, carry) = resolve_op2_and_carry(cpu, data);

        cpu.write_reg(rd, imm);

        if data.update_flags() {
            UpdateApsr_N(cpu, imm);
            UpdateApsr_Z(cpu, imm);
            UpdateApsr_C(cpu, carry);
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Mvns;
impl Executable for Op_Mvns {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }

        let rd = data.arm_operands.rd;
        let (val, carry) = resolve_op2_and_carry(cpu, data);
        let result = !val;

        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_N(cpu, result);
            UpdateApsr_Z(cpu, result);
            UpdateApsr_C(cpu, carry);
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct OpMovsResolver;
impl OperandResolver for OpMovsResolver {
    fn resolve(&self, raw: &ArmOpcode, decoded: &mut DecodedInstructionBuilder) -> u32 {
        let rd = match decoded.get_operand(0) {
            Some(op) => match op.op_type {
                DecodedOperandKind::Reg(reg) => reg,
                _ => 0,
            },
            None => 0,
        };

        decoded.arm_operands.condition = raw.condition();
        decoded.arm_operands.rd = rd;
        decoded.arm_operands.rn = 0;
        decoded.arm_operands.op2 = decoded.get_operand(1).cloned();
        rd
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
        insnid: crate::arch::ArmInsn::ARM_INS_MOVS as u32,
        name: "MOVS".to_string(),
        length: 32,
        cycles: crate::opcodes::opcode::CycleInfo {
            fetch_cycles: 1,
            decode_cycles: 0,
            execute_cycles: 1,
        },
        exec: Op_Movs::execute,
        operand_resolver: &OpMovsResolver,
        adjust_cycles: None,
    }]
}

fn resolve_op2_and_carry(cpu: &mut dyn CpuContext, data: &ArmOpcode) -> (u32, u8) {
    resolve_op2_runtime(cpu, data)
}
