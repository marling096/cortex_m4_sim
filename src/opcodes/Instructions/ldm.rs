use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
    ArmOpcode, Executable, MatchFn, Operand2_resolver, UpdateApsr_C, UpdateApsr_N, UpdateApsr_V,
    UpdateApsr_Z, check_condition, op2_imm_match, op2_reg_match,
};
use capstone::arch::arm::ArmOperandType;

// op{addr_mode}{cond} Rn{!}, reglist
pub struct Op_Ldm;
impl Executable for Op_Ldm {
    fn execute(&self, cpu: &mut dyn CpuContext, data: &ArmOpcode) {
        ldm(cpu, data);
    }
}
pub fn ldm(cpu: &mut dyn CpuContext, data: &ArmOpcode) {
    if !check_condition(cpu, data.condition()) {
        return;
    }

    // Collect operands into a Vec so we can index them
    let operands: Vec<_> = data.operands().collect();
    let base_reg = operands.get(0).expect("missing base register");
    let reg_list = &operands[1..];

    let base_reg_id = match base_reg.op_type {
        ArmOperandType::Reg(reg_id) => data.resolve_reg(reg_id),
        _ => panic!("Expected base register"),
    };

    let reg_list_id = reg_list
        .iter()
        .filter_map(|op| {
            if let ArmOperandType::Reg(reg_id) = op.op_type {
                Some(data.resolve_reg(reg_id))
            } else {
                None
            }
        })
        .collect::<Vec<u32>>();

    let base_addr = cpu.read_reg(base_reg_id);

    let mut addr = base_addr;
    for reg_id in &reg_list_id {
        let value = cpu.read_mem(addr);
        cpu.write_reg(*reg_id, value);
        addr = addr.wrapping_add(4);
    }

    if data.writeback() {
        cpu.write_reg(base_reg_id, addr);
    }
}

pub struct Ldm_builder;
impl InstrBuilder for Ldm_builder {
    fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
        add_ldm_def()
    }
}
pub fn add_ldm_def() -> Vec<crate::opcodes::opcode::Opcode> {
    vec![crate::opcodes::opcode::Opcode {
        insnid: capstone::arch::arm::ArmInsn::ARM_INS_LDM as u32,
        name: "LDM".to_string(),
        length: 32,
        cycles: crate::opcodes::opcode::CycleInfo {
            fetch_cycles: 1,
            decode_cycles: 0,
            execute_cycles: 1,
        },
        exec: &Op_Ldm,
        adjust_cycles: None,
    }]
}
