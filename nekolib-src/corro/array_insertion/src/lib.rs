//! Insert an element into the array.
//!
//! # Examples
//! ```
//! use std::mem::MaybeUninit;
//!
//! use array_insertion::array_insert;
//!
//! fn uninit_array<T, const N: usize>() -> [MaybeUninit<T>; N] {
//!     unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() }
//! }
//!
//! let mut array = uninit_array::<String, 10>();
//! array[0].write("A".to_owned());
//! array[1].write("B".to_owned());
//! array[2].write("C".to_owned());
//! array[3].write("E".to_owned());
//! array[4].write("F".to_owned());
//!
//! unsafe {
//!     array_insert(&mut array, 3, 5, "D".to_owned());
//!
//!     let init = &*(&array[..6] as *const [_] as *const [String]);
//!     assert_eq!(init, ["A", "B", "C", "D", "E", "F"]);
//!
//!     for e in &mut array[..6] {
//!         e.assume_init_drop();
//!     }
//! }
//! ```
//!
//! ```
//! use std::mem::MaybeUninit;
//!
//! use array_insertion::array_splice;
//!
//! fn uninit_array<T, const N: usize>() -> [MaybeUninit<T>; N] {
//!     unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() }
//! }
//!
//! let mut dst = uninit_array::<String, 10>();
//! dst[0].write("A".to_owned());
//! dst[1].write("B".to_owned());
//! dst[2].write("C".to_owned());
//! dst[3].write("F".to_owned());
//! dst[4].write("G".to_owned());
//! let mut src = uninit_array::<String, 10>();
//! src[0].write("D".to_owned());
//! src[1].write("E".to_owned());
//!
//! unsafe {
//!     array_splice(&mut dst, 3, 5, &src, 2);
//!
//!     let init = &*(&dst[..7] as *const [_] as *const [String]);
//!     assert_eq!(init, ["A", "B", "C", "D", "E", "F", "G"]);
//!
//!     for e in &mut dst[..7] {
//!         e.assume_init_drop();
//!     }
//! }
//! ```

use std::{mem::MaybeUninit, ptr};

/// Insert an element into the array.
///
/// # Safety
/// - `array[..len]` is initialized,
/// - `array[len..]` is uninitialized,
/// - `len < N`, and
/// - `i <= len`.
pub unsafe fn array_insert<T, const N: usize>(
    array: &mut [MaybeUninit<T>; N],
    i: usize,
    len: usize,
    elt: T,
) {
    debug_assert!(i <= len && len < N);
    let count = len - i;
    let dst = array[i + 1..][..count].as_mut_ptr();
    // `src` should be after `dst` for Stacked Borrows.
    let src = array[i..][..count].as_ptr();
    ptr::copy(src, dst, count);
    array[i].write(elt);
}

pub unsafe fn array_splice<T, const N: usize>(
    dst: &mut [MaybeUninit<T>; N],
    i: usize,
    dst_len: usize,
    src: &[MaybeUninit<T>; N],
    src_len: usize,
) {
    debug_assert!(i <= dst_len && dst_len + src_len <= N);
    let count = dst_len - i;
    let dst_ptr = dst[i + src_len..][..count].as_mut_ptr();
    let src_ptr = dst[i..][..count].as_ptr();
    ptr::copy(src_ptr, dst_ptr, count);
    let count = src_len;
    let src_ptr = src[..count].as_ptr();
    let dst_ptr = dst[i..][..count].as_mut_ptr();
    ptr::copy_nonoverlapping(src_ptr, dst_ptr, count);
}
