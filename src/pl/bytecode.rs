use std::rc::Rc;

use crate::pv::Pv;

pub enum PlInstruction {
	Nop,
	Hey,
	Jump(u32),
	ReturnNull,
}

pub struct PlState {
	bytecode: Rc<[PlInstruction]>,
	instruction_counter: u32,
}

impl PlState {
	pub fn new(bytecode: Rc<[PlInstruction]>) -> Self {
		PlState {bytecode, instruction_counter: 0}
	}

	// execute one instruction
	pub fn executeone(&mut self) -> Option<Pv> {
		let instruction = &self.bytecode[self.instruction_counter as usize];
		self.instruction_counter += 1;
		match instruction {
			PlInstruction::Nop => None,
			PlInstruction::Hey => {
				println!("hey");
				None
			},
			PlInstruction::Jump(offset) => {
				self.instruction_counter += offset;
				None
			},
			PlInstruction::ReturnNull => {
				Some(Pv::null())
			},
		}
	}

	// execute for up to `steps` steps
	// for web so it doesn't lock up the browser
	pub fn executesteps(&mut self, steps: u32) -> Option<Pv> {
		for _ in 0..steps {
			if let Some(value) = self.executeone() {
				return Some(value);
			}
		}

		return None
	}

	pub fn execute(&mut self) -> Pv {
		loop {
			if let Some(value) = self.executeone() {
				return value;
			}
		}
	}
}