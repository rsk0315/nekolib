use std::{
    cell::RefCell,
    fmt::{self, Debug},
    ops::{Deref, DerefMut, Range},
};

use monoid::{BinaryOp, Identity};
use monoid_action::MonoidAction;
use usize_bounds::UsizeBounds;

const WORD_SIZE: u32 = 0_usize.count_zeros();

fn lcp(i: usize, j: usize) -> usize {
    if i == 0 || j == 0 {
        return 0;
    }
    if i == j {
        return i;
    }
    let (i, j) = (i.min(j), i.max(j));
    let iz = i.leading_zeros();
    let i = i << iz;
    let j = j << j.leading_zeros();
    i >> iz.max(WORD_SIZE - (i ^ j).leading_zeros())
}

pub struct VecLazySegtree<A: MonoidAction> {
    tree: RefCell<Vec<<A::Operand as BinaryOp>::Set>>,
    susp: RefCell<Vec<<A::Operator as BinaryOp>::Set>>,
    len: usize,
    action: A,
}

impl<A: MonoidAction + Clone> Clone for VecLazySegtree<A>
where
    <A::Operand as BinaryOp>::Set: Clone,
    <A::Operator as BinaryOp>::Set: Clone,
{
    fn clone(&self) -> Self {
        Self {
            tree: RefCell::new(self.tree.borrow().to_vec()),
            susp: RefCell::new(self.susp.borrow().to_vec()),
            len: self.len,
            action: self.action.clone(),
        }
    }
}

impl<A: MonoidAction> VecLazySegtree<A> {
    #[must_use]
    pub fn new(len: usize) -> Self
    where
        A: Default,
    {
        let action = A::default();
        let tree: Vec<_> =
            (0..len + len).map(|_| action.operand().id()).collect();
        let susp: Vec<_> = (0..len).map(|_| action.operator().id()).collect();
        Self {
            len,
            tree: RefCell::new(tree),
            susp: RefCell::new(susp),
            action,
        }
    }

    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn len(&self) -> usize { self.len }

    fn arch_pair(&self, l: usize, r: usize) -> (Vec<usize>, Vec<usize>) {
        let mut l = self.len + l;
        let mut r = self.len + r;
        let mut vl = vec![];
        let mut vr = vec![];
        while l < r {
            if l & 1 == 1 {
                vl.push(l);
                l += 1;
            }
            if r & 1 == 1 {
                r -= 1;
                vr.push(r);
            }
            l >>= 1;
            r >>= 1;
        }
        (vl, vr)
    }
    fn arch(&self, l: usize, r: usize) -> Vec<usize> {
        let (mut vl, vr) = self.arch_pair(l, r);
        vl.extend(vr.into_iter().rev());
        vl
    }
    fn arch_rev(&self, l: usize, r: usize) -> Vec<usize> {
        let (vl, mut vr) = self.arch_pair(l, r);
        vr.extend(vl.into_iter().rev());
        vr
    }

    fn build_range(&mut self, start: usize, end: usize) {
        let mut tree = self.tree.borrow_mut();
        let susp = self.susp.borrow();
        let action = &self.action;
        let operand = action.operand();
        let id = action.operator().id();
        for i in self.ancestors_upward(start, end).filter(|&i| susp[i] == id) {
            tree[i] = operand.op(&tree[i << 1], &tree[i << 1 | 1]);
        }
    }

    fn act1(&self, i: usize, op: &<A::Operator as BinaryOp>::Set) {
        let mut tree = self.tree.borrow_mut();
        let mut susp = self.susp.borrow_mut();
        let operator = self.action.operator();
        tree[i] = self.action.act(&tree[i], op);
        if i < self.len {
            susp[i] = operator.op(&susp[i], op);
        }
    }

    fn force(&self, i: usize) {
        let id = || self.action.operator().id();
        let d = {
            let mut susp = self.susp.borrow_mut();
            std::mem::replace(&mut susp[i], id())
        };
        if d != id() {
            self.act1(i << 1, &d);
            self.act1(i << 1 | 1, &d);
        }
    }

