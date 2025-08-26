pub trait Binom: Sized {
    fn binom(self, k: Self) -> impl Iterator<Item = Self>;
}

macro_rules! impl_uint {
    ( $($ty:ty)* ) => { $(
        impl Binom for $ty {
            fn binom(self, k: $ty) -> impl Iterator<Item = Self> {
                let n = self;
                std::iter::successors(Some(!(!(0 as $ty) << k)), move |&i| {
                    if k == 0 {
                        return None;
                    }
                    let x = i & i.wrapping_neg();
                    let y = i + x;
                    let z = (i & !y) >> (x.trailing_zeros() + 1);
                    Some(y | z)
                })
                .take_while(move |&i| i < (1 << n))
            }
        }
    )* }
}

impl_uint! { u8 u16 u32 u64 u128 usize }

#[test]
fn sanity_check() {
    assert_eq!(5_u32.binom(3).collect::<Vec<_>>(), [
        0b00111, 0b01011, 0b01101, 0b01110, 0b10011, 0b10101, 0b10110, 0b11001,
        0b11010, 0b11100
    ]);
    assert_eq!(0_u32.binom(0).collect::<Vec<_>>(), [0]);
    assert_eq!(1_u32.binom(0).collect::<Vec<_>>(), [0]);
    assert_eq!(1_u32.binom(1).collect::<Vec<_>>(), [1]);
}
