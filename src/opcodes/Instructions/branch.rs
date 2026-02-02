use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, CycleInfo, Executable, Opcode, Operand_resolver, check_condition,
};
use capstone::arch::arm::{ArmInsn, ArmOperandType, ArmShift};

pub struct Branch_builder;
impl InstrBuilder for Branch_builder {
    fn build(&self) -> Vec<Opcode> {
        add_branch_def()
    }
}

pub fn add_branch_def() -> Vec<Opcode> {
    vec![
        Opcode {
            insnid: ArmInsn::ARM_INS_B as u32,
            name: "B".to_string(),
            length: 32,
            cycles: CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_B,
            adjust_cycles: None,
        },
        Opcode {
            insnid: ArmInsn::ARM_INS_BL as u32,
            name: "BL".to_string(),
            length: 32,
            cycles: CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Bl,
            adjust_cycles: None,
        },
        Opcode {
            insnid: ArmInsn::ARM_INS_BX as u32,
            name: "BX".to_string(),
            length: 32,
            cycles: CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Bx,
            adjust_cycles: None,
        },
        Opcode {
            insnid: ArmInsn::ARM_INS_BLX as u32,
            name: "BLX".to_string(),
            length: 32,
            cycles: CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Blx,
            adjust_cycles: None,
        },
    ]
}

// B{cond} label
// BL{cond} label
// BX{cond} Rm
// BLX{cond} Rm

pub struct Op_B;
impl Executable for Op_B {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        // B label
        let label = Operand_resolver(cpu, data);

        cpu.write_pc(label);
        0
    }
}

pub struct Op_Bl;
impl Executable for Op_Bl {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        // BL label
        let label = Operand_resolver(cpu, data);
        let pc = cpu.read_pc();
        let insn_len = data.insn.len() as u32;
        let return_addr = pc.wrapping_add(insn_len);

        // Set LSB of return address for Thumb mode
        cpu.write_lr(return_addr | 1);
        cpu.write_pc(label);
        print!("BL to 0x{:08X}, return addr 0x{:08X}\n", label, return_addr | 1);
        0
    }
}

pub struct Op_Bx;
impl Executable for Op_Bx {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        // BX Rm
        let val = Operand_resolver(cpu, data);
        // Bit[0] of the value in Rm must be 1, but the address to branch to is created by changing bit[0] to 0.
        let target = val & !1;
        cpu.write_pc(target);
        0
    }
}

pub struct Op_Blx;
impl Executable for Op_Blx {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        if !check_condition(cpu, data.condition()) {
            return data.size();
        }

        // BLX Rm
        let val = Operand_resolver(cpu, data);
        let pc = cpu.read_pc();
        let insn_len = data.insn.len() as u32;
        let return_addr = pc.wrapping_add(insn_len);

        cpu.write_lr(return_addr | 1);

        // Bit[0] of the value in Rm must be 1, but the address to branch to is created by changing bit[0] to 0.
        let target = val & !1;
        cpu.write_pc(target);
        0
    }
}
