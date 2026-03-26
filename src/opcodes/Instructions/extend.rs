use crate::context::CpuContext;
use crate::opcodes::decoded::{DecodedInstructionBuilder, DecodedOperandKind};
use crate::opcodes::instruction::InstrBuilder;
use crate::opcodes::opcode::{
	ArmOpcode, Executable, OperandResolver, check_condition, resolve_op2_runtime,
};

pub struct Extend_builder;
impl InstrBuilder for Extend_builder {
	fn build(&self) -> Vec<crate::opcodes::opcode::Opcode> {
		add_extend_def()
	}
}

pub fn add_extend_def() -> Vec<crate::opcodes::opcode::Opcode> {
	vec![
		crate::opcodes::opcode::Opcode {
			insnid: crate::arch::ArmInsn::ARM_INS_UXTB as u32,
			name: "UXTB".to_string(),
			length: 32,
			cycles: crate::opcodes::opcode::CycleInfo {
				fetch_cycles: 1,
				decode_cycles: 0,
				execute_cycles: 1,
			},
			exec: Op_Uxtb::execute,
			operand_resolver: &OpExtendResolver,
			adjust_cycles: None,
		},
		crate::opcodes::opcode::Opcode {
			insnid: crate::arch::ArmInsn::ARM_INS_UXTH as u32,
			name: "UXTH".to_string(),
			length: 32,
			cycles: crate::opcodes::opcode::CycleInfo {
				fetch_cycles: 1,
				decode_cycles: 0,
				execute_cycles: 1,
			},
			exec: Op_Uxth::execute,
			operand_resolver: &OpExtendResolver,
			adjust_cycles: None,
		},
	]
}

pub struct OpExtendResolver;
impl OperandResolver for OpExtendResolver {
	fn resolve(&self, raw: &ArmOpcode, decoded: &mut DecodedInstructionBuilder) -> u32 {
		let rd = match decoded.get_operand(0) {
			Some(op) => match op.op_type {
				DecodedOperandKind::Reg(reg) => reg,
				_ => 0,
			},
			None => 0,
		};

		let rn = match decoded.get_operand(1) {
			Some(op) => match op.op_type {
				DecodedOperandKind::Reg(reg) => reg,
				_ => 0,
			},
			None => 0,
		};

		decoded.arm_operands.condition = raw.condition();
		decoded.arm_operands.rd = rd;
		decoded.arm_operands.rn = rn;
		decoded.arm_operands.op2 = decoded.get_operand(1).cloned();
		rd
	}
}

pub struct Op_Uxtb;
impl Executable for Op_Uxtb {
	#[inline(always)]
	fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
		if !check_condition(cpu, data.arm_operands.condition) {
			return data.size();
		}

		let rd = data.arm_operands.rd;
		let (op2, _) = resolve_op2_runtime(cpu, data);
		let result = op2 & 0xFF;
		cpu.write_reg(rd, result);

		if rd == 15 { 0 } else { data.size() }
	}
}

pub struct Op_Uxth;
impl Executable for Op_Uxth {
	#[inline(always)]
	fn execute(cpu: &mut crate::cpu::Cpu, data: &ArmOpcode) -> u32 {
		if !check_condition(cpu, data.arm_operands.condition) {
			return data.size();
		}

		let rd = data.arm_operands.rd;
		let (op2, _) = resolve_op2_runtime(cpu, data);
		let result = op2 & 0xFFFF;
		cpu.write_reg(rd, result);

		if rd == 15 { 0 } else { data.size() }
	}
}
