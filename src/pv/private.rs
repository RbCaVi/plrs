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

impl<T> From<T> for PvFixedSize<T> {
    fn from(value: T) -> Self {
        PvFixedSize::<T>::new(value)
    }
}