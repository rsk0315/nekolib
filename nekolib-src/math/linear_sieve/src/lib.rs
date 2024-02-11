pub struct LinearSieve {
    lpf: Vec<usize>,
    lpf_e: Vec<(usize, u32)>,
    pr: Vec<usize>,
}

impl LinearSieve {
    pub fn new(n: usize) -> Self {
        let mut lpf = vec![1; n + 1];
        let mut pr = vec![];
        for i in 2..=n {
            if lpf[i] == 1 {
                lpf[i] = i;
                pr.push(i);
            }
            let lpf_i = lpf[i];
            for &j in pr.iter().take_while(|&&j| j <= lpf_i.min(n / i)) {
                lpf[i * j] = j;
            }
        }
        let mut lpf_e = vec![(1, 0); n + 1];
        for i in 2..=n {
            let p = lpf[i];
            let j = i / p;
            lpf_e[i] = if lpf[j] == p {
                (lpf_e[j].0 * p, lpf_e[j].1 + 1)
            } else {
                (lpf[i], 1)
            };
        }
        Self { lpf, lpf_e, pr }
    }

    pub fn is_prime(&self, i: usize) -> bool { i >= 2 && self.lpf[i] == i }
    pub fn lpf(&self, i: usize) -> Option<usize> {
        (i >= 2).then(|| self.lpf[i])
    }

    pub fn factors_dup(&self, i: usize) -> impl Iterator<Item = usize> + '_ {
        std::iter::successors(Some(i), move |&i| Some(i / self.lpf[i]))
            .take_while(|&i| i > 1)
            .map(move |i| self.lpf[i])
    }

    pub fn factors(&self, i: usize) -> impl Iterator<Item = (usize, u32)> + '_ {
        std::iter::successors(Some(i), move |&i| Some(i / self.lpf_e[i].0))
            .take_while(|&i| i > 1)
            .map(move |i| (self.lpf[i], self.lpf_e[i].1))
    }

    pub fn euler_phi(&self, i: usize) -> usize {
        std::iter::successors(Some(i), move |&i| Some(i / self.lpf_e[i].0))
            .take_while(|&i| i > 1)
            .map(|i| self.lpf_e[i].0 / self.lpf[i] * (self.lpf[i] - 1))
            .product()
    }

    pub fn euler_phi_star(&self, i: usize) -> usize {
        match i {
            0..=2 => i / 2,
            _ => 1 + self.euler_phi_star(self.euler_phi(i)),
        }
    }

    pub fn divisors(
        &self,
        i: usize,
    ) -> impl Iterator<Item = usize> + DoubleEndedIterator {
        let mut res = vec![1];
        for (p, e) in self.factors(i) {
            let mut tmp = vec![];
            let mut pp = 1;
            for _ in 1..=e {
                pp *= p;
                tmp.extend(res.iter().map(|&x| x * pp));
            }
            res.extend(tmp);
        }
        res.sort_unstable();
        res.into_iter()
    }

    pub fn divisors_count(&self, i: usize) -> usize {
        self.factors(i).map(|(_, e)| e as usize + 1).product()
    }

    pub fn divisors_sum(&self, i: usize) -> usize {
        std::iter::successors(Some(i), move |&i| Some(i / self.lpf_e[i].0))
            .take_while(|&i| i > 1)
            .map(|i| (self.lpf_e[i].0 * self.lpf[i] - 1) / (self.lpf[i] - 1))
            .product()
    }

    pub fn primes(
        &self,
    ) -> impl Iterator<Item = usize> + DoubleEndedIterator + '_ {
        self.pr.iter().copied()
    }

    pub fn dp<T>(
        &self,
        zero: T,
        one: T,
        eq: impl Fn(&T, usize) -> T,
        gt: impl Fn(&T, usize) -> T,
    ) -> Vec<T> {
        let n = self.lpf.len() - 1;

        let mut res = vec![zero, one];
        if n <= 1 {
            res.truncate(n + 1);
            return res;
        }

        res.reserve(n + 1);
        for i in 2..=n {
            let lpf = self.lpf[i];
            let j = i / lpf;
            let prev = &res[j];
            let cur =
                if lpf == self.lpf[j] { eq(prev, lpf) } else { gt(prev, lpf) };
            res.push(cur);
        }
        res
    }
}

#[test]
fn sanity_check() {
    let ls = LinearSieve::new(60);

    let primes =
        vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59];

    assert!(ls.primes().eq(primes.iter().copied()));
    assert!((1..=60).all(|i| primes.contains(&i) == ls.is_prime(i)));

    assert_eq!(ls.lpf(1), None);
    assert_eq!(ls.lpf(3), Some(3));
    assert_eq!(ls.lpf(24), Some(2));

    assert!(ls.factors_dup(1).eq(None));
    assert!(ls.factors_dup(60).eq([2, 2, 3, 5]));

    assert!(ls.factors(1).eq(None));
    assert!(ls.factors(60).eq([(2, 2), (3, 1), (5, 1)]));

    assert_eq!(ls.euler_phi(1), 1);
    assert_eq!(ls.euler_phi(35), 24);
    assert_eq!(ls.euler_phi(60), 16);

    assert_eq!(ls.euler_phi_star(1), 0);
    assert_eq!(ls.euler_phi_star(35), 5);
    assert_eq!(ls.euler_phi_star(60), 5);

    assert!(ls.divisors(1).eq([1]));
    assert!(ls.divisors(60).eq([1, 2, 3, 4, 5, 6, 10, 12, 15, 20, 30, 60]));

    assert_eq!(ls.divisors_count(1), 1);
    assert_eq!(ls.divisors_count(60), 12);

    assert_eq!(ls.divisors_sum(1), 1);
    assert_eq!(ls.divisors_sum(60), 168);
}

#[test]
fn dp() {
    let ls = LinearSieve::new(10);

    // Moebius mu
    let mu = ls.dp(0, 1, |_, _| 0, |&x, _| -x);
    assert_eq!(mu, [0, 1, -1, -1, 0, -1, 1, -1, 0, 0, 1]);

    // Euler phi
    let phi = ls.dp(0, 1, |&x, p| x * p, |&x, p| x * (p - 1));
    assert_eq!(phi, [0, 1, 1, 2, 2, 4, 2, 6, 4, 6, 4]);

    // # of distinct prime factors
    let omega = ls.dp(0, 0, |&x, _| x, |&x, _| x + 1);
    assert_eq!(omega, [0, 0, 1, 1, 1, 1, 2, 1, 1, 1, 2]);

    // # of prime factors
    let cap_omega = ls.dp(0, 0, |&x, _| x + 1, |&x, _| x + 1);
    assert_eq!(cap_omega, [0, 0, 1, 1, 2, 1, 2, 1, 3, 2, 2]);

    // sum of divisors
    let eq = |&(prod, sum, pow): &_, p| (prod, sum + pow * p, pow * p);
    let gt = |&(prod, sum, _): &_, p| (prod * sum, 1 + p, p);
    let sigma: [_; 11] =
        ls.dp((0, 0, 0), (1, 1, 1), eq, gt).try_into().unwrap();
    let sigma = sigma.map(|(prod, sum, _)| prod * sum);
    assert_eq!(sigma, [0, 1, 3, 4, 7, 6, 12, 8, 15, 13, 18]);
}
