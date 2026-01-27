use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, Operand2_resolver, UpdateApsr_C, UpdateApsr_N, UpdateApsr_Z,
    check_condition,
};

pub struct Shiift_builder;
impl InstrBuilder for Shiift_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        addd_shift_def()
    }
}

pub fn addd_shift_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_ASR as u32,
            name: "ASR".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Asr,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_LSL as u32,
            name: "LSL".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Lsl,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_LSR as u32,
            name: "LSR".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Lsr,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_ROR as u32,
            name: "ROR".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Ror,
            adjust_cycles: None,
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_RRX as u32,
            name: "RRX".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: &Op_Rrx,
            adjust_cycles: None,
        },
    ]
}

// ASR, LSL, LSR, ROR, and RRX
// op{S}{cond} Rd, Rm, Rs
// op{S}{cond} Rd, Rm, #n
// RRX{S}{cond} Rd, Rm

pub struct Op_Asr;
impl Executable for Op_Asr {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rd, rm, mut rs_val) = Operand2_resolver(cpu, data);
        let rm_val = cpu.read_reg(rm);
        rs_val = rs_val & 0xFF; // Only bottom byte used

        let result = if rs_val == 0 {
            rm_val
        } else if rs_val >= 32 {
            if (rm_val & 0x80000000) != 0 {
                0xFFFFFFFF
            } else {
                0
            }
        } else {
            ((rm_val as i32) >> rs_val) as u32
        };

        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
            if rs_val > 0 {
                let carry = if rs_val >= 32 {
                    (rm_val >> 31) & 1
                } else {
                    (rm_val >> (rs_val - 1)) & 1
                };
                UpdateApsr_C(cpu, carry as u8);
            }
        }
    }
}

pub struct Op_Lsl;
impl Executable for Op_Lsl {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rd, rm, mut rs_val) = Operand2_resolver(cpu, data);
        let rm_val = cpu.read_reg(rm);
        rs_val = rs_val & 0xFF;

        let result = if rs_val >= 32 {
            0
        } else {
            rm_val.wrapping_shl(rs_val)
        };
        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
            if rs_val > 0 {
                let carry = if rs_val > 32 {
                    0
                } else if rs_val == 32 {
                    rm_val & 1
                } else {
                    (rm_val >> (32 - rs_val)) & 1
                };
                UpdateApsr_C(cpu, carry as u8);
            }
        }
    }
}

pub struct Op_Lsr;
impl Executable for Op_Lsr {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rd, rm, mut rs_val) = Operand2_resolver(cpu, data);
        let rm_val = cpu.read_reg(rm);
        rs_val = rs_val & 0xFF;

        let result = if rs_val >= 32 { 0 } else { rm_val >> rs_val };

        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
            if rs_val > 0 {
                let carry = if rs_val > 32 {
                    0
                } else if rs_val == 32 {
                    (rm_val >> 31) & 1
                } else {
                    (rm_val >> (rs_val - 1)) & 1
                };
                UpdateApsr_C(cpu, carry as u8);
            }
        }
    }
}

pub struct Op_Ror;
impl Executable for Op_Ror {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rd, rm, mut rs_val) = Operand2_resolver(cpu, data);
        let rm_val = cpu.read_reg(rm);
        rs_val = rs_val & 0xFF;

        let shift = rs_val & 31;
        let result = if rs_val == 0 {
            rm_val
        } else {
            rm_val.rotate_right(shift)
        };

        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
            if rs_val > 0 {
                let carry = (result >> 31) & 1;
                UpdateApsr_C(cpu, carry as u8);
            }
        }
    }
}

pub struct Op_Rrx;
impl Executable for Op_Rrx {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        if !check_condition(cpu, data.condition()) {
            return;
        }
        let (rd, rd2, rm) = Operand2_resolver(cpu, data);
        let rm_val = cpu.read_reg(rm);
        let carry_in = (cpu.read_apsr() >> 29) & 1;

        let result = (carry_in << 31) | (rm_val >> 1);
        cpu.write_reg(rd, result);

