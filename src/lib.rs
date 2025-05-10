#[derive(PartialEq)]
struct PvInvalid;

#[derive(PartialEq)]
struct PvNull;

#[derive(PartialEq)]
struct PvBool(bool);

#[derive(PartialEq)]
struct PvInt(isize);

#[derive(PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    // the most pointless tests known to mankind
    #[test]
    fn test_invalid() {
        let val = Pv::invalid();
        match val {
            Pv::Invalid(_) => println!("ok"),
            _ => panic!("Expected Pv::Invalid"),
        }
    }

    #[test]
    fn test_null() {
        let val = Pv::null();
        match val {
            Pv::Null(_) => println!("ok"),
            _ => panic!("Expected Pv::Null"),
        }
    }

    #[test]
    fn test_bool() {
        let val = Pv::bool(true);
        match val {
            Pv::Bool(PvBool(true)) => println!("ok"),
            _ => panic!("Expected Pv::Bool(PvBool(true))"),
        }
    }

    #[test]
    fn test_int() {
        let val = Pv::int(15);
        match val {
            Pv::Int(PvInt(15)) => println!("ok"),
            _ => panic!("Expected Pv::Int(PvInt(15))"),
        }
    }
}