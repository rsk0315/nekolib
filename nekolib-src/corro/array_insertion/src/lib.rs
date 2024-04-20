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
