pub trait Divisors: Sized {
    fn divisors(self) -> impl Iterator<Item = Self>;
}

macro_rules! impl_uint {
    ( $($ty:ty)* ) => { $(
        impl Divisors for $ty {
            fn divisors(self) -> impl Iterator<Item = Self> {
                let n = self;
                std::iter::successors(
                    (n >= 1).then_some((1, true)),
                    move |&(i, asc)| {
                        if asc {
                            if let Some(j) = (i + 1..)
                                .take_while(|j| j * j <= n)
                                .find(|j| n % j == 0)
                            {
                                return Some((j, true));
                            } else if n / i != i {
                                return Some((i, false));
                            }
                        }
                        let j = (1..i).rev().find(|&j| n % j == 0)?;
                        Some((j, false))
                    },
                )
                .map(move |(i, asc)| if asc { i } else { n / i })
            }
        }
    )* };
}

impl_uint! { u8 u16 u32 u64 u128 usize }

#[test]
fn sanity_check() {
    assert!(0_u32.divisors().eq(None));

    for n in 1_u32..=10000 {
        let expected = (1..=n).filter(|i| n % i == 0);
        let actual = n.divisors();
        assert!(actual.eq(expected));
    }
}
