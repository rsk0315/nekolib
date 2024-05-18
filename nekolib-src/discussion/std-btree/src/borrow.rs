//! [`DormantMutRef`] の定義。

use std::{marker::PhantomData, ptr::NonNull};

pub struct DormantMutRef<'a, T> {
    ptr: NonNull<T>,
    _marker: PhantomData<&'a mut T>,
}

unsafe impl<'a, T> Sync for DormantMutRef<'a, T> where &'a mut T: Sync {}
unsafe impl<'a, T> Send for DormantMutRef<'a, T> where &'a mut T: Send {}

impl<'a, T> DormantMutRef<'a, T> {
    pub fn new(t: &'a mut T) -> (&'a mut T, Self) {
        let ptr = NonNull::from(t);
        let new_ref = unsafe { &mut *ptr.as_ptr() };
        (new_ref, Self { ptr, _marker: PhantomData })
    }
    pub unsafe fn awaken(self) -> &'a mut T {
        unsafe { &mut *self.ptr.as_ptr() }
    }
    pub unsafe fn reborrow(&mut self) -> &'a mut T {
        unsafe { &mut *self.ptr.as_ptr() }
    }
    pub unsafe fn reborrow_shared(&self) -> &'a T {
        unsafe { &*self.ptr.as_ptr() }
    }
}
