use std::ops::{Deref, DerefMut, Index, Range};

use monoid::Monoid;
use usize_bounds::UsizeBounds;

#[derive(Clone)]
pub struct VecSegtree<M: Monoid> {
    tree: Vec<M::Set>,
    monoid: M,
}

pub struct PeekMutTmp<'a, M: Monoid> {
    self_: &'a mut VecSegtree<M>,
    index: usize,
}

impl<M: Monoid> VecSegtree<M> {
    fn init(tree: &mut [M::Set], monoid: &M) {
        let n = tree.len() / 2;
        for i in (1..n).rev() {
            tree[i] = monoid.op(&tree[2 * i], &tree[2 * i + 1]);
        }
    }
    pub fn fold(&self, range: impl UsizeBounds) -> M::Set {
        let n = self.tree.len() / 2;
        let monoid = &self.monoid;
        let Range { start, end } = range.to_range(n);
        let (mut il, mut ir) = (n + start, n + end);
        let (mut resl, mut resr) = (monoid.id(), monoid.id());
        while il < ir {
            if il & 1 != 0 {
                resl = monoid.op(&resl, &self.tree[il]);
                il += 1;
            }
            if ir & 1 != 0 {
                ir -= 1;
                resr = monoid.op(&self.tree[ir], &resr);
            }
            il >>= 1;
            ir >>= 1;
        }
        monoid.op(&resl, &resr)
    }
    pub fn peek_mut(&mut self, i: usize) -> PeekMutTmp<'_, M> {
        let index = self.tree.len() / 2 + i;
        PeekMutTmp { self_: self, index }
    }

    fn roots(
        &self,
        Range { start, end }: Range<usize>,
    ) -> impl Iterator<Item = usize> + DoubleEndedIterator {
        let n = self.tree.len() / 2;
        let (mut il, mut ir) = (n + start, n + end);
        let (mut vl, mut vr) = (vec![], vec![]);
        while il < ir {
            if il & 1 != 0 {
                vl.push(il);
                il += 1;
            }
            if ir & 1 != 0 {
                ir -= 1;
                vr.push(ir);
            }
            il >>= 1;
            ir >>= 1;
        }
        vl.into_iter().chain(vr.into_iter().rev())
    }
    pub fn fold_bisect_from<F>(&self, l: usize, pred: F) -> (usize, M::Set)
    where
        F: Fn(&M::Set) -> bool,
    {
        let n = self.tree.len() / 2;
        assert!((0..=n).contains(&l));

        let monoid = &self.monoid;
        let mut x = monoid.id();
        assert!(pred(&x), "`pred(id)` must hold");
        match self.fold(l..) {
            x if pred(&x) => return (n, x),
            _ => {}
        }

        for v in self.roots(l..n) {
            let tmp = monoid.op(&x, &self.tree[v]);
            if pred(&tmp) {
                x = tmp;
                continue;
            }
            let mut v = v;
            while v < n {
                v *= 2;
                let tmp = monoid.op(&x, &self.tree[v]);
                if pred(&tmp) {
                    x = tmp;
                    v += 1;
                }
            }
            return (v - n, x);
        }
        unreachable!();
    }
    pub fn fold_bisect_to<F>(&self, r: usize, pred: F) -> (usize, M::Set)
    where
        F: Fn(&M::Set) -> bool,
    {
        let n = self.tree.len() / 2;
        assert!((0..=n).contains(&r));

        let monoid = &self.monoid;
        let mut x = monoid.id();
        assert!(pred(&x), "`pred(id)` must hold");
        match self.fold(..r) {
            x if pred(&x) => return (0, x),
            _ => {}
        }

        for v in self.roots(0..r).rev() {
            let tmp = monoid.op(&self.tree[v], &x);
            if pred(&tmp) {
                x = tmp;
                continue;
            }
            let mut v = v;
            while v < n {
                v = 2 * v + 1;
                let tmp = monoid.op(&self.tree[v], &x);
                if pred(&tmp) {
                    x = tmp;
                    v -= 1;
                }
            }
            return (v - n + 1, x);
        }
        unreachable!();
    }
}

impl<M: Monoid + Default> From<Vec<M::Set>> for VecSegtree<M> {
    fn from(a: Vec<M::Set>) -> Self {
        let n = a.len();
        let monoid = M::default();
        let mut tree: Vec<_> = (0..n).map(|_| monoid.id()).collect();
        tree.extend(a);
        Self::init(&mut tree, &monoid);
        Self { tree, monoid }
    }
}

