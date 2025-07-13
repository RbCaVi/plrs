use std::rc::Rc;

use crate::pv::Pv;

#[derive(Copy, Clone, Debug)]
pub enum PlInstruction {
	Nop,
	Hey,
	Jump(isize),
	ReturnNull,
}

#[derive(Clone, Debug)]
pub struct PlInstructionPointer {
	bytecode: Rc<[PlInstruction]>,
	counter: isize,
}

impl PlInstructionPointer {
	fn new<const N: usize>(bytecode: [PlInstruction; N]) -> Self {
		PlInstructionPointer {bytecode: Rc::from(bytecode), counter: 0}
	}

	fn get(&self) -> PlInstruction {
		return self.bytecode[<isize as TryInto<usize>>::try_into(self.counter).unwrap()]
	}
}

impl std::ops::AddAssign<isize> for PlInstructionPointer {
	fn add_assign(&mut self, n: isize) {
		self.counter += n;
	}
}

impl std::ops::AddAssign<&isize> for PlInstructionPointer {
	fn add_assign(&mut self, n: &isize) {
		self.counter += n;
	}
}

pub struct PlState {
	instruction_pointer: PlInstructionPointer,
}

impl PlState {
	pub fn new<const N: usize>(bytecode: [PlInstruction; N]) -> Self {
		PlState {instruction_pointer: PlInstructionPointer::new(bytecode)}
	}

	// execute one instruction
	pub fn executeone(&mut self) -> Option<Pv> {
		let instruction = self.instruction_pointer.get();
		self.instruction_pointer += 1;
		match instruction {
			PlInstruction::Nop => None,
			PlInstruction::Hey => {
				println!("hey");
				None
			},
			PlInstruction::Jump(offset) => {
				self.instruction_pointer += offset;
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