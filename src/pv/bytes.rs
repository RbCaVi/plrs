use crate::pv::private::PvpArray;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct PvBytes {
    data: PvpArray<u8>,
}

// all of these methods are inherited from PvpArray
// i can't just say "pub type PvBytes = PvpArray<u8>;"
// because string generic parameters are not allowed
// and i want a custom name for Debug
impl PvBytes {
    // allocates enough space for `len` array elements
    pub fn new_empty_sized(size: usize) -> Self {
        PvpArray::<u8>::new_empty_sized(size).into()
    }

    pub fn new_empty() -> Self {
        PvpArray::<u8>::new_empty().into()
    }

    pub fn new(pvs: &[u8]) -> Self {
        PvpArray::<u8>::new(pvs).into()
    }

    pub fn concat(self, other: &PvBytes) -> Self {
        self.data.concat(&other.data).into()
    }

    pub fn append(&mut self, other: u8) {
        self.data.append(other).into()
    }
}

impl std::fmt::Debug for PvBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f, "PvBytes")
    }
}

impl From<&[u8]> for PvBytes {
    fn from(value: &[u8]) -> Self {
        PvBytes::new(value)
    }
}

impl From<PvpArray<u8>> for PvBytes {
    fn from(value: PvpArray<u8>) -> Self {
        PvBytes {data: value}
    }
}

impl std::ops::Add<&PvBytes> for PvBytes {
    type Output = Self;

    fn add(self, other: &PvBytes) -> Self {
        self.concat(other)
    }
}