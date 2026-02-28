use crate::context::CpuContext;
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{ArmOpcode, Executable, OperandResolver, check_condition};
use capstone::arch::arm::ArmOperandType;

pub struct Bit_field_builder;
impl InstrBuilder for Bit_field_builder {
	fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
		add_bit_field_def()
	}
}

pub fn add_bit_field_def() -> Vec<crate::opcodes::opcode::Opcode> {
	vec![crate::opcodes::opcode::Opcode {
		insnid: capstone::arch::arm::ArmInsn::ARM_INS_UBFX as u32,
		name: "UBFX".to_string(),
		length: 32,
		cycles: crate::opcodes::opcode::CycleInfo {
			fetch_cycles: 1,
			decode_cycles: 0,
			execute_cycles: 1,
		},
		exec: Op_Ubfx::execute,
		operand_resolver: &OpBitFieldResolver,
		adjust_cycles: None,
	}]
}

pub struct OpBitFieldResolver;
impl OperandResolver for OpBitFieldResolver {
	fn resolve(&self, data: &mut ArmOpcode) -> u32 {
		let rd = match data.get_operand(0) {
			Some(op) => match op.op_type {
				ArmOperandType::Reg(r) => data.resolve_reg(r),
				_ => 0,
			},
			None => 0,
		};
		let rn = match data.get_operand(1) {
			Some(op) => match op.op_type {
				ArmOperandType::Reg(r) => data.resolve_reg(r),
				_ => 0,
			},
			None => 0,
		};
		data.arm_operands.rd = rd;
		data.arm_operands.rn = rn;
		rd
	}
}

pub struct Op_Ubfx;
impl Executable for Op_Ubfx {
	fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
		if !check_condition(cpu, data.condition()) {
			return data.size();
		}

		let rd = data.arm_operands.rd;
		let rn = data.arm_operands.rn;
		let lsb = imm_operand(data, 2);
		let width = imm_operand(data, 3);

		calculate_ubfx_core(cpu, rd, rn, lsb, width);
		if rd == 15 { 0 } else { data.size() }
	}
}

fn imm_operand(data: &ArmOpcode, index: usize) -> u32 {
	match data.get_operand(index) {
		Some(op) => match op.op_type {
			ArmOperandType::Imm(v) => v as u32,
			_ => 0,
		},
		None => 0,
	}
}

fn calculate_ubfx_core(cpu: &mut dyn CpuContext, rd: u32, rn: u32, lsb: u32, width: u32) {
	let rn_val = cpu.read_reg(rn);

	let result = if width == 0 || lsb >= 32 {
		0
	} else {
		let eff_width = width.min(32 - lsb);
		let mask = if eff_width >= 32 {
			u32::MAX
		} else {
			(1u32 << eff_width) - 1
		};
		(rn_val >> lsb) & mask
	};

	cpu.write_reg(rd, result);
}
