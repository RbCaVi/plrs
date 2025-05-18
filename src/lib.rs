#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
struct PvInvalid;

impl PvInvalid {
    pub fn new() -> Self {
        PvInvalid
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
struct PvNull;

impl PvNull {
    pub fn new() -> Self {
        PvNull
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
struct PvBool(bool);

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
struct PvInt(isize);

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

macro_rules! decref {
    ($pv:expr) => (unsafe {
        (*$pv.data).refcount -= 1;
        (*$pv.data).refcount
    } == 0)
}

macro_rules! incref {
    ($pv:expr) => (unsafe {(*$pv.data).refcount += 1})
}

#[derive(Copy, Clone)]
struct PvStringData {
    refcount: usize,
    len: usize,
    alloc_size: usize,
}

#[derive(Eq)]
struct PvString {
    data: *mut PvStringData,
}

impl PvString {
    // get a Layout that fits a PvStringData with `size` bytes after it
    fn get_layout(size: usize) -> std::alloc::Layout {
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

    // allocates enough space for `len` bytes of string
    pub fn new_empty_sized(size: usize) -> Self {
        let layout = PvString::get_layout(size);

        let data = unsafe {std::alloc::alloc(layout)} as *mut PvStringData;

        unsafe {*data = PvStringData {refcount: 1, len: 0, alloc_size: size};}

        PvString {data}
    }

    pub fn new_empty() -> Self {
        PvString::new_empty_sized(16) // any size would work
    }

    pub fn new(str: &str) -> Self {
        let out = PvString::new_empty_sized(str.len() * 2); // any >= str.len()
        unsafe {
            (*out.data).len = str.len();
        }
        out.get_data_mut().copy_from_slice(str.as_bytes());
        out
    }

    // move one copy of this string out and resize its allocation
    // will reuse the old allocation if possible
    unsafe fn resize_move(mut self, newsize: usize) -> Self {
        let data = *self.data;

        if data.refcount == 1 {
            assert!(newsize >= data.len); // just a suggestion

            let oldlayout = PvString::get_layout(data.alloc_size);

            let newlayout = PvString::get_layout(newsize);
            
            self.data = std::alloc::realloc(self.data as *mut u8, oldlayout, newlayout.size()) as *mut PvStringData;

            (*self.data).alloc_size = newsize;

            self
        } else {
            let out = PvString::new_empty_sized(newsize);
            (*out.data).len = data.len;
            out.get_data_mut().copy_from_slice(self.get_data_mut());
            out
        }
    }

    // get a mutable slice reference to the string data
    // only use when refcount = 1
    fn get_data_mut(&self) -> &mut [u8] {
        let data = unsafe {*self.data};

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

        unsafe {
            std::slice::from_raw_parts_mut(
                (self.data as *mut u8).add(offset),
                data.len
            )
        }
    }

    // get an immutable reference to the string data
    // only use when refcount = 1
    fn get_str(&self) -> &str {
        // maybe use std::str::from_utf8_unchecked()?
        // (because the "only" way to get a PvString is from a &str)
        std::str::from_utf8(self.get_data_mut()).unwrap()
    }

    pub fn concat(self, other: &PvString) -> Self {
        let data = unsafe {*self.data};
        let otherdata = unsafe {*other.data};

        let s = if data.refcount == 1 && data.alloc_size >= data.len + otherdata.len {
            self
        } else {
            unsafe {
                self.resize_move((data.len + otherdata.len) * 2)
            }
        };

        unsafe {
            (*s.data).len += otherdata.len;
        }

        s.get_data_mut()[data.len..].copy_from_slice(other.get_data_mut());

        s
    }
}

impl std::fmt::Debug for PvString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = unsafe {*self.data};
        f.debug_struct("PvString")
         .field("len", &data.len)
         .field("alloc_size", &data.alloc_size)
         .field("data", &self.get_str())
         .finish()
    }
}

impl Drop for PvString {
    fn drop(&mut self) {
        if (decref!(self)) {
            let size = unsafe {(*self.data).alloc_size};
            let layout = PvString::get_layout(size);

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

impl std::hash::Hash for PvString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_str().hash(state);
    }
}

impl From<&str> for PvString {
    fn from(value: &str) -> Self {
        PvString::new(value)
    }
}

impl std::ops::Add<&PvString> for PvString {
    type Output = Self;

    fn add(self, other: &PvString) -> Self {
        self.concat(other)
    }
}

#[derive(Copy, Clone)]
struct PvArrayData {
    refcount: usize,
    len: usize,
    alloc_size: usize,
}

#[derive(Eq)]
struct PvArray {
    data: *mut PvArrayData,
}

macro_rules! clone_to_uninit {
    ($src:expr, $dst:expr, $offset:expr, $len:expr) => (
        let src = $src;
        let dst = $dst;
        for i in 0..$len {
            dst[$offset + i].write(src[i].clone());
        }
    )
}

impl PvArray {
    // get a Layout that fits a PvArrayData with `size` Pv's after it
    fn get_layout(size: usize) -> std::alloc::Layout {
        let mut layout = std::alloc::Layout::new::<PvArrayData>();
        // i'm not assuming nightly rust with std::alloc::Layout::repeat()
        for _ in 0..size {
            // no errors need to be handled
            // the total size does not overflow isize (i'm assuming this one)
            (layout, _) = layout.extend(
                std::alloc::Layout::new::<Pv>()
            ).unwrap();
        }
        layout
    }

    // allocates enough space for `len` array elements
    pub fn new_empty_sized(size: usize) -> Self {
        let layout = PvArray::get_layout(size);

        let data = unsafe {std::alloc::alloc(layout)} as *mut PvArrayData;

        unsafe {*data = PvArrayData {refcount: 1, len: 0, alloc_size: size};}

        PvArray {data}
    }

    pub fn new_empty() -> Self {
        PvArray::new_empty_sized(16) // any size would work
    }

    pub fn new(pvs: &[Pv]) -> Self {
        let out = PvArray::new_empty_sized(pvs.len() * 2); // any >= str.len()
        unsafe {
            (*out.data).len = pvs.len();
        }

        clone_to_uninit!(pvs, out.get_data_mut(), 0, pvs.len());

        out
    }

    // move one copy of this array out and resize its allocation
    // will reuse the old allocation if possible
    unsafe fn resize_move(mut self, newsize: usize) -> Self {
        let data = *self.data;

        if data.refcount == 1 {
            assert!(newsize >= data.len); // just a suggestion

            let oldlayout = PvArray::get_layout(data.alloc_size);

            let newlayout = PvArray::get_layout(newsize);
            
            self.data = std::alloc::realloc(self.data as *mut u8, oldlayout, newlayout.size()) as *mut PvArrayData;

            (*self.data).alloc_size = newsize;

            self
        } else {
            let out = PvArray::new_empty_sized(newsize);
            (*out.data).len = data.len;
            clone_to_uninit!(self.get_data(), out.get_data_mut(), 0, data.len);
            out
        }
    }

    // get a mutable slice reference to the array data
    // only use when refcount = 1
    fn get_data_mut(&self) -> &mut [std::mem::MaybeUninit<Pv>] {
        let data = unsafe {*self.data};

        let layout = std::alloc::Layout::new::<PvArrayData>();
        // no errors need to be handled
        // the total size does not overflow isize (i'm assuming this one)
        let (_, offset) = layout.extend(
            std::alloc::Layout::new::<Pv>()
        ).unwrap();
        // i could probably just get the size of layout
        // but just to be safe :)

        unsafe {
            std::slice::from_raw_parts_mut(
                (self.data as *mut u8).add(offset) as *mut std::mem::MaybeUninit<Pv>,
                data.len
            )
        }
    }

    // get an immutable reference to the array data
    // only use when refcount = 1
    fn get_data(&self) -> &[Pv] {
        unsafe {std::mem::transmute::<_, _>(self.get_data_mut())}
    }

    pub fn concat(self, other: &PvArray) -> Self {
        let data = unsafe {*self.data};
        let otherdata = unsafe {*other.data};

        let s = if data.refcount == 1 && data.alloc_size >= data.len + otherdata.len {
            self
        } else {unsafe {
            self.resize_move((data.len + otherdata.len) * 2)
        }};

        unsafe {
            (*s.data).len += otherdata.len;
        }

        clone_to_uninit!(other.get_data(), s.get_data_mut(), data.len, otherdata.len);

        s
    }

    pub fn append(self, other: &Pv) -> Self {
        let data = unsafe {*self.data};

        let s = if data.refcount == 1 && data.alloc_size >= data.len + 1 {
            self
        } else {unsafe {
            self.resize_move((data.len + 1) * 2)
        }};

        unsafe {
            (*s.data).len += 1;
        }

        s.get_data_mut()[data.len].write(other.clone());

        s
    }
}

impl std::fmt::Debug for PvArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = unsafe {*self.data};
        f.debug_struct("PvArray")
         .field("len", &data.len)
         .field("alloc_size", &data.alloc_size)
         .field("data", &self.get_data())
         .finish()
    }
}

impl Drop for PvArray {
    fn drop(&mut self) {
        if (decref!(self)) {
            let size = unsafe {(*self.data).alloc_size};
            let layout = PvArray::get_layout(size);

            for mval in self.get_data_mut() {
                // take the value (run its destructor)
                // std::mem::MaybeUninit::assume_init_read() instead of
                // std::mem::MaybeUninit::assume_init() so it doesn't
                // consume `mval`
                unsafe {mval.assume_init_read();}
            }

            unsafe {std::alloc::dealloc(self.data as *mut u8, layout);}
        }
    }
}

impl Clone for PvArray {
    fn clone(&self) -> Self {
        incref!(self);
        PvArray {data: self.data}
    }
}

impl PartialEq for PvArray {
    fn eq(&self, other: &PvArray) -> bool {
        self.get_data() == other.get_data()
    }
}

impl std::hash::Hash for PvArray {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_data().hash(state);
    }
}

impl From<&[Pv]> for PvArray {
    fn from(value: &[Pv]) -> Self {
        PvArray::new(value)
    }
}

impl std::ops::Add<&PvArray> for PvArray {
    type Output = Self;

