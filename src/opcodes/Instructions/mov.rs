use crate::context::CpuContext;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, Operand2_resolver, UpdateApsr_C, UpdateApsr_N, UpdateApsr_Z,
    check_condition,
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
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Mov,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_MVN as u32,
            name: "MVN".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Mvn,
            adjust_cycles: None,
        },
    ]
}

// MOV{S}{cond} Rd, Operand2
// MOV{cond} Rd, #imm16
// MVN{S}{cond} Rd, Operand2

fn get_ops(cpu: &mut dyn crate::context::CpuContext, data: &ArmOpcode) -> (u32, u32) {
    let (rn, rd, op2) = Operand2_resolver(cpu, data);
    (rn, op2)
}

pub struct Op_Mov;
impl Executable for Op_Mov {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        let (rd, imm) = get_ops(cpu, data);

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

        let (rd, val) = get_ops(cpu, data);
        let result = !val;

        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_N(cpu, result);
            UpdateApsr_Z(cpu, result);
        }
        if rd == 15 { 0 } else { data.size() }
    }
}
