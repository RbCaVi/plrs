macro_rules! incref {
    ($pv:expr) => (unsafe {(*$pv.data).refcount += 1})
}

macro_rules! decref {
    ($pv:expr) => (unsafe {
        (*$pv.data).refcount -= 1;
        (*$pv.data).refcount
    } == 0)
}

pub(crate) use incref;
pub(crate) use decref;

struct PvFixedSizeData<T> {
    refcount: usize,
    data: T,
}

pub struct PvFixedSize<T> {
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
    pub fn get_data_mut(&self) -> &mut T {
        unsafe {&mut (*self.data).data}
    }

    // get an immutable reference to the array data
    // only use when refcount = 1
    pub fn get_data(&self) -> &T {
        self.get_data_mut()
    }
}

impl<T: Clone> PvFixedSize<T> {
    // move one copy of this value out
    // will reuse the old allocation if possible
    pub fn move_out(self) -> Self {
        if unsafe {(*self.data).refcount} == 1 {
            self
        } else {
            PvFixedSize::<T>::new(self.get_data().clone())
        }
    }
}

impl<T: Copy> PvFixedSize<T> {
    // move one copy of this value out
    // will reuse the old allocation if possible
    pub fn copy_out(self) -> Self {
        if unsafe {(*self.data).refcount} == 1 {
            self
        } else {
            PvFixedSize::<T>::new(*self.get_data())
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

impl<T> From<T> for PvFixedSize<T> {
    fn from(value: T) -> Self {
        PvFixedSize::<T>::new(value)
    }
}

#[derive(Copy, Clone)]
struct PvArrayData {
    refcount: usize,
    len: usize,
    alloc_size: usize,
}

#[derive(Eq)]
pub struct PvpArray<T> {
    data: *mut PvArrayData,
    _data: std::marker::PhantomData<[T]>,
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

impl<T> PvpArray<T> {
    // get a Layout that fits a PvArrayData with `size` T's after it
    fn get_layout(size: usize) -> std::alloc::Layout {
        let mut layout = std::alloc::Layout::new::<PvArrayData>();
        // i'm not assuming nightly rust with std::alloc::Layout::repeat()
        for _ in 0..size {
            // no errors need to be handled
            // the total size does not overflow isize (i'm assuming this one)
            (layout, _) = layout.extend(
                std::alloc::Layout::new::<T>()
            ).unwrap();
        }
        layout
    }

    // allocates enough space for `len` array elements
    pub fn new_empty_sized(size: usize) -> Self {
        let layout = PvpArray::<T>::get_layout(size);

        let data = unsafe {std::alloc::alloc(layout)} as *mut PvArrayData;

        unsafe {*data = PvArrayData {refcount: 1, len: 0, alloc_size: size};}

        PvpArray::<T> {data, _data: std::marker::PhantomData}
    }

    pub fn new_empty() -> Self {
        PvpArray::<T>::new_empty_sized(16) // any size would work
    }

    // get a mutable slice reference to the array data
    // only use when refcount = 1
    fn get_data_mut(&self) -> &mut [std::mem::MaybeUninit<T>] {
        let data = unsafe {*self.data};

        let layout = std::alloc::Layout::new::<PvArrayData>();
        // no errors need to be handled
        // the total size does not overflow isize (i'm assuming this one)
        let (_, offset) = layout.extend(
            std::alloc::Layout::new::<T>()
        ).unwrap();
        // i could probably just get the size of layout
        // but just to be safe :)

        unsafe {
            std::slice::from_raw_parts_mut(
                (self.data as *mut u8).add(offset) as *mut std::mem::MaybeUninit<T>,
                data.len
            )
        }
    }

    // get an immutable reference to the array data
    // only use when refcount = 1
    fn get_data(&self) -> &[T] {
        unsafe {std::mem::transmute::<_, _>(self.get_data_mut())}
    }

    pub fn len(&self) -> usize {
        unsafe {*self.data}.len
    }
}

impl<T: Clone> PvpArray<T> {
    pub fn new(pvs: &[T]) -> Self {
        let out = PvpArray::<T>::new_empty_sized(pvs.len() * 2); // any >= str.len()
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

            let oldlayout = PvpArray::<T>::get_layout(data.alloc_size);

            let newlayout = PvpArray::<T>::get_layout(newsize);
            
            self.data = std::alloc::realloc(self.data as *mut u8, oldlayout, newlayout.size()) as *mut PvArrayData;

            (*self.data).alloc_size = newsize;

            self
        } else {
            let out = PvpArray::<T>::new_empty_sized(newsize);
            (*out.data).len = data.len;
            clone_to_uninit!(self.get_data(), out.get_data_mut(), 0, data.len);
            out
        }
    }

    pub fn append(self, other: T) -> Self {
        let data = unsafe {*self.data};

        let s = if data.refcount == 1 && data.alloc_size >= data.len + 1 {
            self
        } else {unsafe {
            self.resize_move((data.len + 1) * 2)
        }};

        unsafe {
            (*s.data).len += 1;
        }

        s.get_data_mut()[data.len].write(other);

        s
    }

    pub fn concat(self, other: &PvpArray<T>) -> Self {
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

    pub fn pop(self) -> Self {
        let data = unsafe {*self.data};

        let s = if data.refcount == 1 {
            self
        } else {unsafe {
            self.resize_move(data.alloc_size)
        }};


        unsafe {
            // drop the last element
            s.get_data_mut()[data.len - 1].assume_init_drop();
            (*s.data).len -= 1;
        }

        s
    }

    pub fn popn(self, n: usize) -> Self {
        let data = unsafe {*self.data};

        let s = if data.refcount == 1 {
            self
        } else {unsafe {
            self.resize_move(data.alloc_size)
        }};

        unsafe {
            // drop the last n elements
            for i in data.len - n..data.len {
                s.get_data_mut()[i].assume_init_drop();
            }

            (*s.data).len -= n;
        }

        s
    }

    pub fn get(&self, i: usize) -> T {
        self.get_data()[i].clone()
    }
}

impl<T: std::fmt::Debug> PvpArray<T> {
    // for Debug implementation
    pub fn fmt(&self, f: &mut std::fmt::Formatter<'_>, typename: &str) -> std::fmt::Result {
        let data = unsafe {*self.data};
        f.debug_struct(typename)
         .field("len", &data.len)
         .field("alloc_size", &data.alloc_size)
         .field("data", &self.get_data())
         .finish()
    }
}

impl<T> Drop for PvpArray<T> {
    fn drop(&mut self) {
        if (decref!(self)) {
            let size = unsafe {(*self.data).alloc_size};
            let layout = PvpArray::<T>::get_layout(size);

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

impl<T> Clone for PvpArray<T> {
    fn clone(&self) -> Self {
        incref!(self);
        PvpArray {data: self.data, _data: std::marker::PhantomData}
    }
}

impl<T: PartialEq> PartialEq for PvpArray<T> {
    fn eq(&self, other: &PvpArray<T>) -> bool {
        self.get_data() == other.get_data()
    }
}

impl<T: std::hash::Hash> std::hash::Hash for PvpArray<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_data().hash(state);
    }
}

impl<T: Clone> From<&[T]> for PvpArray<T> {
    fn from(value: &[T]) -> Self {
        PvpArray::new(value)
    }
}