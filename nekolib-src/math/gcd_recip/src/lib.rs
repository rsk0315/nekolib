pub trait GcdRecip: Sized {
    fn gcd_recip(self, other: Self) -> (Self, Self);
}

macro_rules! impl_uint {
    ($t:ty) => {
        impl GcdRecip for $t {
            fn gcd_recip(self, other: Self) -> (Self, Self) {
                assert!(other > 0);
                let a = self % other;
                if a == 0 {
                    return (other, 0);
                }

                let mut s = other;
                let mut t = a;
                let mut m0 = 0;
                let mut m1 = 1;
                while t > 0 {
                    let u = s / t;
                    s -= t * u;
                    // m0 -= m1 * u;
                    let v = (m1 * u) % other;
                    m0 = if m0 < v { m0 + other - v } else { m0 - v };
                    std::mem::swap(&mut s, &mut t);
                    std::mem::swap(&mut m0, &mut m1);
                }
                (s, m0 % (other / s))
            }
        }
    };
    ( $($t:ty)* ) => { $(impl_uint!($t);)* };
}

macro_rules! impl_int {
    ($t:ty) => {
        impl GcdRecip for $t {
            fn gcd_recip(self, other: Self) -> (Self, Self) {
                assert!(other > 0);
                let a = self.rem_euclid(other);
                if a == 0 {
                    return (other, 0);
                }

                let mut s = other;
                let mut t = a;
                let mut m0 = 0;
                let mut m1 = 1;
                while t > 0 {
                    let u = s / t;
                    s -= t * u;
                    m0 -= m1 * u;
                    std::mem::swap(&mut s, &mut t);
                    std::mem::swap(&mut m0, &mut m1);
                }
                if m0 < 0 {
                    m0 += other / s;
                }
                (s, m0)
            }
        }
    };
    ( $($t:ty)* ) => { $(impl_int!($t);)* };
}

impl_uint!(u8 u16 u32 u64 u128 usize);
impl_int!(i8 i16 i32 i64 i128 isize);

#[test]
fn test() {
    for b in 1_i32..=1000 {
        for a in 0..b {
            let (g, r) = a.gcd_recip(b);
            assert!(0 <= r && r < b / g);
            assert_eq!(a * r % b, g % b);
        }
    }
    for b in 1_u32..=1000 {
        for a in 0..b {
            let (g, r) = a.gcd_recip(b);
            assert!(r < b / g);
            assert_eq!(a * r % b, g % b);
        }
    }
}
