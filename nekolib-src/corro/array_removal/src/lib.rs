//! Remove an element from the array.
//!
//! # Examples
//! ```
//! use std::mem::MaybeUninit;
//!
//! use array_removal::array_remove;
//!
//! fn uninit_array<T, const N: usize>() -> [MaybeUninit<T>; N] {
//!     unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() }
//! }
//!
//! let mut array = uninit_array::<String, 10>();
//! array[0].write("A".to_owned());
//! array[1].write("B".to_owned());
//! array[2].write("C".to_owned());
//! array[3].write("X".to_owned());
//! array[4].write("D".to_owned());
//! array[5].write("E".to_owned());
//!
//! unsafe {
//!     let elt = array_remove(&mut array, 3, 6);
//!     assert_eq!(elt, "X");
//!
//!     let init = &*(&array[..5] as *const [_] as *const [String]);
//!     assert_eq!(init, ["A", "B", "C", "D", "E"]);
//!
//!     for e in &mut array[..5] {
//!         e.assume_init_drop();
//!     }
//! }
//! ```

use std::{mem::MaybeUninit, ptr};

/// Remove an element from the array.
///
/// # Safety
/// - `array[..len]` is initialized,
/// - `len <= N`, and
/// - `i < len`.
pub unsafe fn array_remove<T, const N: usize>(
    array: &mut [MaybeUninit<T>; N],
    i: usize,
    len: usize,
) -> T {
    debug_assert!(i < len && len <= N);
    let elt = array[i].assume_init_read();
    let count = len - i - 1;
    let dst = array[i..][..count].as_mut_ptr();
    let src = array[i + 1..][..count].as_ptr();
    ptr::copy(src, dst, count);
    elt
}