    fn parent_root(&self, i: usize) -> usize {
        let n = self.len;
        if n.is_power_of_two() {
            return 0;
        }
        let n2 = 2 * n;
        let lsb = n2 & n2.wrapping_neg();
        lcp(i, if i < n2 ^ lsb { n2 ^ lsb } else { n2 })
    }

    fn ancestors_downward(
        &self,
        start: usize,
        end: usize,
    ) -> impl Iterator<Item = usize> + DoubleEndedIterator {
        self.ancestors_upward(start, end).rev()
    }

    fn ancestors_upward(
        &self,
        start: usize,
        end: usize,
    ) -> impl Iterator<Item = usize> + DoubleEndedIterator {
        let mut res = vec![];
        if start >= end {
            return res.into_iter();
        }

        let l = self.len + start;
        let r = self.len + end;
        let pl = self.parent_root(l);
        let pr = self.parent_root(r - 1);
        let (mut il, mut ir) = (1, 1);
        while l >> il != pl || (r - 1) >> ir != pr {
            if l >> il != pl {
                if l >> il << il != l {
                    res.push(l >> il);
                }
                il += 1;
            }
            if r >> ir != pr {
                if r >> ir << ir != r {
                    res.push((r - 1) >> ir);
                }
                ir += 1;
            }
        }
        res.dedup();
        res.into_iter()
    }

    fn force_range(&self, l: usize, r: usize) {
        for i in self.ancestors_downward(l, r) {
            self.force(i);
        }
    }

    fn force_all(&self) {
        let mut tree = self.tree.borrow_mut();
        let mut susp = self.susp.borrow_mut();
        let action = &self.action;
        let operator = action.operator();
        let id = || operator.id();
        for i in 1..self.len {
            let d = std::mem::replace(&mut susp[i], id());
            for &j in &[i << 1, i << 1 | 1] {
                if j < self.len {
                    susp[j] = operator.op(&susp[j], &d);
                }
                tree[j] = action.act(&tree[j], &d);
            }
        }
    }
}

impl<A: MonoidAction + Default> From<Vec<<A::Operand as BinaryOp>::Set>>
    for VecLazySegtree<A>
{
    fn from(value: Vec<<A::Operand as BinaryOp>::Set>) -> Self {
        Self::from((value, A::default()))
    }
}

impl<A: MonoidAction> From<(Vec<<A::Operand as BinaryOp>::Set>, A)>
    for VecLazySegtree<A>
{
    fn from(
        (mut value, action): (Vec<<A::Operand as BinaryOp>::Set>, A),
    ) -> Self {
        let len = value.len();
        let mut tree: Vec<_> =
            (0..len).map(|_| action.operand().id()).collect();
        tree.append(&mut value);
        for i in (0..len).rev() {
            tree[i] = action.operand().op(&tree[i << 1], &tree[i << 1 | 1]);
        }
        let tree = RefCell::new(tree);
        let susp: Vec<_> = (0..len).map(|_| action.operator().id()).collect();
        let susp = RefCell::new(susp);
        Self { tree, susp, len, action }
    }
}

impl<A: MonoidAction> Debug for VecLazySegtree<A>
where
    <A::Operand as BinaryOp>::Set: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.force_all(); // This may affect the content unintendedly.
        let tree = self.tree.borrow();
        f.debug_list().entries(&tree[self.len..]).finish()
    }
}

impl<A: MonoidAction> From<VecLazySegtree<A>>
    for Vec<<A::Operand as BinaryOp>::Set>
{
    fn from(value: VecLazySegtree<A>) -> Self {
        value.force_all();
        let mut res = value.tree.into_inner();
        res.drain(..value.len);
        res
    }
}

impl<A: MonoidAction> VecLazySegtree<A> {
    pub fn fold(
        &self,
        range: impl UsizeBounds,
    ) -> <A::Operand as BinaryOp>::Set {
        let Range { start, end } = range.to_range(self.len);
        let operand = self.action.operand();
        if start >= end {
            return operand.id();
        }
        self.force_range(start, end);
        let mut res = operand.id();
        let tree = self.tree.borrow();
        for v in self.arch(start, end) {
            res = operand.op(&res, &tree[v]);
        }
        res
    }

