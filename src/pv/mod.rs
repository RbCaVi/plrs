pub mod private; // maybe rename this to implementation?
mod singletons;
mod string;
mod array;
mod object;

pub use singletons::{PvInvalid, PvNull, PvBool, PvInt};
pub use string::PvString;
pub use array::PvArray;
pub use object::PvObject;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Pv {
    Invalid(PvInvalid),
    Null(PvNull),
    Bool(PvBool),
    Int(PvInt),
    String(PvString),
    Array(PvArray),
    Object(PvObject),
}

impl Pv {
    pub fn invalid() -> Self {
        Pv::Invalid(PvInvalid::new())
    }
    
    pub fn null() -> Self {
        Pv::Null(PvNull::new())
    }
    
    pub fn bool(value: bool) -> Self {
        Pv::Bool(PvBool::new(value))
    }
    
    pub fn int(value: isize) -> Self {
        Pv::Int(PvInt::new(value))
    }
    
    pub fn array() -> Self {
        Pv::Array(PvArray::new_empty())
    }
    
    pub fn object() -> Self {
        Pv::Object(PvObject::new_empty())
    }
}

impl<T: Into<Pv>> From<Option<T>> for Pv {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => value.into(),
            None => Pv::null(),
        }
    }
}

impl std::ops::Add<&Pv> for Pv {
    type Output = Self;

    fn add(self, other: &Pv) -> Self {
        match (self, other) {
            (Pv::Int(v1), Pv::Int(v2)) => (v1 + v2).into(),
            (Pv::String(v1), Pv::String(v2)) => (v1 + v2).into(),
            _ => Pv::invalid(),
        }
    }
}

impl std::ops::Sub<&Pv> for Pv {
    type Output = Self;

    fn sub(self, other: &Pv) -> Self {
        match (self, other) {
            (Pv::Int(v1), Pv::Int(v2)) => (v1 - v2).into(),
            _ => Pv::invalid(),
        }
    }
}

impl std::ops::Mul<&Pv> for Pv {
    type Output = Self;

    fn mul(self, other: &Pv) -> Self {
        match (self, other) {
            (Pv::Int(v1), Pv::Int(v2)) => (v1 * v2).into(),
            _ => Pv::invalid(),
        }
    }
}

impl std::ops::Div<&Pv> for Pv {
    type Output = Self;

    fn div(self, other: &Pv) -> Self {
        match (self, other) {
            (Pv::Int(v1), Pv::Int(v2)) => (v1 / v2).into(),
            _ => Pv::invalid(),
        }
    }
}

impl std::ops::Rem<&Pv> for Pv {
    type Output = Self;

    fn rem(self, other: &Pv) -> Self {
        match (self, other) {
            (Pv::Int(v1), Pv::Int(v2)) => (v1 % v2).into(),
            _ => Pv::invalid(),
        }
    }
}

macro_rules! pvfrom {
    ($item:ident, $type:ty) => {
        impl From<$type> for Pv {
            fn from(value: $type) -> Self {
                Pv::$item(value)
            }
        }
    }
}

macro_rules! pvfromtrans {
    ($type:ty, $intertype:ty) => {
        impl From<$type> for Pv {
            fn from(value: $type) -> Self {
                <$type as Into<$intertype>>::into(value).into()
            }
        }
    }
}

pvfrom!(Invalid, PvInvalid);
pvfrom!(Null, PvNull);
pvfrom!(Bool, PvBool);
pvfromtrans!(bool, PvBool);
pvfrom!(Int, PvInt);
pvfromtrans!(isize, PvInt);
pvfrom!(String, PvString);
pvfromtrans!(&str, PvString);
pvfrom!(Array, PvArray);
pvfromtrans!(&[Pv], PvArray);

macro_rules! unref_op_impl {
    ($type1:ident $type2:ident $optrait:ident $op:ident) => {
        impl std::ops::$optrait<$type2> for $type1 {
            // i feel like the 'static lifetime is not a good idea
            // but i can't put an "unconstrained lifetime" to use instead
            type Output = <$type1 as std::ops::$optrait<&'static $type2>>::Output;

            fn $op(self, other: $type2) -> Self::Output {
                self.$op(&other)
            }
        }
    }
}

unref_op_impl!(Pv Pv Add add);
unref_op_impl!(Pv Pv Sub sub);
unref_op_impl!(Pv Pv Mul mul);
unref_op_impl!(Pv Pv Div div);
unref_op_impl!(Pv Pv Rem rem);

unref_op_impl!(PvInt PvInt Add add);
unref_op_impl!(PvInt PvInt Sub sub);
unref_op_impl!(PvInt PvInt Mul mul);
unref_op_impl!(PvInt PvInt Div div);
unref_op_impl!(PvInt PvInt Rem rem);

