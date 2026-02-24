use crate::context::CpuContext;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, OperandResolver, UpdateApsr_C, UpdateApsr_N,
    UpdateApsr_Z, check_condition,
};
use crate::opcodes::instruction::{InstrBuilder};
use capstone::arch::arm::{ArmOperandType, ArmShift};

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

pub struct Op_Mov;
impl Executable for Op_Mov {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        let rd = data.arm_operands.rd;
        let (imm, carry) = resolve_op2_and_carry(cpu, data);

        cpu.write_reg(rd, imm);
        // print!("mov addr:0x{:08x}\n",imm);
        if data.update_flags() {
            UpdateApsr_N(cpu, imm);
            UpdateApsr_Z(cpu, imm);
            UpdateApsr_C(cpu, carry);
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

fn resolve_op2_and_carry(cpu: &mut dyn CpuContext, data: &ArmOpcode) -> (u32, u8) {
    let current_c = ((cpu.read_apsr() >> 29) & 1) as u8;
    let Some(op2) = &data.arm_operands.op2 else {
        return (0, current_c);
    };

    match op2.op_type {
        ArmOperandType::Reg(reg) => {
            let value = cpu.read_reg(data.resolve_reg(reg));
            op_shift_match(op2.shift, value, current_c)
        }
        ArmOperandType::Imm(imm) => (imm as u32, current_c),
        _ => (0, current_c),
    }
}

fn op_shift_match(shift: ArmShift, value: u32, current_c: u8) -> (u32, u8) {
    match shift {
        ArmShift::Lsl(amount) => match amount {
            0 => (value, current_c),
            1..=31 => {
                let carry = ((value >> (32 - amount)) & 1) as u8;
                (value.wrapping_shl(amount), carry)
            }
            32 => (0, (value & 1) as u8),
            _ => (0, 0),
        },
        ArmShift::Lsr(amount) => match amount {
            0 => (value, current_c),
            1..=31 => {
                let carry = ((value >> (amount - 1)) & 1) as u8;
                (value >> amount, carry)
            }
            32 => (0, (value >> 31) as u8),
            _ => (0, 0),
        },
        ArmShift::Asr(amount) => match amount {
            0 => (value, current_c),
            1..=31 => {
                let carry = ((value >> (amount - 1)) & 1) as u8;
                (((value as i32) >> amount) as u32, carry)
            }
            _ => {
                let carry = ((value >> 31) & 1) as u8;
                let result = if (value as i32) < 0 { 0xFFFF_FFFF } else { 0 };
                (result, carry)
            }
        },
        ArmShift::Ror(amount) => {
            if amount == 0 {
                (value, current_c)
            } else {
                let shift_mod = amount % 32;
                if shift_mod == 0 {
                    (value, (value >> 31) as u8)
                } else {
                    let result = value.rotate_right(shift_mod);
                    (result, ((result >> 31) & 1) as u8)
                }
            }
        }
        ArmShift::Rrx(_) => {
            let carry = (value & 1) as u8;
            let result = (value >> 1) | ((current_c as u32) << 31);
            (result, carry)
        }
        _ => (value, current_c),
    }
}
