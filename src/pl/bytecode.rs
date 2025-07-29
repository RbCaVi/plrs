use std::rc::Rc;

use crate::pv::Pv;
use crate::pl::stack::PlStack;

#[derive(Copy, Clone, Debug)]
pub enum PlInstruction {
	Nop,
	Hey,
	Jump(isize),
	Return,
	PushInt(isize),
	PushNull,
	PrintTop,
	Debug,
	PushFrame,
	PopFrame,
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
	stack: PlStack,
}

impl PlState {
	pub fn new<const N: usize>(bytecode: [PlInstruction; N]) -> Self {
		PlState {
			instruction_pointer: PlInstructionPointer::new(bytecode),
			stack: PlStack::new(),
		}
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
			PlInstruction::Return => {
				Some(self.stack.top())
			},
			PlInstruction::PushInt(n) => {
				self.stack.push(Pv::int(n));
				None
			},
			PlInstruction::PushNull => {
				self.stack.push(Pv::null());
				None
			},
			PlInstruction::PrintTop => {
				dbg!(self.stack.top());
				None
			},
			PlInstruction::Debug => {
				dbg!(&self.stack);
				None
			},
			PlInstruction::PushFrame => {
				self.stack.push_frame(self.instruction_pointer.clone());
				None
			},
			PlInstruction::PopFrame => {
				dbg!(self.stack.pop_frame());
				None
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