        if data.update_flags() {
            UpdateApsr_Z(cpu, result);
            UpdateApsr_N(cpu, result);
            let carry = rm_val & 1;
            UpdateApsr_C(cpu, carry as u8);
        }
    }
}

// fn get_ops_imm(ops: &Vec<u32>) -> (u32, u32, u32) {
//     if ops.len() >= 3 {
//         (ops[0], ops[1], ops[2])
//     } else {
//         (ops[0], ops[0], ops[1])
//     }
// }

// fn get_ops_reg(ops: &Vec<u32>) -> (u32, u32, u32) {
//     if ops.len() >= 3 {
//         (ops[0], ops[1], ops[2])
//     } else {
//         (ops[0], ops[0], ops[1])
//     }
// }

// fn get_ops_2(ops: &Vec<u32>) -> (u32, u32) {
//     if ops.len() >= 2 {
//         (ops[0], ops[1])
//     } else {
//         (ops[0], ops[0])
//     }
// }

// pub struct Shift_Asr_Imm;

// impl Executable for Shift_Asr_Imm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         shift_asr_imm(cpu, data);
//     }
// }

// fn shift_asr_imm(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rm, imm) = get_ops_imm(&data.operands);
//     let rm_val = cpu.read_reg(rm);

//     // ASR #0 is encoded as 0 but implies #32
//     let shift = if imm == 0 { 32 } else { imm };

//     // Asr is arithmetic shift right
//     let result = if shift >= 32 {
//         if (rm_val & 0x80000000) != 0 {
//             0xFFFFFFFF
//         } else {
//             0
//         }
//     } else {
//         ((rm_val as i32) >> shift) as u32
//     };

//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//         let carry = if shift >= 32 {
//             (rm_val >> 31) & 1
//         } else {
//             (rm_val >> (shift - 1)) & 1
//         };
//         UpdateApsr_C(cpu, carry as u8);
//     }
// }

// pub struct Shift_Asr_Reg;

// impl Executable for Shift_Asr_Reg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         shift_asr_reg(cpu, data);
//     }
// }

// fn shift_asr_reg(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rm, rs) = get_ops_reg(&data.operands);
//     let rm_val = cpu.read_reg(rm);
//     let rs_val = cpu.read_reg(rs) & 0xFF; // Only bottom byte used

//     let result = if rs_val == 0 {
//         rm_val
//     } else if rs_val >= 32 {
//         if (rm_val & 0x80000000) != 0 {
//             0xFFFFFFFF
//         } else {
//             0
//         }
//     } else {
//         ((rm_val as i32) >> rs_val) as u32
//     };

//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//         if rs_val > 0 {
//             let carry = if rs_val >= 32 {
//                 (rm_val >> 31) & 1
//             } else {
//                 (rm_val >> (rs_val - 1)) & 1
//             };
//             UpdateApsr_C(cpu, carry as u8);
//         }
//     }
// }

// pub struct Shift_Lsl_Imm;

// impl Executable for Shift_Lsl_Imm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         shift_lsl_imm(cpu, data);
//     }
// }

// fn shift_lsl_imm(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rm, imm) = get_ops_imm(&data.operands);
//     let rm_val = cpu.read_reg(rm);

//     let result = rm_val.wrapping_shl(imm);
//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//         if imm > 0 {
//             // Carry is the last bit shifted out.
//             // For LSL #n, it is bit (32 - n) of Rm.
//             // Note: If imm >= 32, result is 0.
//             let carry = if imm > 32 {
//                 0
//             } else if imm == 32 {
//                 rm_val & 1
//             } else {
//                 (rm_val >> (32 - imm)) & 1
//             };
//             UpdateApsr_C(cpu, carry as u8);
//         }
//     }
// }

// pub struct Shift_Lsl_Reg;

// impl Executable for Shift_Lsl_Reg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         shift_lsl_reg(cpu, data);
//     }
// }

// fn shift_lsl_reg(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rm, rs) = get_ops_reg(&data.operands);
//     let rm_val = cpu.read_reg(rm);
//     let rs_val = cpu.read_reg(rs) & 0xFF;

