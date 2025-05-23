use crate::pv::private::PvFixedSize;

use crate::pv::Pv;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct PvObject {
    data: PvFixedSize<std::collections::HashMap<Pv, Pv>>,
}

impl PvObject {
    pub fn new_empty() -> Self {
        PvObject::new(std::collections::HashMap::new())
    }

    pub fn new(data: std::collections::HashMap<Pv, Pv>) -> Self {
        PvObject {data: data.into()}
    }
}

impl std::hash::Hash for PvObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // you know what
        // i'm going to deal with this "later"
        panic!("Tried to hash unhashable type PvObject.");
    }
}