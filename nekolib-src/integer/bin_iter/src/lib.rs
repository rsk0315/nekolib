//! 2 進法のイテレータ。

/// 2 進法のイテレータ。
///
/// ```
/// # use bin_iter::BinIter;
/// assert!(0_u32.bin_iter().eq([]));
/// assert!(1_u32.bin_iter().map(u32::from).eq([1]));
/// assert!(0b_1001011_u64.bin_iter().map(u32::from).eq([1, 1, 0, 1, 0, 0, 1]));
/// assert!((!0_u128).bin_iter().map(u32::from).eq([1; 128]));
/// ```
pub trait BinIter {
    type Iter: Iterator<Item = bool>;
    fn bin_iter(&self) -> Self::Iter;
}

pub struct UIntIter<U>(U);

pub trait Binary {
    fn pop(&mut self) -> Option<bool>;
}

macro_rules! impl_binary {
    ( $($ty:ty)* ) => { $(
        impl Binary for $ty {
            fn pop(&mut self) -> Option<bool> {
                if *self == 0 {
                    None
                } else {
                    let tmp = *self & 1 != 0;
                    *self >>= 1;
                    Some(tmp)
                }
            }
        }
    )* }
}

impl_binary! { u8 u16 u32 u64 u128 usize }

impl<U: Binary> UIntIter<U> {
    pub fn new(u: U) -> Self { Self(u) }
}

impl<U: Binary> Iterator for UIntIter<U> {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> { self.0.pop() }
}

macro_rules! impl_bin_iter {
    ( $($ty:ty)* ) => { $(
        impl BinIter for $ty {
            type Iter = UIntIter<$ty>;
            fn bin_iter(&self) -> Self::Iter { Self::Iter::new(*self) }
        }
    )* }
}

impl_bin_iter! { u8 u16 u32 u64 u128 usize }

#[test]
fn sanity_check() {
    assert!(0_u32.bin_iter().eq([]));
    assert!(1_u32.bin_iter().map(u32::from).eq([1]));
    assert!(0b_1001011_u64.bin_iter().map(u32::from).eq([1, 1, 0, 1, 0, 0, 1]));
    assert!((!0_u128).bin_iter().map(u32::from).eq([1; 128]));
}