impl<M: Monoid> From<(Vec<M::Set>, M)> for VecSegtree<M> {
    fn from((a, monoid): (Vec<M::Set>, M)) -> Self {
        let n = a.len();
        let mut tree: Vec<_> = (0..n).map(|_| monoid.id()).collect();
        tree.extend(a);
        Self::init(&mut tree, &monoid);
        Self { tree, monoid }
    }
}

impl<M: Monoid> Index<usize> for VecSegtree<M> {
    type Output = M::Set;
    fn index(&self, i: usize) -> &Self::Output {
        let i = self.tree.len() / 2 + i;
        self.tree.index(i)
    }
}

impl<M: Monoid> Deref for PeekMutTmp<'_, M> {
    type Target = M::Set;
    fn deref(&self) -> &Self::Target { &self.self_.tree[self.index] }
}

impl<M: Monoid> DerefMut for PeekMutTmp<'_, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.self_.tree[self.index]
    }
}

impl<M: Monoid> Drop for PeekMutTmp<'_, M> {
    fn drop(&mut self) {
        let Self {
            self_: VecSegtree { ref mut tree, ref monoid },
            index: mut i,
        } = self;
        while i > 1 {
            i >>= 1;
            tree[i] = monoid.op(&tree[2 * i], &tree[2 * i + 1]);
        }
    }
}

impl<M: Monoid> From<VecSegtree<M>> for Vec<M::Set> {
    fn from(mut self_: VecSegtree<M>) -> Vec<M::Set> {
        let n = self_.tree.len() / 2;
        self_.tree.split_off(n)
    }
}

impl<M: Monoid + Default> FromIterator<M::Set> for VecSegtree<M> {
    fn from_iter<I: IntoIterator<Item = M::Set>>(iter: I) -> Self {
        let buf: Vec<_> = iter.into_iter().collect();
        buf.into()
    }
}

#[test]
fn sanity_check() {
    use op_add::OpAdd;

    let mut tree: VecSegtree<OpAdd<i32>> = vec![1, 2, 3].into();
    assert_eq!(tree.fold(..), 6);
    *tree.peek_mut(1) -= 2;
    assert_eq!(tree.fold(..), 4);
}

