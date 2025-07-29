use crate::pv::private::PvpArray;
use crate::pl::bytecode::PlInstructionPointer;

use crate::pv::Pv;

#[derive(Debug, Clone)]
struct PlStackFrame {
    // needs fields
    // return address
    // maybe last frame index?
    retaddr: PlInstructionPointer,
    lastframe: isize,
}

#[derive(Debug, Clone)]
enum PlStackElement {
    Value(Pv),
    Frame(PlStackFrame)
}

#[derive(Clone)]
pub struct PlStack {
    data: PvpArray<PlStackElement>,
    topframe: isize,
}

impl PlStack {
    pub fn new() -> Self {
        PlStack {data: PvpArray::<PlStackElement>::new_empty(), topframe: -1}
    }

    pub fn push(&mut self, other: Pv) {
        self.data.append(PlStackElement::Value(other));
    }

    pub fn push_frame(&mut self, retaddr: PlInstructionPointer) {
        let topframe = self.data.len();
        self.data.append(PlStackElement::Frame(PlStackFrame {
            retaddr,
            lastframe: self.topframe,
        }));
        self.topframe = topframe.try_into().unwrap();
    }

    pub fn pop(&mut self) {
        if let PlStackElement::Value(_) = self.topelement() {
            self.data.pop();
        } else {
            panic!("can't pop a stack frame :/");
        }
    }

    pub fn pop_frame(&mut self) -> Option<PlInstructionPointer> {
        if self.topframe == -1 {
            return None;
        }
        let frame = self.data.get(self.topframe.try_into().unwrap());
        if let PlStackElement::Frame(frame) = frame {
            self.data.popn(self.data.len() - <isize as TryInto<usize>>::try_into(self.topframe).unwrap());
            self.topframe = frame.lastframe;
            return Some(frame.retaddr);
        } else {
            panic!("can't pop a non stack frame :/");
        }
    }

    pub fn top(&self) -> Pv {
        if let PlStackElement::Value(v) = self.topelement() {
            v
        } else {
            panic!("no value at a stack frame :/");
        }
    }

    fn topelement(&self) -> PlStackElement {
        self.data.get((self.data.len() - 1).try_into().unwrap())
    }
}

impl std::fmt::Debug for PlStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f, "PlStack")
    }
}
