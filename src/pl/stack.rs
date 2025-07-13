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
    pub fn new_empty() -> Self {
        PlStack {data: PvpArray::<PlStackElement>::new_empty(), topframe: -1}
    }

    pub fn push(self, other: Pv) -> Self {
        PlStack {
            data: self.data.append(PlStackElement::Value(other)),
            topframe: self.topframe,
        }
    }

    pub fn push_frame(self, retaddr: PlInstructionPointer) -> Self {
        let topframe = self.data.len();
        PlStack {
            data: self.data.append(PlStackElement::Frame(PlStackFrame {
                retaddr,
                lastframe: self.topframe,
            })),
            topframe: topframe.try_into().unwrap(),
        }
    }

    pub fn pop(self) -> Self {
        if let PlStackElement::Value(_) = self.topelement() {
            PlStack {data: self.data.pop(), topframe: self.topframe}
        } else {
            panic!("can't pop a stack frame :/");
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
