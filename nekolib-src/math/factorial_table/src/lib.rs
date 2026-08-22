use modint::ModInt;

#[derive(Clone, Debug)]
pub struct FactorialTable<M> {
    factorial: Vec<M>,
    factorial_recip: Vec<M>,
}

// where M: PrimeMod? n <= M?
impl<M: ModInt> FactorialTable<M> {
    pub fn new(n: usize) -> Self {
        let mut factorial = vec![M::new(1); n + 1];
        for i in 0..n {
            factorial[i + 1] = factorial[i] * M::new(i + 1);
        }
        let mut factorial_recip = vec![M::new(1); n + 1];
        factorial_recip[n] = factorial[n].recip();
        for i in (0..n).rev() {
            factorial_recip[i] = factorial_recip[i + 1] * M::new(i + 1);
        }
        Self { factorial, factorial_recip }
    }

    pub fn factorial(&self, i: usize) -> M { self.factorial[i] }
    pub fn factorial_recip(&self, i: usize) -> M { self.factorial_recip[i] }

    pub fn binom(&self, i: usize, j: usize) -> M {
        if j <= i {
            self.factorial[i]
                * self.factorial_recip[j]
                * self.factorial_recip[i - j]
        } else {
            M::new(0)
        }
    }
    pub fn binom_recip(&self, i: usize, j: usize) -> M {
        self.factorial_recip[i] * self.factorial[j] * self.factorial[i - j]
    }

    pub fn recip(&self, i: usize) -> M {
        self.factorial_recip[i] * self.factorial[i - 1]
    }
}

#[test]
fn sanity_check() {
    type Mi = modint::ModInt998244353;

    let ft = FactorialTable::<Mi>::new(10);
    assert_eq!(ft.factorial(7), Mi::new(5040));
    assert_eq!(ft.factorial_recip(7), Mi::new(5040).recip());
    assert_eq!(ft.binom(5, 2), Mi::new(10));
    assert_eq!(ft.binom_recip(5, 2), Mi::new(10).recip());
    assert_eq!(ft.recip(10), Mi::new(10).recip());
}
