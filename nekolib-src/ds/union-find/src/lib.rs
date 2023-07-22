use std::cell::RefCell;

pub struct UnionFind(RefCell<Vec<usize>>, usize);

impl UnionFind {
    pub fn new(n: usize) -> Self {
        Self(RefCell::new(vec![1_usize.wrapping_neg(); n]), n)
    }
    pub fn unite(&mut self, u: usize, v: usize) -> bool {
        let u = self.repr(u);
        let v = self.repr(v);
        if u == v {
            return false;
        }

        let (par, child) =
            if self.count(u) < self.count(v) { (u, v) } else { (v, u) };

        let mut buf = self.0.borrow_mut();
        buf[par] = buf[par].wrapping_add(buf[child]);
        buf[child] = par;
        self.1 -= 1;
        true
    }
    pub fn equiv(&self, u: usize, v: usize) -> bool {
        self.repr(u) == self.repr(v)
    }
    pub fn repr(&self, u: usize) -> usize {
        let par = self.0.borrow()[u];
        if par >= self.0.borrow().len() {
            return u;
        }
        let repr = self.repr(par);
        self.0.borrow_mut()[u] = repr;
        repr
    }
    pub fn count(&self, u: usize) -> usize {
        let repr = self.repr(u);
        self.0.borrow()[repr].wrapping_neg()
    }
    pub fn partition(&self) -> Vec<Vec<usize>> {
        let buf = self.0.borrow();
        let mut ptn = vec![vec![]; buf.len()];
        for i in 0..buf.len() {
            ptn[self.repr(i)].push(i);
        }
        ptn
    }
    pub fn partition_len(&self) -> usize { self.1 }
}

#[test]
fn sanity_check() {
    let n = 10;
    let mut actual = UnionFind::new(n);
    let mut expected = naive::DisjointSet::new(n);

    let f = |(u, v)| 2_u128.pow(u as _) * 3_u128.pow(v as _) % 625;
    let query = {
        let mut query: Vec<_> =
            (0..n).flat_map(|u| (0..u).map(move |v| (u, v))).collect();
        query.sort_unstable_by_key(|&(u, v)| f((u, v)));
        query
    };

    for (u, v) in query {
        assert_eq!(actual.unite(u, v), expected.unite(u, v));
        for i in 0..n {
            for j in 0..n {
                assert_eq!(actual.equiv(i, j), expected.equiv(i, j));
            }
            assert_eq!(actual.count(i), expected.count(i));
        }
    }
}
