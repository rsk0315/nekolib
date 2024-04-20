//! Rotate arrays and elements.
//!
//! # Examples
//! ```
//! use std::mem::MaybeUninit;
//!
//! use slice_rotation::rotate_2;
//!
//! fn uninit_array<T, const N: usize>() -> [MaybeUninit<T>; N] {
//!     unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() }
//! }
//!
//! let mut left = uninit_array::<String, 10>();
//! left[0].write("A".to_owned());
//! left[1].write("B".to_owned());
//! left[2].write("C".to_owned());
//! left[3].write("D".to_owned());
//! let mut right = uninit_array::<String, 10>();
//! right[0].write("E".to_owned());
//!
//! let leftlen_old = 4;
//! let rightlen_old = 1;
//! let leftlen_new = 2;
//! let rightlen_new = unsafe {
//!     rotate_2(&mut left, &mut right, leftlen_old, rightlen_old, leftlen_new)
//! };
//!
//! assert_eq!(rightlen_new, 3);
//! unsafe {
//!     let left = &*(&left as *const [_] as *const [String]);
//!     assert_eq!(left[..leftlen_new], ["A", "B"]);
//!     let right = &*(&right as *const [_] as *const [String]);
//!     assert_eq!(right[..rightlen_new], ["C", "D", "E"]);
//! }
//!
//! unsafe {
//!     rotate_2(&mut left, &mut right, leftlen_new, rightlen_new, 0);
//!     for e in &mut right[..leftlen_new + rightlen_new] {
//!         e.assume_init_drop();
//!     }
//! }
//! ```
//!
//! ```
//! use std::mem::MaybeUninit;
//!
//! use slice_rotation::rotate_3;
//!
//! fn uninit_array<T, const N: usize>() -> [MaybeUninit<T>; N] {
//!     unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() }
//! }
//!
//! let mut left = uninit_array::<String, 10>();
//! left[0].write("A".to_owned());
//! left[1].write("B".to_owned());
//! left[2].write("C".to_owned());
//! left[3].write("D".to_owned());
//! let mut mid = MaybeUninit::new("E".to_owned());
//! let mut right = uninit_array::<String, 10>();
//! right[0].write("F".to_owned());
//!
//! let leftlen_old = 4;
//! let rightlen_old = 1;
//! let leftlen_new = 2;
//! let rightlen_new = unsafe {
//!     rotate_3(&mut left, &mut mid, &mut right, leftlen_old, rightlen_old, leftlen_new)
//! };
//!
//! assert_eq!(rightlen_new, 3);
//! unsafe {
//!     let left = &*(&left as *const [_] as *const [String]);
//!     assert_eq!(left[..leftlen_new], ["A", "B"]);
//!     assert_eq!(mid.assume_init_ref(), "C");
//!     let right = &*(&right as *const [_] as *const [String]);
//!     assert_eq!(right[..rightlen_new], ["D", "E", "F"]);
//! }
//!
//! unsafe {
//!     rotate_3(&mut left, &mut mid, &mut right, leftlen_new, rightlen_new, 0);
//!     mid.assume_init_drop();
//!     for e in &mut right[..leftlen_new + rightlen_new] {
//!         e.assume_init_drop();
//!     }
//! }
//! ```

use std::{mem::MaybeUninit, ptr};

