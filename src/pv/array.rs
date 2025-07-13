use crate::pv::private::PvpArray;

use crate::pv::Pv;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct PvArray {
    data: PvpArray<Pv>,
}

// all of these methods are inherited from PvpArray
// i can't just say "pub type PvArray = PvpArray<Pv>;"
// because string generic parameters are not allowed
// and i want a custom name for Debug
impl PvArray {
    // allocates enough space for `len` array elements
    pub fn new_empty_sized(size: usize) -> Self {
        PvpArray::<Pv>::new_empty_sized(size).into()
    }

    pub fn new_empty() -> Self {
        PvpArray::<Pv>::new_empty().into()
    }

    pub fn new(pvs: &[Pv]) -> Self {
        PvpArray::<Pv>::new(pvs).into()
    }

    pub fn concat(&mut self, other: &PvArray) {
        self.data.concat(&other.data)
    }

    pub fn append(&mut self, other: Pv) {
        self.data.append(other)
    }
}

impl std::fmt::Debug for PvArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f, "PvArray")
    }
}

impl From<&[Pv]> for PvArray {
    fn from(value: &[Pv]) -> Self {
        PvArray::new(value)
    }
}

impl From<PvpArray<Pv>> for PvArray {
    fn from(value: PvpArray<Pv>) -> Self {
        PvArray {data: value}
    }
}

impl std::ops::Add<&PvArray> for PvArray {
    type Output = Self;

    fn add(mut self, other: &PvArray) -> Self {
        self.concat(other);
        self
    }
}