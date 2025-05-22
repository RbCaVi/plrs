use crate::pv::private::{incref, decref};

use crate::pv::Pv;

#[derive(Copy, Clone)]
struct PvArrayData {
    refcount: usize,
    len: usize,
    alloc_size: usize,
}

#[derive(Eq)]
pub struct PvArray {
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