/// Rotate two arrays.
///
/// # Safety
/// - `(left|right)len_(old|new) in 0..=N`,
/// - `left[..leftlen_old]` is initialized,
/// - `left[leftlen_old..]` is uninitialized,
/// - `right[..rightlen_old]` is initialized, and
/// - `right[rightlen_old..]` is uninitialized.
pub unsafe fn rotate_2<T, const N: usize>(
    left: &mut [MaybeUninit<T>; N],
    right: &mut [MaybeUninit<T>; N],
    leftlen_old: usize,
    rightlen_old: usize,
    leftlen_new: usize,
) -> usize {
    debug_assert!(leftlen_old <= N && rightlen_old <= N);
    debug_assert!(leftlen_new <= N);
    debug_assert!(leftlen_new <= leftlen_old + rightlen_old);
    let rightlen_new = leftlen_old + rightlen_old - leftlen_new;
    debug_assert!(rightlen_new <= N);
    if leftlen_old < leftlen_new {
        // [A, B, C] ++ [D, E, F, G, H, I]
        let src = ptr::addr_of!(right[0]);
        let dst = ptr::addr_of_mut!(left[leftlen_old]);
        let left_diff = leftlen_new - leftlen_old;
        ptr::copy_nonoverlapping(src, dst, left_diff);
        // [A, B, C, D, E] ++ [_, _, F, G, H, I]
        let src = ptr::addr_of!(right[left_diff]);
        let dst = ptr::addr_of_mut!(right[0]);
        ptr::copy(src, dst, rightlen_new);
        // [A, B, C, D, E] ++ [F, G, H, I]
    } else if leftlen_old > leftlen_new {
        // [A, B, C, D, E, F] ++ [G, H, I]
        let right_diff = rightlen_new - rightlen_old;
        let src = ptr::addr_of!(right[0]);
        let dst = ptr::addr_of_mut!(right[right_diff]);
        eprintln!("right_diff: {right_diff}");
        eprintln!("copy({src:?}, {dst:?}, {rightlen_old})");
        ptr::copy(src, dst, rightlen_old);
        // [A, B, C, D, E, F] ++ [_, _, G, H, I]
        let src = ptr::addr_of!(left[leftlen_new]);
        let dst = ptr::addr_of_mut!(right[0]);
        eprintln!("copy_nonoverlapping({src:?}, {dst:?}, {right_diff})");
        ptr::copy_nonoverlapping(src, dst, right_diff);
        // [A, B, C, D] ++ [E, F, G, H, I]
    }
    rightlen_new
}

/// Rotate two arrays and one element.
///
/// # Safety
/// - `(left|right)len_(old|new) in 0..=N`,
/// - `left[..leftlen_old]` is initialized,
/// - `left[leftlen_old..]` is uninitialized,
/// - `right[..rightlen_old]` is initialized,
/// - `right[rightlen_old..]` is uninitialized, and
/// - `mid` is initialized.
pub unsafe fn rotate_3<T, const N: usize>(
    left: &mut [MaybeUninit<T>; N],
    mid: &mut MaybeUninit<T>,
    right: &mut [MaybeUninit<T>; N],
    leftlen_old: usize,
    rightlen_old: usize,
    leftlen_new: usize,
) -> usize {
    debug_assert!(leftlen_old <= N && rightlen_old <= N);
    debug_assert!(leftlen_new <= N);
    debug_assert!(leftlen_new <= leftlen_old + rightlen_old);
    let rightlen_new = leftlen_old + rightlen_old - leftlen_new;
    debug_assert!(rightlen_new <= N);
    if leftlen_old < leftlen_new {
        let mid_elt = unsafe { mid.assume_init_read() };
        // [A, B, C] ++ [D] ++ [E, F, G, H, I, J, K, L]
        left[leftlen_old].write(mid_elt);
        // [A, B, C, D] ++ [_] ++ [E, F, G, H, I, J, K, L]
        let src = ptr::addr_of!(right[0]);
        let dst = ptr::addr_of_mut!(left[leftlen_old + 1]);
        let left_diff = leftlen_new - leftlen_old;
        ptr::copy_nonoverlapping(src, dst, left_diff - 1);
        // [A, B, C, D, E, F] ++ [_] ++ [_, _, G, H, I, J, K, L]
        let new_mid_elt = unsafe { right[left_diff - 1].assume_init_read() };
        mid.write(new_mid_elt);
        // [A, B, C, D, E, F] ++ [G] ++ [_, _, _, H, I, J, K, L]
        let src = ptr::addr_of!(right[left_diff]);
        let dst = ptr::addr_of_mut!(right[0]);
        ptr::copy(src, dst, rightlen_new);
        // [A, B, C, D, E, F] ++ [G] ++ [H, I, J, K, L]
    } else if leftlen_old > leftlen_new {
        let mid_elt = unsafe { mid.assume_init_read() };
        // [A, B, C, D, E, F, G, H] ++ [I] ++ [J, K, L]
        let right_diff = rightlen_new - rightlen_old;
        let src = ptr::addr_of!(right[0]);
        let dst = ptr::addr_of_mut!(right[right_diff]);
        ptr::copy(src, dst, rightlen_old);
        // [A, B, C, D, E, F, G, H] ++ [I] ++ [_, _, _, J, K, L]
        right[right_diff - 1].write(mid_elt);
        // [A, B, C, D, E, F, G, H] ++ [_] ++ [_, _, I, J, K, L]
        let src = ptr::addr_of!(left[leftlen_new + 1]);
        let dst = ptr::addr_of_mut!(right[0]);
        ptr::copy_nonoverlapping(src, dst, right_diff - 1);
        // [A, B, C, D, E, F] ++ [_] ++ [G, H, I, J, K, L]
        let new_mid_elt = unsafe { left[leftlen_new].assume_init_read() };
        mid.write(new_mid_elt);
        // [A, B, C, D, E] ++ [F] ++ [G, H, I, J, K, L]
    }
    rightlen_new
}

