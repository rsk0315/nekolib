pub trait Gcd {
    fn gcd(self, other: Self) -> Self;
}

macro_rules! impl_uint {
    ( $($ty:ty)* ) => { $(
        impl Gcd for $ty {
            fn gcd(mut self, mut other: Self) -> Self {
                while other != 0 {
                    let tmp = self % other;
                    self = std::mem::replace(&mut other, tmp);
                }
                self
            }
        }
    )* }
}

impl_uint! { u8 u16 u32 u64 u128 usize }

#[test]
fn sanity_check() {
    assert_eq!(24_u32.gcd(16), 8);
    assert_eq!(0_u32.gcd(0), 0);
}
