use plrs::{PlInstruction, PlState};

fn main() {
	let mut state = PlState::new([
		PlInstruction::Hey,
		PlInstruction::PushInt(14),
		PlInstruction::PrintTop,
		PlInstruction::PushInt(3),
		PlInstruction::PrintTop,
		PlInstruction::ReturnNull
	]);

	dbg!(state.execute());
}