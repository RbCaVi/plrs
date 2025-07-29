use plrs::{PlInstruction, PlState};

fn main() {
	let mut state = PlState::new([
		PlInstruction::Hey,
		PlInstruction::PushInt(14),
		PlInstruction::PrintTop,
		PlInstruction::PushInt(3),
		PlInstruction::PrintTop,
		PlInstruction::Debug,
		PlInstruction::PushFrame,
		PlInstruction::PushInt(23),
		PlInstruction::Debug,
		PlInstruction::PushFrame,
		PlInstruction::PushInt(8),
		PlInstruction::Debug,
		PlInstruction::PopFrame,
		PlInstruction::PushInt(17),
		PlInstruction::Debug,
		PlInstruction::PopFrame,
		PlInstruction::Debug,
		PlInstruction::Return,
	]);

	dbg!(state.execute());
}