unref_op_impl!(PvString PvString Add add);

unref_op_impl!(PvArray PvArray Add add);

#[cfg(test)]
mod tests {
    use super::*;

    // these four are the most pointless tests known to mankind
    #[test]
    fn test_invalid() {
        assert_eq!(Pv::invalid(), Pv::Invalid(PvInvalid::new()));
    }

    #[test]
    fn test_null() {
        assert_eq!(Pv::null(), Pv::Null(PvNull::new()));
    }

    #[test]
    fn test_bool() {
        assert_eq!(Pv::bool(true), Pv::Bool(PvBool::new(true)));
    }

    #[test]
    fn test_int() {
        assert_eq!(Pv::int(15), Pv::Int(PvInt::new(15)));
    }

    // real stuff
    #[test]
    fn test_int_add() {
        assert_eq!(Pv::int(15) + Pv::int(3), Pv::int(18));
    }

    #[test]
    fn test_add_invalid() {
        assert_eq!(Pv::int(15) + Pv::bool(true), Pv::invalid());
    }

    #[test]
    fn test_string_empty_constructor() {
        PvString::new_empty();
    }

    #[test]
    fn test_string_empty_eq() {
        assert_eq!(PvString::new_empty(), PvString::new_empty());
    }

    #[test]
    fn test_string_constructor() {
        PvString::new("string");
    }

    #[test]
    fn test_string_eq() {
        assert_eq!(PvString::new("string"), PvString::new("string"));
    }

    #[test]
    fn test_string_concat() {
        assert_eq!(PvString::new("string").concat(&PvString::new("STRING")), PvString::new("stringSTRING"));
    }

    #[test]
    fn test_string_concat_unchanged() {
        let a = PvString::new("string");
        let b = PvString::new("STRING");
        assert_eq!(a.clone().concat(&b), PvString::new("stringSTRING"));
        assert_eq!(a, PvString::new("string"));
        assert_eq!(b, PvString::new("STRING"));
    }

    // use the PvString::resize_move() path
    #[test]
    fn test_string_concat2() {
        assert_eq!(PvString::new("s").concat(&PvString::new("STRING")), PvString::new("sSTRING"));
    }

    #[test]
    fn test_string_concat_unchanged2() {
        let a = PvString::new("s");
        let b = PvString::new("STRING");
        assert_eq!(a.clone().concat(&b), PvString::new("sSTRING"));
        assert_eq!(a, PvString::new("s"));
        assert_eq!(b, PvString::new("STRING"));
    }

    #[test]
    fn test_array_empty_constructor() {
        PvArray::new_empty();
    }

    #[test]
    fn test_array_empty_eq() {
        assert_eq!(PvArray::new_empty(), PvArray::new_empty());
    }

    #[test]
    fn test_array_constructor() {
        PvArray::new(&["string".into()]);
    }

    #[test]
    fn test_array_eq() {
        assert_eq!(PvArray::new(&["string".into()]), PvArray::new(&["string".into()]));
    }

    #[test]
    fn test_array_dbg() {
        dbg!(PvArray::new(&["string".into()]));
    }

    #[test]
    fn test_array_concat_empty() {
        assert_eq!(PvArray::new(&["string".into()]).concat(&PvArray::new(&[])), PvArray::new(&["string".into()]));
    }

    #[test]
    fn test_array_concat() {
        assert_eq!(PvArray::new(&["string".into()]).concat(&PvArray::new(&["STRING".into()])), PvArray::new(&["string".into(), "STRING".into()]));
    }

    #[test]
    fn test_array_concat_unchanged() {
        let a = PvArray::new(&["string".into()]);
        let b = PvArray::new(&["STRING".into()]);
        assert_eq!(a.clone().concat(&b), PvArray::new(&["string".into(), "STRING".into()]));
        assert_eq!(a, PvArray::new(&["string".into()]));
        assert_eq!(b, PvArray::new(&["STRING".into()]));
    }

    // use the PvArray::resize_move() path
    #[test]
    fn test_array_concat2() {
        assert_eq!(PvArray::new(&["s".into()]).concat(&PvArray::new(&["STRING".into()])), PvArray::new(&["s".into(), "STRING".into()]));
    }

    #[test]
    fn test_array_concat_unchanged2() {
        let a = PvArray::new(&["s".into()]);
        let b = PvArray::new(&["STRING".into()]);
        assert_eq!(a.clone().concat(&b), PvArray::new(&["s".into(), "STRING".into()]));
        assert_eq!(a, PvArray::new(&["s".into()]));
        assert_eq!(b, PvArray::new(&["STRING".into()]));
    }
}