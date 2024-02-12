trait FactorsDup {
    fn factors_dup(self) -> impl Iterator<Item = Self>;
}

macro_rules! impl_uint {
    ( $($ty:ty)* ) => { $(
        impl FactorsDup for $ty {
            fn factors_dup(self) -> impl Iterator<Item = Self> {
                let n = self;
                std::iter::successors(
                    (n >= 1).then_some((1, n)),
                    move |&(i, n)| {
                        if let Some(j) = (i.max(2)..)
                            .take_while(|j| j * j <= n)
                            .find(|j| n % j == 0)
                        {
                            Some((j, n / j))
                        } else if n > 1 {
                            Some((n, 1))
                        } else {
                            None
                        }
                    }
                )
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

    for i in 1..=n_max {
        let actual = i.factors_dup();
        assert!(actual.eq(expected[i].iter().copied()));
    }
}
