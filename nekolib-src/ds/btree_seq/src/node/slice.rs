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

/// # Safety
/// `slice` has at least `idx` elements.
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

/// # Safety
/// `slice` has more than `idx` elements.
pub(super) unsafe fn slice_remove<T>(
    slice: &mut [MaybeUninit<T>],
    idx: usize,
) -> T {
    unsafe {
        let len = slice.len();
        debug_assert!(idx < len);
        let slice_ptr = slice.as_mut_ptr();
        let ret = (*slice_ptr.add(idx)).assume_init_read();
        ptr::copy(slice_ptr.add(idx + 1), slice_ptr.add(idx), len - idx - 1);
        ret
    }
}

/// # Safety
/// `slice` has at least `distance` elements.
pub(super) unsafe fn slice_shl<T>(
    slice: &mut [MaybeUninit<T>],
    distance: usize,
) {
    unsafe {
        let slice_ptr = slice.as_mut_ptr();
        ptr::copy(slice_ptr.add(distance), slice_ptr, slice.len() - distance);
    }
}

/// # Safety
/// `slice` has at least `distance` elements.
pub(super) unsafe fn slice_shr<T>(
    slice: &mut [MaybeUninit<T>],
    distance: usize,
) {
    unsafe {
        let slice_ptr = slice.as_mut_ptr();
        ptr::copy(slice_ptr, slice_ptr.add(distance), slice.len() - distance);
    }
}
