use std::ops::{Deref, DerefMut, Index, Range};

use monoid::Monoid;
use usize_bounds::UsizeBounds;

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

#[test]
fn sanity_check() {
    use op_add::OpAdd;

    let mut tree: VecSegtree<OpAdd<i32>> = vec![1, 2, 3].into();
    assert_eq!(tree.fold(..), 6);
    *tree.peek_mut(1) -= 2;
    assert_eq!(tree.fold(..), 4);
}
