use std::{mem::MaybeUninit, ptr};

pub(super) fn move_to_slice<T>(
    src: &mut [MaybeUninit<T>],
    dst: &mut [MaybeUninit<T>],
) {
    assert_eq!(src.len(), dst.len());
    unsafe {
        ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }
}
