use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, CycleInfo, Executable, OperandResolver, check_condition, count_reg_operands,
    reg_list_contains, resolve_multi_reg_operands,
};
use capstone::arch::arm::{ArmOperand, ArmReg};

pub struct Stack_builder;
impl InstrBuilder for Stack_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_stack_def()
    }
}

pub fn add_stack_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_PUSH as u32,
            name: "PUSH".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Push::execute,
            operand_resolver: &OpStackResolver,
            adjust_cycles: Some(adjust_push_cycles),
        },
        crate::opcodes::opcode::Opcode {
            insnid: capstone::arch::arm::ArmInsn::ARM_INS_POP as u32,
            name: "POP".to_string(),
            length: 32,
            cycles: crate::opcodes::opcode::CycleInfo {
                fetch_cycles: 1,
                decode_cycles: 0,
                execute_cycles: 1,
            },
            exec: Op_Pop::execute,
            operand_resolver: &OpStackResolver,
            adjust_cycles: Some(adjust_pop_cycles),
        },
    ]
}

fn adjust_push_cycles(cycles: &mut CycleInfo, operands: &[ArmOperand]) {
    let reg_count = count_reg_operands(operands);
    cycles.execute_cycles = 1u32.saturating_add(reg_count);
}

fn adjust_pop_cycles(cycles: &mut CycleInfo, operands: &[ArmOperand]) {
    let reg_count = count_reg_operands(operands);
    let mut execute = 1u32.saturating_add(reg_count);
    if reg_list_contains(operands, ArmReg::ARM_REG_PC as u16, false) {
        execute = execute.saturating_add(1);
    }
    cycles.execute_cycles = execute;
}

// PUSH{cond} reglist
// POP{cond} reglist
pub struct Op_Push;
impl Executable for Op_Push {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        stack_push(cpu, data)
    }
}

pub struct Op_Pop;
impl Executable for Op_Pop {
    #[inline(always)]
    fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
        stack_pop(cpu, data)
    }
}

fn stack_push(cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
    if !check_condition(cpu, data.arm_operands.condition) {
        return data.size();
    }
    data.op_writer();
    let mut sp = cpu.read_reg(13);
    // print!("SP before PUSH:0x{:08X}\n", sp);
    // PUSH: full-descending (pre-decrement)
    let regs = &data.transed_operands;
    let count = regs.len() as u32;
    let mut addr = sp.wrapping_sub(4 * count);
    for &r in regs {
        let val = cpu.read_reg(r);
        cpu.write_mem(addr, val);
        addr = addr.wrapping_add(4);
    }
    sp = sp.wrapping_sub(4 * count);
    // print!("SP:0x{:08X}\n", sp);
    cpu.write_reg(13, sp);
    data.size()
}

fn stack_pop(cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
    if !check_condition(cpu, data.arm_operands.condition) {
        return data.size();
    }

    let mut sp = cpu.read_reg(13);
    // POP: full-descending stack, so pop is post-increment
    let regs = &data.transed_operands;
    let mut pc_popped = false;
    let mut pc_val = 0u32;
    for &r in regs {
        let val = cpu.read_mem(sp);
        sp = sp.wrapping_add(4);
        if r == 15 {
            // 鍏堟洿鏂?SP锛屽啀澶勭悊 PC/EXC_RETURN
            pc_popped = true;
            pc_val = val;
        } else {
            cpu.write_reg(r, val);
        }
    }
    cpu.write_reg(13, sp);
    if pc_popped {
        // 濡傛灉寮瑰嚭鍊兼槸 EXC_RETURN锛岃Е鍙戝紓甯歌繑鍥烇紙涓嶆竻闄?Thumb bit锛?
        if !cpu.try_exception_return(pc_val) {
            cpu.write_reg(15, pc_val & !1);
        }
        return 0;
    }
    data.size()
}

pub struct OpStackResolver;
impl OperandResolver for OpStackResolver {
    fn resolve(&self, data: &mut ArmOpcode) -> u32 {
        resolve_multi_reg_operands(data, false)
    }
}
