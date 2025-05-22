use crate::pv::private::{incref, decref};

#[derive(Copy, Clone)]
struct PvStringData {
    refcount: usize,
    len: usize,
    alloc_size: usize,
}

#[derive(Eq)]
pub struct PvString {
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