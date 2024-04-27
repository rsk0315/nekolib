use std::{cell::RefCell, fmt};

#[derive(Clone)]
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
        let len = self.0.borrow().len();
        let mut ptn = vec![vec![]; len];
        for i in 0..len {
            ptn[self.repr(i)].push(i);
        }
        ptn
    }
    pub fn partition_len(&self) -> usize { self.1 }
}

struct AsSet<'a>(&'a Vec<usize>);
impl fmt::Debug for AsSet<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_set().entries(self.0.iter()).finish()
    }
}

impl fmt::Debug for UnionFind {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ptn = self.partition();
        let len = self.0.borrow().len();
        fmt.debug_map()
            .entries(
                (0..len)
                    .filter(|&i| !ptn[i].is_empty())
                    .map(|i| (i, AsSet(&ptn[i]))),
            )
            .finish()
    }
}

impl fmt::Display for UnionFind {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ptn = self.partition();
        fmt.debug_set()
            .entries(
                ptn.iter().filter(|set| !set.is_empty()).map(|set| AsSet(set)),
            )
            .finish()
    }
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

#[test]
fn debug_fmt() {
    let mut uf = UnionFind::new(8);
    uf.unite(1, 5);
    uf.unite(2, 4);
    uf.unite(0, 2);
    uf.unite(1, 6);
    uf.unite(6, 7);
    assert_eq!(format!("{uf}"), "{{0, 2, 4}, {3}, {1, 5, 6, 7}}");
    assert_eq!(format!("{uf:?}"), "{0: {0, 2, 4}, 3: {3}, 7: {1, 5, 6, 7}}");
}
