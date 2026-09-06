// https://maspypy.com/o1-mod-inv-mod-pow
// https://rsk0315.hatenablog.com/entry/2023/04/29/043512

use bsgs::Bsgs;
use linear_sieve::LinearSieve;
use modint::ModInt;

const B: usize = 65536; // FIXME

pub struct RatConvertQuery<M: ModInt> {
    n: usize,
    bin: Vec<(M, M)>,
    prev: Vec<usize>,
    next: Vec<usize>,
}

pub struct RatConvertRecipLogPowQuery<M: ModInt> {
    rcq: RatConvertQuery<M>,
    recip: Vec<M>,
    log: Vec<M>,
    giant_pow: Vec<M>,
    baby_pow: Vec<M>,
}

impl<M: ModInt> RatConvertQuery<M> {
    pub fn new(n: usize) -> Self {
        const { assert!(M::IS_PRIME) };

        let na = (M::new(0), M::new(0));
        let mut bin = vec![na; n * n + 1];
        let nn = (n * n) as u128;
        for num in (1..=n).rev() {
            for den in num..=n {
                let i = (nn * num as u128 / den as u128) as usize;
                bin[i] = (M::new(num), M::new(den));
            }
        }
        bin[0] = (M::new(0), M::new(1));

        let npos = usize::MAX;
        let mut prev = vec![npos; n * n + 1];
        let mut last = 0;
        for i in 1..=n * n {
            prev[i] = last;
            if bin[i] != na {
                last = i;
            }
        }
        let mut next = vec![npos; n * n + 1];
        let mut last = n * n;
        for i in (0..n * n).rev() {
            next[i] = last;
            if bin[i] != na {
                last = i;
            }
        }

        Self { n, bin, prev, next }
    }

    pub fn rat_convert(&self, x: M) -> ((bool, M), M) {
        let x_u32 = x.get();
        if x_u32 == 0 || x_u32 == 1 {
            return ((false, x), M::new(1));
        }

        let n = self.n;
        let nn = (n * n) as u128;
        let p = M::MOD;
        let i = (nn * x_u32 as u128 / p as u128) as usize;

        let (c, b) = self.bin[i];
        let (c_u32, b_u32) = (c.get(), b.get());
        let ((c1, b1), (c2, b2)) = if (c_u32, b_u32) == (0, 0) {
            (self.bin[self.prev[i]], self.bin[self.next[i]])
        } else if c_u32 as u64 * p as u64 <= b_u32 as u64 * x_u32 as u64 {
            // c/b <= x/p
            ((c, b), self.bin[self.next[i]])
        } else {
            (self.bin[self.prev[i]], (c, b))
        };
        let (b, c) = if b1.get() < b2.get() { (b1, c1) } else { (b2, c2) };
        let (b_u32, c_u32) = (b.get(), c.get());
        let a = b * x;
        let a_is_neg =
            (b_u32 as u64 * x_u32 as u64) < (c_u32 as u64 * p as u64);
        ((a_is_neg, a), b)
    }
}

fn ceil_cbrt(n: usize) -> usize {
    (0_usize..).find(|&i| i.pow(3) >= n).unwrap()
}

impl<M: ModInt> RatConvertRecipLogPowQuery<M> {
    pub fn new(r: M) -> Self {
        let n = ceil_cbrt(M::MOD as usize);
        // r must be a primitive root of M::MOD

        let rcq = RatConvertQuery::new(n);
        let recip = M::recip_table(2 * M::MOD as usize / n);
        let log = Self::log_table(2 * M::MOD as usize / n, r);

        let giant_pow = r.pow(B).pow_table(B); // FIXME
        let baby_pow = r.pow_table(B); // FIXME

        Self { rcq, recip, log, giant_pow, baby_pow }
    }

    fn log_table(k: usize, r: M) -> Vec<M> {
        let ls = LinearSieve::new(k);
        let b = 200000; // FIXME
        let bsgs = Bsgs::<M>::new(b, r);
        let p_usize = M::MOD as usize;
        let log_neg1 = M::new((p_usize - 1) / 2);
        let mut res = vec![M::new(0); k + 1];
        for i in 2..=k {
            let lpf_i = ls.lpf(i).unwrap();
            res[i] = if lpf_i < i {
                res[lpf_i] + res[i / lpf_i]
            } else if i * i >= p_usize {
                let j = p_usize / i;
                let k = p_usize % i;
                log_neg1 + res[k] - res[j]
            } else {
                M::new(bsgs.log(M::new(i)))
            };
        }
        res
    }

    pub fn rat_convert(&self, x: M) -> ((bool, M), M) {
        self.rcq.rat_convert(x)
    }

    pub fn recip(&self, x: M) -> M {
        assert_ne!(x, M::new(0));

        let ((a_is_neg, a), b) = self.rat_convert(x);
        if a_is_neg {
            -self.recip[(-a).get() as usize] * b
        } else {
            self.recip[a.get() as usize] * b
        }
    }

    pub fn log(&self, x: M) -> M {
        let ((a_is_neg, a), b) = self.rat_convert(x);
        let log_neg1 = M::new((M::MOD as u32 - 1) / 2);
        let log_b = self.log[b.get() as usize];

        if a_is_neg {
            // log(-|a|/b) = log(-1) + log(|a|) - log(b)
            log_neg1 + self.log[(-a).get() as usize] - log_b
        } else {
            self.log[a.get() as usize] - log_b
        }
    }

    pub fn pow(&self, a: M, b: usize) -> M {
        if a.get() == 0 {
            return if b == 0 { M::new(1) } else { a };
        }

        let log_a = self.log(a);
        let e =
            ((b as u64 * log_a.get() as u64) % (M::MOD - 1) as u64) as usize;
        self.giant_pow[e / B] * self.baby_pow[e % B]
    }
}

#[test]
fn sanity_check() {
    const P: u32 = 1000003;
    type Mi = modint::StaticModInt<P>;

    let n = 101;
    let rcq = RatConvertQuery::<Mi>::new(n);

    for x_u32 in 0..P {
        let x = Mi::new(x_u32);
        let ((a_is_neg, a), b) = rcq.rat_convert(x);
        assert_eq!(a / b, x);

        let a_abs_u32 = if a_is_neg { P - a.get() } else { a.get() };
        let b_u32 = b.get();
        assert!(a_abs_u32 <= 2 * P / n as u32);
        assert!(1 <= b_u32 && b_u32 <= n as u32);
    }
}

#[test]
fn recip() {
    type Mi = modint::ModInt998244353;
    const P: u32 = Mi::MOD;

    let r = Mi::new(3);
    let rcq = RatConvertRecipLogPowQuery::<Mi>::new(r);

    for x_u32 in (1..P).step_by(20) {
        let x = Mi::new(x_u32);

        let x_recip = rcq.recip(x);
        assert_eq!(x * x_recip, Mi::new(1));
    }
}

#[test]
fn pow() {
    type Mi = modint::ModInt998244353;

    let r = Mi::new(3);
    let rcq = RatConvertRecipLogPowQuery::<Mi>::new(r);

    let x = Mi::new(123);
    let k = 200000;
    let x_pow = x.pow_table(k);
    for i in 0..=k {
        assert_eq!(rcq.pow(x, i), x_pow[i]);
    }
}
