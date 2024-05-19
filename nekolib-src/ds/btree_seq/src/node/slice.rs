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

pub(super) unsafe fn slice_insert<T>(
    slice: &mut [MaybeUninit<T>],
    idx: usize,
    val: T,
) {
    unsafe {
        let len = slice.len();
        debug_assert!(idx < len);
        let slice_ptr = slice.as_mut_ptr();
        if idx + 1 < len {
            ptr::copy(
                slice_ptr.add(idx),
                slice_ptr.add(idx + 1),
                len - idx - 1,
            );
        }
        (*slice_ptr.add(idx)).write(val);
    }
}