#[test]
fn fold_bisect() {
    use op_add::OpAdd;

    let tree: VecSegtree<OpAdd<i32>> = vec![1, 2, 3, 4, 5].into();

    assert_eq!(tree.fold_bisect_from(0, |&x| x <= 0), (0, 0));
    assert_eq!(tree.fold_bisect_from(0, |&x| x <= 1), (1, 1));
    assert_eq!(tree.fold_bisect_from(0, |&x| x <= 2), (1, 1));
    assert_eq!(tree.fold_bisect_from(0, |&x| x <= 3), (2, 3));
    assert_eq!(tree.fold_bisect_from(0, |&x| x <= 5), (2, 3));
    assert_eq!(tree.fold_bisect_from(0, |&x| x <= 6), (3, 6));
    assert_eq!(tree.fold_bisect_from(0, |&x| x <= 9), (3, 6));
    assert_eq!(tree.fold_bisect_from(0, |&x| x <= 10), (4, 10));
    assert_eq!(tree.fold_bisect_from(0, |&x| x <= 14), (4, 10));
    assert_eq!(tree.fold_bisect_from(0, |&x| x <= 15), (5, 15));
    assert_eq!(tree.fold_bisect_from(0, |_| true), (5, 15));

    assert_eq!(tree.fold_bisect_from(1, |&x| x <= 1), (1, 0));
    assert_eq!(tree.fold_bisect_from(1, |&x| x <= 2), (2, 2));
    assert_eq!(tree.fold_bisect_from(1, |&x| x <= 4), (2, 2));
    assert_eq!(tree.fold_bisect_from(1, |&x| x <= 5), (3, 5));
    assert_eq!(tree.fold_bisect_from(1, |&x| x <= 8), (3, 5));
    assert_eq!(tree.fold_bisect_from(1, |&x| x <= 9), (4, 9));
    assert_eq!(tree.fold_bisect_from(1, |&x| x <= 13), (4, 9));
    assert_eq!(tree.fold_bisect_from(1, |&x| x <= 14), (5, 14));
    assert_eq!(tree.fold_bisect_from(1, |_| true), (5, 14));

    assert_eq!(tree.fold_bisect_from(2, |&x| x <= 2), (2, 0));
    assert_eq!(tree.fold_bisect_from(2, |&x| x <= 3), (3, 3));
    assert_eq!(tree.fold_bisect_from(2, |&x| x <= 6), (3, 3));
    assert_eq!(tree.fold_bisect_from(2, |&x| x <= 7), (4, 7));
    assert_eq!(tree.fold_bisect_from(2, |&x| x <= 11), (4, 7));
    assert_eq!(tree.fold_bisect_from(2, |&x| x <= 12), (5, 12));
    assert_eq!(tree.fold_bisect_from(2, |_| true), (5, 12));

    assert_eq!(tree.fold_bisect_from(3, |&x| x <= 3), (3, 0));
    assert_eq!(tree.fold_bisect_from(3, |&x| x <= 4), (4, 4));
    assert_eq!(tree.fold_bisect_from(3, |&x| x <= 8), (4, 4));
    assert_eq!(tree.fold_bisect_from(3, |&x| x <= 9), (5, 9));
    assert_eq!(tree.fold_bisect_from(3, |_| true), (5, 9));

    assert_eq!(tree.fold_bisect_from(4, |&x| x <= 4), (4, 0));
    assert_eq!(tree.fold_bisect_from(4, |&x| x <= 5), (5, 5));
    assert_eq!(tree.fold_bisect_from(4, |_| true), (5, 5));

    assert_eq!(tree.fold_bisect_from(5, |_| true), (5, 0));

    assert_eq!(tree.fold_bisect_to(0, |_| true), (0, 0));

    assert_eq!(tree.fold_bisect_to(1, |&x| x <= 0), (1, 0));
    assert_eq!(tree.fold_bisect_to(1, |&x| x <= 1), (0, 1));
    assert_eq!(tree.fold_bisect_to(1, |_| true), (0, 1));

    assert_eq!(tree.fold_bisect_to(2, |&x| x <= 0), (2, 0));
    assert_eq!(tree.fold_bisect_to(2, |&x| x <= 1), (2, 0));
    assert_eq!(tree.fold_bisect_to(2, |&x| x <= 2), (1, 2));
    assert_eq!(tree.fold_bisect_to(2, |&x| x <= 3), (0, 3));
    assert_eq!(tree.fold_bisect_to(2, |_| true), (0, 3));

    assert_eq!(tree.fold_bisect_to(3, |&x| x <= 2), (3, 0));
    assert_eq!(tree.fold_bisect_to(3, |&x| x <= 3), (2, 3));
    assert_eq!(tree.fold_bisect_to(3, |&x| x <= 4), (2, 3));
    assert_eq!(tree.fold_bisect_to(3, |&x| x <= 5), (1, 5));
    assert_eq!(tree.fold_bisect_to(3, |&x| x <= 6), (0, 6));
    assert_eq!(tree.fold_bisect_to(3, |_| true), (0, 6));

    assert_eq!(tree.fold_bisect_to(4, |&x| x <= 0), (4, 0));
    assert_eq!(tree.fold_bisect_to(4, |&x| x <= 3), (4, 0));
    assert_eq!(tree.fold_bisect_to(4, |&x| x <= 4), (3, 4));
    assert_eq!(tree.fold_bisect_to(4, |&x| x <= 6), (3, 4));
    assert_eq!(tree.fold_bisect_to(4, |&x| x <= 7), (2, 7));
    assert_eq!(tree.fold_bisect_to(4, |&x| x <= 8), (2, 7));
    assert_eq!(tree.fold_bisect_to(4, |&x| x <= 9), (1, 9));
    assert_eq!(tree.fold_bisect_to(4, |&x| x <= 10), (0, 10));
    assert_eq!(tree.fold_bisect_to(4, |_| true), (0, 10));

    assert_eq!(tree.fold_bisect_to(5, |&x| x <= 0), (5, 0));
    assert_eq!(tree.fold_bisect_to(5, |&x| x <= 4), (5, 0));
    assert_eq!(tree.fold_bisect_to(5, |&x| x <= 5), (4, 5));
    assert_eq!(tree.fold_bisect_to(5, |&x| x <= 8), (4, 5));
    assert_eq!(tree.fold_bisect_to(5, |&x| x <= 9), (3, 9));
    assert_eq!(tree.fold_bisect_to(5, |&x| x <= 10), (3, 9));
    assert_eq!(tree.fold_bisect_to(5, |&x| x <= 12), (2, 12));
    assert_eq!(tree.fold_bisect_to(5, |&x| x <= 13), (2, 12));
    assert_eq!(tree.fold_bisect_to(5, |&x| x <= 14), (1, 14));
    assert_eq!(tree.fold_bisect_to(5, |_| true), (0, 15));
}
