use std::{marker::PhantomData, ptr::NonNull};

pub struct DormantMutRef<'a, T> {
    ptr: NonNull<T>,
    _marker: PhantomData<&'a mut T>,
}
