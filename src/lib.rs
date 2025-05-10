#[derive(PartialEq, Debug, Copy, Clone)]
struct PvInvalid;

#[derive(PartialEq, Debug, Copy, Clone)]
struct PvNull;

#[derive(PartialEq, Debug, Copy, Clone)]
struct PvBool(bool);

#[derive(PartialEq, Debug, Copy, Clone)]
struct PvInt(isize);

#[derive(PartialEq, Debug, Clone)]
enum Pv {
    Invalid(PvInvalid),
    Null(PvNull),
    Bool(PvBool),
    Int(PvInt),
}

impl PvInvalid {
    pub fn new() -> Self {
        PvInvalid
    }
}

impl PvNull {
    pub fn new() -> Self {
        PvNull
    }
}

impl PvBool {
    pub fn new(value: bool) -> Self {
        PvBool(value)
    }
}

impl PvInt {
    pub fn new(value: isize) -> Self {
        PvInt(value)
    }
}

impl std::ops::Add for PvInt {
    type Output = Self;

    fn add(self, other: PvInt) -> Self {
        PvInt(self.0 + other.0)
    }
}

impl std::ops::Sub for PvInt {
    type Output = Self;

    fn sub(self, other: PvInt) -> Self {
        PvInt(self.0 - other.0)
    }
}

impl std::ops::Mul for PvInt {
    type Output = Self;

    fn mul(self, other: PvInt) -> Self {
        PvInt(self.0 * other.0)
    }
}

impl std::ops::Div for PvInt {
    type Output = Self;

    fn div(self, other: PvInt) -> Self {
        PvInt(self.0 / other.0)
    }
}

impl std::ops::Rem for PvInt {
    type Output = Self;

    fn rem(self, other: PvInt) -> Self {
        PvInt(self.0 % other.0)
    }
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
}

impl From<PvInvalid> for Pv {
    fn from(value: PvInvalid) -> Self {
        Pv::Invalid(value)
    }
}

impl From<PvNull> for Pv {
    fn from(value: PvNull) -> Self {
        Pv::Null(value)
    }
}

impl From<PvBool> for Pv {
    fn from(value: PvBool) -> Self {
        Pv::Bool(value)
    }
}

impl From<PvInt> for Pv {
    fn from(value: PvInt) -> Self {
        Pv::Int(value)
    }
}

impl From<bool> for Pv {
    fn from(value: bool) -> Self {
        Pv::bool(value)
    }
}

impl From<isize> for Pv {
    fn from(value: isize) -> Self {
        Pv::int(value)
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

impl std::ops::Add for Pv {
    type Output = Self;

    fn add(self, other: Pv) -> Self {
        match (self, other) {
            (Pv::Int(v1), Pv::Int(v2)) => (v1 + v2).into(),
            _ => Pv::invalid(),
        }
    }
}

impl std::ops::Sub for Pv {
    type Output = Self;

    fn sub(self, other: Pv) -> Self {
        match (self, other) {
            (Pv::Int(v1), Pv::Int(v2)) => (v1 - v2).into(),
            _ => Pv::invalid(),
        }
    }
}

impl std::ops::Mul for Pv {
    type Output = Self;

    fn mul(self, other: Pv) -> Self {
        match (self, other) {
            (Pv::Int(v1), Pv::Int(v2)) => (v1 * v2).into(),
            _ => Pv::invalid(),
        }
    }
}

impl std::ops::Div for Pv {
    type Output = Self;

    fn div(self, other: Pv) -> Self {
        match (self, other) {
            (Pv::Int(v1), Pv::Int(v2)) => (v1 / v2).into(),
            _ => Pv::invalid(),
        }
    }
}

impl std::ops::Rem for Pv {
    type Output = Self;

    fn rem(self, other: Pv) -> Self {
        match (self, other) {
            (Pv::Int(v1), Pv::Int(v2)) => (v1 % v2).into(),
            _ => Pv::invalid(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // the most pointless tests known to mankind
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

    #[test]
    fn test_int_add() {
        assert_eq!(Pv::int(15) + Pv::int(3), Pv::int(18));
    }

    #[test]
    fn test_add_invalid() {
        assert_eq!(Pv::int(15) + Pv::bool(true), Pv::invalid());
    }
}