#[cfg(test)]
mod tests {
    use std::mem::MaybeUninit;

    use super::*;

    #[cfg(test)]
    fn uninit_array<T, const N: usize>() -> [MaybeUninit<T>; N] {
        // polyfill of `std::mem::MaybeUninit::transpose()`
        unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() }
    }

    #[cfg(test)]
    unsafe fn assume_init_collect<'a, I, E, O>(iter: I) -> O
    where
        I: IntoIterator<Item = &'a MaybeUninit<E>>,
        E: 'a + Clone,
        O: FromIterator<E>,
    {
        iter.into_iter()
            .map(|e| unsafe { e.assume_init_ref() }.clone())
            .collect()
    }

    #[test]
    fn sanity_check_2_few() {
        let mut left = uninit_array::<String, 10>();
        left[0].write("A".to_owned());
        left[1].write("B".to_owned());
        left[2].write("C".to_owned());
        left[3].write("D".to_owned());
        left[4].write("E".to_owned());
        left[5].write("F".to_owned());
        left[6].write("G".to_owned());
        left[7].write("H".to_owned());
        left[8].write("I".to_owned());
        let mut right = uninit_array::<String, 10>();

        let leftlen_old = 9;
        let rightlen_old = 0;

        let expected: String =
            unsafe { assume_init_collect(&left[..leftlen_old]) };

        let (mut leftlen_old, mut rightlen_old) = (leftlen_old, rightlen_old);
        for &leftlen_new in &[3, 7, 9, 0, 2, 1, 5, 5, 8, 6] {
            let rightlen_new = unsafe {
                rotate_2(
                    &mut left,
                    &mut right,
                    leftlen_old,
                    rightlen_old,
                    leftlen_new,
                )
            };

            let left: String =
                unsafe { assume_init_collect(&left[..leftlen_new]) };
            let right: String =
                unsafe { assume_init_collect(&right[..rightlen_new]) };
            assert_eq!(left, &expected[..leftlen_new]);
            assert_eq!(right, &expected[leftlen_new..]);

            leftlen_old = leftlen_new;
            rightlen_old = rightlen_new;
        }

        unsafe {
            for e in &mut left[..leftlen_old] {
                e.assume_init_drop();
            }
            for e in &mut right[..rightlen_old] {
                e.assume_init_drop();
            }
        }
    }

    #[test]
    fn sanity_check_2_many() {
        let mut left = uninit_array::<String, 10>();
        left[0].write("A".to_owned());
        left[1].write("B".to_owned());
        left[2].write("C".to_owned());
        left[3].write("D".to_owned());
        left[4].write("E".to_owned());
        left[5].write("F".to_owned());
        left[6].write("G".to_owned());
        left[7].write("H".to_owned());
        left[8].write("I".to_owned());
        let mut right = uninit_array::<String, 10>();
        right[0].write("J".to_owned());
        right[1].write("K".to_owned());
        right[2].write("L".to_owned());
        right[3].write("M".to_owned());
        right[4].write("N".to_owned());
        right[5].write("O".to_owned());

        let leftlen_old = 9;
        let rightlen_old = 6;

        let expected = {
            let left: String =
                unsafe { assume_init_collect(&left[..leftlen_old]) };
            let right: String =
                unsafe { assume_init_collect(&right[..rightlen_old]) };
            left + &right
        };

        let (mut leftlen_old, mut rightlen_old) = (leftlen_old, rightlen_old);
        for &leftlen_new in &[5, 9, 7, 8, 6, 8, 8] {
            let rightlen_new = unsafe {
                rotate_2(
                    &mut left,
                    &mut right,
                    leftlen_old,
                    rightlen_old,
                    leftlen_new,
                )
            };

            let left: String =
                unsafe { assume_init_collect(&left[..leftlen_new]) };
            let right: String =
                unsafe { assume_init_collect(&right[..rightlen_new]) };
            assert_eq!(left, &expected[..leftlen_new]);
            assert_eq!(right, &expected[leftlen_new..]);

            leftlen_old = leftlen_new;
            rightlen_old = rightlen_new;
        }

        unsafe {
            for e in &mut left[..leftlen_old] {
                e.assume_init_drop();
            }
            for e in &mut right[..rightlen_old] {
                e.assume_init_drop();
            }
        }
    }

    #[test]
    fn sanity_check_3_few() {
        let mut left = uninit_array::<String, 10>();
        left[0].write("A".to_owned());
        left[1].write("B".to_owned());
        left[2].write("C".to_owned());
        left[3].write("D".to_owned());
        left[4].write("E".to_owned());
        left[5].write("F".to_owned());
        left[6].write("G".to_owned());
        left[7].write("H".to_owned());
        left[8].write("I".to_owned());
        let mut mid = MaybeUninit::new("J".to_owned());
        let mut right = uninit_array::<String, 10>();

        let leftlen_old = 9;
        let rightlen_old = 0;

        let expected = {
            let left: String =
                unsafe { assume_init_collect(&left[..leftlen_old]) };
            left + unsafe { &mid.assume_init_ref() }
        };

        let (mut leftlen_old, mut rightlen_old) = (leftlen_old, rightlen_old);
        for &leftlen_new in &[3, 7, 9, 0, 2, 1, 5, 5, 8, 6] {
            let rightlen_new = unsafe {
                rotate_3(
                    &mut left,
                    &mut mid,
                    &mut right,
                    leftlen_old,
                    rightlen_old,
                    leftlen_new,
                )
            };

            let left: String =
                unsafe { assume_init_collect(&left[..leftlen_new]) };
            let mid = unsafe { mid.assume_init_ref().clone() };
            let right: String =
                unsafe { assume_init_collect(&right[..rightlen_new]) };
            assert_eq!(left, &expected[..leftlen_new]);
            assert_eq!(mid, &expected[leftlen_new..leftlen_new + 1]);
            assert_eq!(right, &expected[leftlen_new + 1..]);

            leftlen_old = leftlen_new;
            rightlen_old = rightlen_new;
        }

        unsafe {
            for e in &mut left[..leftlen_old] {
                e.assume_init_drop();
            }
            for e in &mut right[..rightlen_old] {
                e.assume_init_drop();
            }
            mid.assume_init_drop();
        }
    }

    #[test]
    fn sanity_check_3_many() {
        let mut left = uninit_array::<String, 10>();
        left[0].write("A".to_owned());
        left[1].write("B".to_owned());
        left[2].write("C".to_owned());
        left[3].write("D".to_owned());
        left[4].write("E".to_owned());
        left[5].write("F".to_owned());
        left[6].write("G".to_owned());
        left[7].write("H".to_owned());
        left[8].write("I".to_owned());
        let mut mid = MaybeUninit::new("J".to_owned());
        let mut right = uninit_array::<String, 10>();
        right[0].write("K".to_owned());
        right[1].write("L".to_owned());
        right[2].write("M".to_owned());
        right[3].write("N".to_owned());
        right[4].write("O".to_owned());
        right[5].write("P".to_owned());

        let leftlen_old = 9;
        let rightlen_old = 6;

        let expected = {
            let left: String =
                unsafe { assume_init_collect(&left[..leftlen_old]) };
            let right: String =
                unsafe { assume_init_collect(&right[..rightlen_old]) };
            left + unsafe { &mid.assume_init_ref() } + &right
        };

        let (mut leftlen_old, mut rightlen_old) = (leftlen_old, rightlen_old);
        for &leftlen_new in &[7, 9, 9, 8, 7, 8, 6, 8] {
            let rightlen_new = unsafe {
                rotate_3(
                    &mut left,
                    &mut mid,
                    &mut right,
                    leftlen_old,
                    rightlen_old,
                    leftlen_new,
                )
            };

            let left: String =
                unsafe { assume_init_collect(&left[..leftlen_new]) };
            let mid = unsafe { mid.assume_init_ref().clone() };
            let right: String =
                unsafe { assume_init_collect(&right[..rightlen_new]) };
            assert_eq!(left, &expected[..leftlen_new]);
            assert_eq!(mid, &expected[leftlen_new..leftlen_new + 1]);
            assert_eq!(right, &expected[leftlen_new + 1..]);

            leftlen_old = leftlen_new;
            rightlen_old = rightlen_new;
        }

        unsafe {
            for e in &mut left[..leftlen_old] {
                e.assume_init_drop();
            }
            for e in &mut right[..rightlen_old] {
                e.assume_init_drop();
            }
            mid.assume_init_drop();
        }
    }
}
