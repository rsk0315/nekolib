use modint::ModInt;

#[derive(Clone, Debug)]
pub struct FactorialTable<M> {
    fact: Vec<M>,
    fact_recip: Vec<M>,
}

// where M: PrimeMod? n <= M?
impl<M: ModInt> FactorialTable<M> {
    pub fn new(n: usize) -> Self {
        let mut fact = vec![M::new(1); n + 1];
        for i in 0..n {
            fact[i + 1] = fact[i] * M::new(i + 1);
        }
        let mut fact_recip = vec![M::new(1); n + 1];
        fact_recip[n] = fact[n].recip();
        for i in (0..n).rev() {
            fact_recip[i] = fact_recip[i + 1] * M::new(i + 1);
        }
        Self { fact, fact_recip }
    }

    pub fn factorial(&self, i: usize) -> M { self.fact[i] }
    pub fn factorial_recip(&self, i: usize) -> M { self.fact_recip[i] }

    pub fn perm(&self, i: usize, j: usize) -> M {
        if j <= i { self.fact[i] * self.fact_recip[i - j] } else { M::new(0) }
    }
    pub fn perm_recip(&self, i: usize, j: usize) -> M {
        self.fact_recip[i] * self.fact[i - j]
    }

    pub fn binom(&self, i: usize, j: usize) -> M {
        self.perm(i, j) * self.fact_recip[j]
    }
    pub fn binom_recip(&self, i: usize, j: usize) -> M {
        self.perm_recip(i, j) * self.fact[j]
    }

    pub fn recip(&self, i: usize) -> M { self.fact_recip[i] * self.fact[i - 1] }
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
