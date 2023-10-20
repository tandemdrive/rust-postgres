use std::pin::Pin;

pub(crate) struct LazyPin<T> {
    value: Box<T>,
    pinned: bool,
}

impl<T> LazyPin<T> {
    pub fn new(value: T) -> LazyPin<T> {
        LazyPin {
            value: Box::new(value),
            pinned: false,
        }
    }

    pub fn pinned(&mut self) -> Pin<&mut T> {
        self.pinned = true;
        // SAFETY: we have the .pinned boolean to check we are pinned
        #[allow(unsafe_code)]
        unsafe {
            Pin::new_unchecked(&mut *self.value)
        }
    }

    pub fn into_unpinned(self) -> Option<T> {
        if self.pinned {
            None
        } else {
            Some(*self.value)
        }
    }
}
