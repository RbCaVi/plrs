use plrs::{PlInstruction, PlState};

fn main() {
	let mut state = PlState::new([PlInstruction::Hey, PlInstruction::ReturnNull]);

	dbg!(state.execute());
}