    pub fn act(
        &mut self,
        range: impl UsizeBounds,
        op: &<A::Operator as BinaryOp>::Set,
    ) {
        let Range { start, end } = range.to_range(self.len);
        if start >= end {
            return;
        }
        self.force_range(start, end);
        for v in self.arch(start, end) {
            self.act1(v, &op);
        }
        self.build_range(start, end);
    }

    pub fn fold_bisect_from<F>(
        &self,
        l: usize,
        pred: F,
    ) -> (usize, <A::Operand as BinaryOp>::Set)
    where
        F: Fn(&<A::Operand as BinaryOp>::Set) -> bool,
    {
        let len = self.len;
        assert!((0..=len).contains(&l));

        let operand = self.action.operand();
        let mut x = operand.id();
        assert!(pred(&x), "`pred(id) mus hold");
        match self.fold(l..) {
            x if pred(&x) => return (len, x),
            _ => (),
        }

        self.force_range(l, len);
        let tree = || self.tree.borrow();
        for v in self.arch(l, len) {
            let tmp = operand.op(&x, &tree()[v]);
            if pred(&tmp) {
                x = tmp;
                continue;
            }
            let mut v = v;
            while v < len {
                self.force(v);
                v <<= 1;
                let tmp = operand.op(&x, &tree()[v]);
                if pred(&tmp) {
                    x = tmp;
                    v += 1;
                }
            }
            return (v - len, x);
        }
        unreachable!();
    }

    pub fn fold_bisect_to<F>(
        &self,
        r: usize,
        pred: F,
    ) -> (usize, <A::Operand as BinaryOp>::Set)
    where
        F: Fn(&<A::Operand as BinaryOp>::Set) -> bool,
    {
        let len = self.len;
        assert!((0..=len).contains(&r));

        let operand = self.action.operand();
        let mut x = operand.id();
        assert!(pred(&x), "`pred(id) mus hold");
        match self.fold(..r) {
            x if pred(&x) => return (0, x),
            _ => (),
        }

        self.force_range(0, r);
        let tree = || self.tree.borrow();
        for v in self.arch_rev(0, r) {
            let tmp = operand.op(&tree()[v], &x);
            if pred(&tmp) {
                x = tmp;
                continue;
            }
            let mut v = v;
            while v < len {
                self.force(v);
                v = v << 1 | 1;
                let tmp = operand.op(&tree()[v], &x);
                if pred(&tmp) {
                    x = tmp;
                    v -= 1;
                }
            }
            return (v - len + 1, x);
        }
        unreachable!();
    }
}

#[doc(hidden)]
pub struct PeekMutTmp<'a, A: MonoidAction> {
    tree: &'a mut VecLazySegtree<A>,
    index: usize,
    elt: <A::Operand as BinaryOp>::Set,
}

impl<'a, A: MonoidAction + 'a> VecLazySegtree<A> {
    pub fn peek_mut(&'a mut self, index: usize) -> PeekMutTmp<'a, A> {
        self.force_range(index, index + 1);
        let i = self.len + index;
        let e = self.action.operand().id();
        let elt = std::mem::replace(&mut self.tree.borrow_mut()[i], e);
        PeekMutTmp { tree: self, index, elt }
    }
}

impl<A: MonoidAction> Drop for PeekMutTmp<'_, A> {
    fn drop(&mut self) {
        let Self { index, tree, elt } = self;
        let i = *index;
        let elt = std::mem::replace(elt, tree.action.operand().id());
        tree.tree.borrow_mut()[tree.len + i] = elt;
        tree.build_range(i, i + 1);
    }
}

impl<A: MonoidAction> Deref for PeekMutTmp<'_, A> {
    type Target = <A::Operand as BinaryOp>::Set;
    fn deref(&self) -> &Self::Target { &self.elt }
}

impl<A: MonoidAction> DerefMut for PeekMutTmp<'_, A> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.elt }
}
