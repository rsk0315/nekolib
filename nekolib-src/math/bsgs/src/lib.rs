use std::collections::HashMap;

use modint::ModInt;

pub struct Bsgs<M: ModInt> {
    b: usize,
    rb_recip: M,
    giant: HashMap<u32, usize>,
}

impl<M: ModInt> Bsgs<M> {
    pub fn new(b: usize, r: M) -> Self {
        const { assert!(M::IS_PRIME) }
        // r must be a primitive root of M

        // r^n = r^(ib+j) = (r^b)^i * r^j
        let rb_recip = r.pow(b).recip();
        let r_pow = r.pow_table(b - 1);
        let mut giant = HashMap::new();
        for i in 0..b {
            giant.insert(r_pow[i].get(), i as usize);
        }
        Self { b, rb_recip, giant }
    }

    pub fn log(&self, mut x: M) -> usize {
        assert_ne!(x.get(), 0);

        let rb_recip = self.rb_recip;
        for ib in (0..M::MOD as usize).step_by(self.b) {
            if let Some(&j) = self.giant.get(&x.get()) {
                return ib + j;
            }
            x *= rb_recip;
        }
        unreachable!()
    }
}

#[test]
fn sanity_check() {
    type Mi = modint::ModInt998244353;

    let b = 200000;
    let r = Mi::new(3);
    let n = 10000;
    let s = Mi::new(5);
    let s_pow = s.pow_table(n);

    let bsgs = Bsgs::new(b, r);
    for i in 0..=n {
        let log = bsgs.log(s_pow[i]);
        assert_eq!(r.pow(log), s_pow[i]);
    }
}
