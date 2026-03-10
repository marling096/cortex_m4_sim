use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, OperandResolver, UpdateApsr_C, UpdateApsr_N,
    UpdateApsr_Z, check_condition, resolve_op2_runtime,
};
use capstone::arch::arm::ArmOperandType;

// TST{cond} Rn, Operand2
// TEQ{cond} Rn, Operand2

pub struct Op_Tst;
impl Executable for Op_Tst {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }

        let rn = data.arm_operands.rn;
        let (op2, carry) = resolve_op2_runtime(cpu, data);
        let rn_data = cpu.read_reg(rn);
        let result = rn_data & op2;

        UpdateApsr_N(cpu, result);
        UpdateApsr_Z(cpu, result);
        UpdateApsr_C(cpu, carry);

        data.size()
    }
}

pub struct Op_Teq;
impl Executable for Op_Teq {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.arm_operands.condition) {
            return data.size();
        }

        let rn = data.arm_operands.rn;
        let (op2, carry) = resolve_op2_runtime(cpu, data);
        let rn_data = cpu.read_reg(rn);
        let result = rn_data ^ op2;

        UpdateApsr_N(cpu, result);
        UpdateApsr_Z(cpu, result);
        UpdateApsr_C(cpu, carry);

        data.size()
    }
}

pub struct OpTst_resolver;
impl OperandResolver for OpTst_resolver {
    fn resolve(&self, data: &mut ArmOpcode) -> u32 {

        let rn = match data.get_operand(0) {
            Some(op) => match op.op_type {
                ArmOperandType::Reg(r) => data.resolve_reg(r),
                _ => 0,
            },
            None => 0,
        };
        data.arm_operands.condition = data.condition();
        data.arm_operands.rn = rn;
        data.arm_operands.op2 = data.get_operand(1);
        rn
    }
}

pub struct Tst_builder;
impl InstrBuilder for Tst_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_tst_def()
    }
}
pub fn add_tst_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_TST as u32,
            name: "TST".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Tst::execute,
            operand_resolver: &OpTst_resolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_TEQ as u32,
            name: "TEQ".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Teq::execute,
            operand_resolver: &OpTst_resolver,
            adjust_cycles: None,
        },
    ]
}
