use crate::context::CpuContext;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, Operand2_resolver, OperandResolver, UpdateApsr_C, UpdateApsr_N,
    UpdateApsr_Z, check_condition,
};
use crate::opcodes::instruction::{InstrBuilder};

pub struct Mov_builder;
impl InstrBuilder for Mov_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_mov_def()
    }
}

pub fn add_mov_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_MOV as u32,
            name: "MOV".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 0,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Mov,
            operand_resolver: &OpMovResolver,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_MVN as u32,
            name: "MVN".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 0,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Mvn,
            operand_resolver: &OpMovResolver,
            adjust_cycles: None,
        },
    ]
}

// MOV{S}{cond} Rd, Operand2
// MOV{cond} Rd, #imm16
// MVN{S}{cond} Rd, Operand2

pub struct OpMovResolver;
impl OperandResolver for OpMovResolver {
    fn resolve(&self, cpu: &mut dyn crate::context::CpuContext, data: &mut ArmOpcode) -> u32 {
        let (rd, _rn, op2) = Operand2_resolver(cpu, data);
        data.transed_operands.reserve(2);
        data.transed_operands.push(rd);
        data.transed_operands.push(op2);
        op2
    }
}

pub struct Op_Mov;
impl Executable for Op_Mov {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        let rd = data.transed_operands.get(0).copied().unwrap_or(0);
        let imm = data.transed_operands.get(1).copied().unwrap_or(0);

        cpu.write_reg(rd, imm);
        // print!("mov addr:0x{:08x}\n",imm);
        if data.update_flags() {
            UpdateApsr_N(cpu, imm);
            UpdateApsr_Z(cpu, imm);
        }
        if rd == 15 { 0 } else { data.size() }
    }
}

pub struct Op_Mvn;
impl Executable for Op_Mvn {
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
        }
        if rd == 15 { 0 } else { data.size() }
    }
}