    fn add(self, other: &PvArray) -> Self {
        self.concat(other)
    }
}

struct PvFixedSizeData<T> {
    refcount: usize,
    data: T,
}

struct PvFixedSize<T> {
    data: *mut PvFixedSizeData<T>,
}

impl<T> PvFixedSize<T> {
    // get a Layout that fits
    fn get_layout() -> std::alloc::Layout {
        std::alloc::Layout::new::<PvFixedSizeData<T>>()
    }

    pub fn new(val: T) -> Self {
        let layout = PvFixedSize::<T>::get_layout();

        let data = unsafe {std::alloc::alloc(layout)} as *mut PvFixedSizeData<T>;

        unsafe {std::ptr::write(data, PvFixedSizeData::<T> {refcount: 1, data: val});}

        PvFixedSize::<T> {data}
    }

    // get a mutable slice reference to the array data
    // only use when refcount = 1
    fn get_data_mut(&self) -> &mut T {
        unsafe {&mut (*self.data).data}
    }

    // get an immutable reference to the array data
    // only use when refcount = 1
    fn get_data(&self) -> &T {
        self.get_data_mut()
    }
}

impl<T: Clone> PvFixedSize<T> {
    // move one copy of this value out
    // will reuse the old allocation if possible
    fn move_out(self) -> Self {
        if unsafe {(*self.data).refcount} == 1 {
            self
        } else {
            PvFixedSize::<T>::new(self.get_data().clone())
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for PvFixedSize<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<PvFixedSize<T>>())
         .field("data", &self.get_data())
         .finish()
    }
}

impl<T> Drop for PvFixedSize<T> {
    fn drop(&mut self) {
        if (decref!(self)) {
            let layout = PvFixedSize::<T>::get_layout();

            // this should take the value out so it gets dropped
            unsafe {std::ptr::read(self.data);}

            unsafe {std::alloc::dealloc(self.data as *mut u8, layout);}
        }
    }
}

impl<T> Clone for PvFixedSize<T> {
    fn clone(&self) -> Self {
        incref!(self);
        PvFixedSize::<T> {data: self.data}
    }
}

impl<T: PartialEq> PartialEq for PvFixedSize<T> {
    fn eq(&self, other: &PvFixedSize<T>) -> bool {
        self.get_data() == other.get_data()
    }
}

impl<T: Eq> Eq for PvFixedSize<T> {}

impl<T: std::hash::Hash> std::hash::Hash for PvFixedSize<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_data().hash(state);
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
struct PvObject {
    data: PvFixedSize<std::collections::HashMap<Pv, Pv>>,
}

impl std::hash::Hash for PvObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // you know what
        // i'm going to deal with this "later"
        panic!("Tried to hash unhashable type PvObject.");
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
enum Pv {
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