//     let result = if rs_val >= 32 {
//         0
//     } else {
//         rm_val.wrapping_shl(rs_val)
//     };
//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//         if rs_val > 0 {
//             let carry = if rs_val > 32 {
//                 0
//             } else if rs_val == 32 {
//                 rm_val & 1
//             } else {
//                 (rm_val >> (32 - rs_val)) & 1
//             };
//             UpdateApsr_C(cpu, carry as u8);
//         }
//     }
// }

// pub struct Shift_Lsr_Imm;

// impl Executable for Shift_Lsr_Imm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         shift_lsr_imm(cpu, data);
//     }
// }

// fn shift_lsr_imm(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rm, imm) = get_ops_imm(&data.operands);
//     let rm_val = cpu.read_reg(rm);

//     // LSR #0 is encoded as 0 but implies #32
//     let shift = if imm == 0 { 32 } else { imm };

//     let result = if shift >= 32 { 0 } else { rm_val >> shift };
//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//         let carry = if shift > 32 {
//             0
//         } else if shift == 32 {
//             (rm_val >> 31) & 1
//         } else {
//             (rm_val >> (shift - 1)) & 1
//         };
//         UpdateApsr_C(cpu, carry as u8);
//     }
// }

// pub struct Shift_Lsr_Reg;

// impl Executable for Shift_Lsr_Reg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         shift_lsr_reg(cpu, data);
//     }
// }

// fn shift_lsr_reg(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rm, rs) = get_ops_reg(&data.operands);
//     let rm_val = cpu.read_reg(rm);
//     let rs_val = cpu.read_reg(rs) & 0xFF;

//     let result = if rs_val >= 32 { 0 } else { rm_val >> rs_val };

//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//         if rs_val > 0 {
//             let carry = if rs_val > 32 {
//                 0
//             } else if rs_val == 32 {
//                 (rm_val >> 31) & 1
//             } else {
//                 (rm_val >> (rs_val - 1)) & 1
//             };
//             UpdateApsr_C(cpu, carry as u8);
//         }
//     }
// }

// pub struct Shift_Ror_Imm;

// impl Executable for Shift_Ror_Imm {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         shift_ror_imm(cpu, data);
//     }
// }

// fn shift_ror_imm(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rm, imm) = get_ops_imm(&data.operands);
//     let rm_val = cpu.read_reg(rm);

//     // ROR #0 is usually RRX, but strict decoding should separate them.
//     let shift = imm & 31;
//     let result = rm_val.rotate_right(shift);
//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//         let carry = if shift == 0 {
//             (result >> 31) & 1
//         } else {
//             (result >> 31) & 1
//         };
//         UpdateApsr_C(cpu, carry as u8);
//     }
// }

// pub struct Shift_Ror_Reg;

// impl Executable for Shift_Ror_Reg {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         shift_ror_reg(cpu, data);
//     }
// }

// fn shift_ror_reg(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rm, rs) = get_ops_reg(&data.operands);
//     let rm_val = cpu.read_reg(rm);
//     let rs_val = cpu.read_reg(rs) & 0xFF;

//     let shift = rs_val & 31;
//     let result = if rs_val == 0 {
//         rm_val
//     } else {
//         rm_val.rotate_right(shift)
//     };

//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//         if rs_val > 0 {
//             let carry = (result >> 31) & 1;
//             UpdateApsr_C(cpu, carry as u8);
//         }
//     }
// }

// pub struct Shift_Rrx;

// impl Executable for Shift_Rrx {
//     fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//         shift_rrx(cpu, data);
//     }
// }

// fn shift_rrx(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
//     if !check_condition(cpu, data.condition()) {
//         return;
//     }
//     let (rd, rm) = get_ops_2(&data.operands);
//     let rm_val = cpu.read_reg(rm);
//     let carry_in = (cpu.read_apsr() >> 29) & 1;

//     let result = (carry_in << 31) | (rm_val >> 1);
//     cpu.write_reg(rd, result);

//     if data.update_flags() {
//         UpdateApsr_Z(cpu, result);
//         UpdateApsr_N(cpu, result);
//         let carry = rm_val & 1;
//         UpdateApsr_C(cpu, carry as u8);
//     }
// }
