mod pv;

use pv::{PvInvalid, PvNull, PvBool, PvInt, PvString, PvArray, PvObject, Pv};

#[cfg(test)]
mod tests {
    use super::*;

    // these four are the most pointless tests known to mankind
    #[test]
    fn test_invalid() {
        assert_eq!(Pv::invalid(), Pv::Invalid(PvInvalid));
    }

    #[test]
    fn test_null() {
        assert_eq!(Pv::null(), Pv::Null(PvNull));
    }

    #[test]
    fn test_bool() {
        assert_eq!(Pv::bool(true), Pv::Bool(PvBool(true)));
    }

    #[test]
    fn test_int() {
        assert_eq!(Pv::int(15), Pv::Int(PvInt(15)));
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