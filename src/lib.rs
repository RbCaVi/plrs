#[derive(PartialEq, Debug, Copy, Clone)]
struct PvInvalid;

impl PvInvalid {
    pub fn new() -> Self {
        PvInvalid
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
struct PvNull;

impl PvNull {
    pub fn new() -> Self {
        PvNull
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
struct PvBool(bool);

impl PvBool {
    pub fn new(value: bool) -> Self {
        PvBool(value)
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
struct PvInt(isize);

impl PvInt {
    pub fn new(value: isize) -> Self {
        PvInt(value)
    }
}

impl std::ops::Add<&PvInt> for PvInt {
    type Output = Self;

    fn add(self, other: &PvInt) -> Self {
        PvInt(self.0 + other.0)
    }
}

impl std::ops::Sub<&PvInt> for PvInt {
    type Output = Self;

    fn sub(self, other: &PvInt) -> Self {
        PvInt(self.0 - other.0)
    }
}

impl std::ops::Mul<&PvInt> for PvInt {
    type Output = Self;

    fn mul(self, other: &PvInt) -> Self {
        PvInt(self.0 * other.0)
    }
}

impl std::ops::Div<&PvInt> for PvInt {
    type Output = Self;

    fn div(self, other: &PvInt) -> Self {
        PvInt(self.0 / other.0)
    }
}

impl std::ops::Rem<&PvInt> for PvInt {
    type Output = Self;

    fn rem(self, other: &PvInt) -> Self {
        PvInt(self.0 % other.0)
    }
}

#[derive(Copy, Clone)]
struct PvStringData {
    refcount: usize,
    len: usize,
    alloc_size: usize,
}

#[derive(Debug)]
struct PvString {
    data: *mut PvStringData,
}

fn get_string_layout(size: usize) -> std::alloc::Layout {
    let layout = std::alloc::Layout::new::<PvStringData>();
    // no errors need to be handled
    // the total size does not overflow isize (i'm assuming this one)
    let (layout, _) = layout.extend(
        // align (1) is not zero
        // align (1) is a power of two
        // size (size) does not overflow isize when rounded up to align (i'm assuming this one)
        std::alloc::Layout::from_size_align(size, 1).unwrap()
    ).unwrap();
    layout
}

impl PvString {
    // allocates enough space for `len` bytes of string
    pub fn new_empty_sized(size: usize) -> Self {
        let layout = get_string_layout(size);

        let data = unsafe {std::alloc::alloc(layout)} as *mut PvStringData;

        unsafe {*data = PvStringData {refcount: 1, len: 0, alloc_size: size};}

        PvString {data}
    }

    pub fn new_empty() -> Self {
        PvString::new_empty_sized(16) // any size would work
    }

    pub fn new(str: &str) -> Self {
        let out = PvString::new_empty_sized(str.len());
        unsafe {
            (*out.data).len = str.len();
            out.get_data_mut().copy_from_slice(str.as_bytes());
        }
        out
    }

    // allocates enough space for `len` bytes of string
    unsafe fn resize(mut self, newsize: usize) {
        let data = *self.data;

        assert!(data.refcount == 1); // just a suggestion
        assert!(newsize >= data.len); // just a suggestion

        let oldlayout = get_string_layout(data.alloc_size);

        let newlayout = get_string_layout(newsize);
        
        self.data = std::alloc::realloc(self.data as *mut u8, oldlayout, newlayout.size()) as *mut PvStringData;

        (*self.data).alloc_size = newsize;
    }

    // only use when refcount = 1
    unsafe fn get_data_mut(&self) -> &mut [u8] {
        let data = unsafe {*self.data};

        assert!(data.refcount == 1); // just a suggestion

        let layout = std::alloc::Layout::new::<PvStringData>();
        // no errors need to be handled
        // the total size does not overflow isize (i'm assuming this one)
        let (_, offset) = layout.extend(
            // align (1) is not zero
            // align (1) is a power of two
            // size (data.len) does not overflow usize when rounded up to align (i'm assuming this one)
            std::alloc::Layout::from_size_align(data.len, 1).unwrap()
        ).unwrap();
        // i could probably just get the size of layout
        // but just to be safe :)

        std::slice::from_raw_parts_mut(
            (self.data as *mut u8).add(offset),
            data.len
        )
    }

    // get an immutable reference to the string data
    // only use when refcount = 1
    pub fn get_str(&self) -> &str {
        // maybe use std::str::from_utf8_unchecked()?
        unsafe {std::str::from_utf8(self.get_data_mut()).unwrap()}
    }
}

macro_rules! decref {
    ($pv:expr) => (unsafe {
        (*$pv.data).refcount -= 1;
        (*$pv.data).refcount
    } == 0)
}

macro_rules! incref {
    ($pv:expr) => (unsafe {(*$pv.data).refcount += 1})
}

impl Drop for PvString {
    fn drop(&mut self) {
        if (decref!(self)) {
            let size = unsafe {(*self.data).alloc_size};
            let layout = get_string_layout(size);

            unsafe {std::alloc::dealloc(self.data as *mut u8, layout);}
        }
    }
}

impl Clone for PvString {
    fn clone(&self) -> Self {
        incref!(self);
        PvString {data: self.data}
    }
}

impl PartialEq for PvString {
    fn eq(&self, other: &PvString) -> bool {
        self.get_str() == other.get_str()
    }
}

#[derive(PartialEq, Debug, Clone)]
enum Pv {
    Invalid(PvInvalid),
    Null(PvNull),
    Bool(PvBool),
    Int(PvInt),
    String(PvString),
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

impl std::ops::Add<&Pv> for Pv {
    type Output = Self;

    fn add(self, other: &Pv) -> Self {
        match (self, other) {
            (Pv::Int(v1), Pv::Int(v2)) => (v1 + v2).into(),
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

    #[test]
    fn test_str_empty_constructor() {
        PvString::new_empty();
    }

    #[test]
    fn test_str_constructor() {
        PvString::new("string");
    }

    #[test]
    fn test_str_eq() {
        assert_eq!(PvString::new("string"), PvString::new("string"));
    }
}