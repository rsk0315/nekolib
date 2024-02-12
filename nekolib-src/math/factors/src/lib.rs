pub trait Factors: Sized {
    fn factors(self) -> impl Iterator<Item = ((Self, u32), Self)>;
}

pub trait FactorsDup: Sized {
    fn factors_dup(self) -> impl Iterator<Item = Self>;
}

macro_rules! impl_uint {
    ( $($ty:ty)* ) => { $(
        impl Factors for $ty {
            fn factors(self) -> impl Iterator<Item = ((Self, u32), Self)> {
                let n = self;
                std::iter::successors(
                    (n >= 1).then_some((((1, 0), 1), n)),
                    move |&(((i, _), _), n)| {
                        if let Some(j) =
                            (i + 1..).take_while(|j| j * j <= n).find(|j| n % j == 0)
                        {
                            let (jj, e) =
                                std::iter::successors(Some((j, 1)), |&(jj, e)| {
                                    (n / jj % j == 0).then(|| (jj * j, e + 1))
                                })
                                .last()
                                .unwrap();
                            Some((((j, e), jj), n / jj))
                        } else if n > 1 {
                            Some((((n, 1), n), 1))
                        } else {
                            None
                        }
                    },
                )
                .skip(1)
                .map(move |(i, _)| i)
            }
        }

        impl FactorsDup for $ty {
            fn factors_dup(self) -> impl Iterator<Item = Self> {
                let n = self;
                std::iter::successors((n >= 1).then_some((1, n)), move |&(i, n)| {
                    if let Some(j) =
                        (i.max(2)..).take_while(|j| j * j <= n).find(|j| n % j == 0)
                    {
                        Some((j, n / j))
                    } else if n > 1 {
                        Some((n, 1))
                    } else {
                        None
                    }
                })
                .skip(1)
                .map(move |(i, _)| i)
            }
        }
    )* };
}

impl_uint! { u8 u16 u32 u64 u128 usize }

#[test]
fn sanity_check() {
    assert!(0_u32.factors_dup().eq(None));

    let n_max = 10000;
    let primes: Vec<_> = {
        let mut is_prime = vec![true; n_max + 1];
        is_prime[0] = false;
        is_prime[1] = false;
        for n in 2..=n_max {
            for i in n..=n_max / n {
                is_prime[i * n] = false;
            }
        }
        (2..=n_max).filter(|&n| is_prime[n]).collect()
    };
    let expected = {
        let mut res = vec![vec![]; n_max + 1];
        for n in 2..=n_max {
            let mut n_ = n;
            for &p in &primes {
                while n_ % p == 0 {
                    n_ /= p;
                    res[n].push(p);
                }
            }
        }
        res
    };

    for n in 1..=n_max {
        let actual = n.factors_dup();
        assert!(actual.eq(expected[n].iter().copied()));

        let expected_dedup = {
            let mut tmp: Vec<_> = n.factors_dup().collect();
            tmp.dedup();
            tmp
        };
        let actual_dedup = n.factors().map(|((p, _), _)| p);
        assert!(actual_dedup.eq(expected_dedup));

        for ((p, e), pp) in n.factors() {
            assert_eq!(n / (pp / p) % p, 0);
            assert_ne!(n / pp % p, 0);
            assert_eq!(pp, p.pow(e));
        }
    }
}
