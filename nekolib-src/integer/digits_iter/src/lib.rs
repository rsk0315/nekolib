pub trait Digits: Sized {
    fn digits(self, base: Self) -> DigitsIter<Self>;
}

pub struct DigitsIter<I> {
    x: I,
    base: I,
}

impl<I> DigitsIter<I> {
    pub fn new(x: I, base: I) -> Self { Self { x, base } }
}

macro_rules! impl_uint {
    ( $($ty:ty)* ) => { $(
        impl Digits for $ty {
            fn digits(self, base: Self) -> DigitsIter<Self> {
                DigitsIter::new(self, base)
            }
        }
        impl Iterator for DigitsIter<$ty> {
            type Item = $ty;
            fn next(&mut self) -> Option<$ty> {
                if self.x == 0 {
                    return None;
                }

                let res = self.x % self.base;
                self.x /= self.base;
                Some(res)
            }
        }
    )* }
}

impl_uint! { u8 u16 u32 u64 u128 usize }

#[test]
fn sanity_check() {
    assert_eq!(1234_u32.digits(10).collect::<Vec<_>>(), [4, 3, 2, 1]);
    assert_eq!(0o123_u32.digits(8).collect::<Vec<_>>(), [3, 2, 1]);
    assert_eq!(0x3e0f_u32.digits(16).collect::<Vec<_>>(), [15, 0, 14, 3]);
    assert_eq!(0_u32.digits(10).next(), None);
}
