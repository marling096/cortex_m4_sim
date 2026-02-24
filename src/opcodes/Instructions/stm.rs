use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, OperandResolver, check_condition,
};
use capstone::arch::arm::ArmOperandType;

// op{addr_mode}{cond} Rn{!}, reglist
pub struct Op_Stm;
impl Executable for Op_Stm {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
        stm(cpu, data)
    }
}

pub fn stm(cpu: &mut dyn CpuContext, data: &ArmOpcode) -> u32 {
    if !check_condition(cpu, data.condition()) {
        return data.size();
    }
    // Use transed_operands populated by resolver: [base, rlist...]
    let base_reg_id = data.transed_operands.get(0).copied().unwrap_or(0);
    let reg_list_ids = if data.transed_operands.len() > 1 {
        data.transed_operands[1..].to_vec()
    } else {
        Vec::new()
    };

    let base_addr = cpu.read_reg(base_reg_id);
    let mut addr = base_addr;
    for reg_id in &reg_list_ids {
        let value = cpu.read_reg(*reg_id);
        cpu.write_mem(addr, value);
        addr = addr.wrapping_add(4);
    }

    if data.writeback() {
        cpu.write_reg(base_reg_id, addr);
    }

    data.size()
}

pub struct Stm_builder;
impl InstrBuilder for Stm_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_stm_def()
    }
}
pub fn add_stm_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![crate::opcodes::opcode::Opcode {
        insnid: capstone::arch::arm::ArmInsn::ARM_INS_STM as u32,
        name: "STM".to_string(),
        length: 32,
        cycles: crate::opcodes::opcode::CycleInfo {
            fetch_cycles: 1,
            decode_cycles: 0,
            execute_cycles: 1,
        },
        exec: &Op_Stm,
        operand_resolver: &OpStm_resolver,
        adjust_cycles: None,
    }]
}

pub struct OpStm_resolver;
impl OperandResolver for OpStm_resolver {
    fn resolve(&self, data: &mut ArmOpcode) -> u32 {
        let operands: Vec<_> = data.operands().collect();
        let base_reg = operands.get(0).expect("missing base register");

        if let ArmOperandType::Reg(reg_id) = base_reg.op_type {
            let base = data.resolve_reg(reg_id);
            data.transed_operands.reserve(operands.len());
            data.transed_operands.push(base);
            for op in &operands[1..] {
                if let ArmOperandType::Reg(r) = op.op_type {
                    data.transed_operands.push(data.resolve_reg(r));
                }
            }
            base
        } else {
            panic!("Expected base register");
        }
    }
}
