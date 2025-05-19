#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct PvInvalid;

impl PvInvalid {
    pub fn new() -> Self {
        PvInvalid
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct PvNull;

impl PvNull {
    pub fn new() -> Self {
        PvNull
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct PvBool(bool);

impl PvBool {
    pub fn new(value: bool) -> Self {
        PvBool(value)
    }
}

impl From<bool> for PvBool {
    fn from(value: bool) -> Self {
        PvBool::new(value)
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct PvInt(isize);

impl PvInt {
    pub fn new(value: isize) -> Self {
        PvInt(value)
    }
}

impl From<isize> for PvInt {
    fn from(value: isize) -> Self {
        PvInt::new(value)
    }
}

macro_rules! pvint_op_impl {
    ($optrait:ident $op:ident) => {
        impl std::ops::$optrait<&PvInt> for PvInt {
            type Output = Self;

            fn $op(self, other: &PvInt) -> Self {
                PvInt(self.0.$op(other.0))
            }
        }
    }
}

pvint_op_impl!(Add add);
pvint_op_impl!(Sub sub);
pvint_op_impl!(Mul mul);
pvint_op_impl!(Div div);
pvint_op_impl!(Rem rem);

#[cfg(test)]
mod tests {
    use super::*;

    // the most pointless tests known to mankind
    #[test]
    fn test_invalid() {
        assert_eq!(PvInvalid::new(), PvInvalid);
    }

    #[test]
    fn test_null() {
        assert_eq!(PvNull::new(), PvNull);
    }

    #[test]
    fn test_bool() {
        assert_eq!(PvBool::new(true), PvBool(true));
    }

    #[test]
    fn test_int() {
        assert_eq!(PvInt::new(15), PvInt(15));
    }

    // real
    #[test]
    fn test_int_add() {
        assert_eq!(PvInt::new(15) + PvInt::new(3), PvInt::new(18));
    }
}