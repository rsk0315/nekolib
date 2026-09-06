use std::ops::Range;

use monoid::{Group, Monoid};
use usize_bounds::{PrefixRange, UsizeBounds};

pub struct Accum<M: Monoid> {
    acc: Vec<M::Set>,
    monoid: M,
}

impl<M: Monoid + Default> From<Vec<M::Set>> for Accum<M> {
    fn from(mut value: Vec<M::Set>) -> Self {
        let n = value.len();
        let monoid = M::default();
        value.insert(0, monoid.id());
        for i in 0..n {
            value[i + 1] = monoid.op(&value[i], &value[i + 1]);
        }
        Self { acc: value, monoid }
    }
}

impl<M: Monoid> From<(Vec<M::Set>, M)> for Accum<M> {
    fn from((mut value, monoid): (Vec<M::Set>, M)) -> Self {
        let n = value.len();
        value.insert(0, monoid.id());
        for i in 0..n {
            value[i + 1] = monoid.op(&value[i], &value[i + 1]);
        }
        Self { acc: value, monoid }
    }
}

impl<'a, M: Monoid + Default> From<&'a [M::Set]> for Accum<M> {
    fn from(value: &'a [M::Set]) -> Self {
        let monoid = M::default();
        let mut acc = vec![monoid.id()];
        for x in value {
            acc.push(monoid.op(acc.last().unwrap(), x));
        }
        Self { acc, monoid }
    }
}

impl<'a, M: Monoid> From<(&'a [M::Set], M)> for Accum<M> {
    fn from((value, monoid): (&'a [M::Set], M)) -> Self {
        let mut acc = vec![monoid.id()];
        for x in value {
            acc.push(monoid.op(acc.last().unwrap(), x));
        }
        Self { acc, monoid }
    }
}

impl<M: Monoid + Default, const N: usize> From<[M::Set; N]> for Accum<M> {
    fn from(value: [M::Set; N]) -> Self { Self::from(&value[..]) }
}

impl<M: Monoid, const N: usize> From<([M::Set; N], M)> for Accum<M> {
    fn from((value, monoid): ([M::Set; N], M)) -> Self {
        Self::from((&value[..], monoid))
    }
}

impl<M: Monoid> Accum<M> {
    pub fn prefix_fold(
        &self,
        range: impl UsizeBounds + PrefixRange,
    ) -> &M::Set {
        let n = self.acc.len() - 1;
        let Range { start: _, end } = range.to_range(n);
        &self.acc[end]
    }

    pub fn fold(&self, range: impl UsizeBounds) -> M::Set
    where
        M: Group,
    {
        let n = self.acc.len() - 1;
        let Range { start, end } = range.to_range(n);
        self.monoid.op(
            &self.monoid.recip(self.prefix_fold(..start)),
            &self.prefix_fold(..end),
        )
    }
}

#[test]
fn sanity_check() {
    use op_add::OpAdd;

    let a = vec![2, 3, 4, 5];
    let acc: Accum<OpAdd<i32>> = a.into();
    assert_eq!(acc.prefix_fold(..0), &0);
    assert_eq!(acc.prefix_fold(..1), &2);
    assert_eq!(acc.prefix_fold(..2), &5);
    assert_eq!(acc.prefix_fold(..3), &9);
    assert_eq!(acc.prefix_fold(..4), &14);
    assert_eq!(acc.prefix_fold(..), &14);

    assert_eq!(acc.fold(1..3), 7);

    let _acc: Accum<OpAdd<i32>> = [2, 3, 4, 5].into();
    let _acc: Accum<OpAdd<i32>> = [2, 3, 4, 5][..].into();
}

#[test]
fn fold() {
    type OpAffine<T> = op_affine::OpAffine<T>;
    type Mi = modint::ModInt998244353;

    let a = [(31, 41), (59, 26), (53, 58), (97, 93), (23, 84)]
        .map(|(a0, a1)| (Mi::new(a0), Mi::new(a1)));
    let acc_0: Accum<OpAffine<Mi>> = a.into();

    let n = a.len();
    for il in 0..n {
        let acc_il: Accum<OpAffine<Mi>> = a[il..].into();
        for ir in il..=n {
            assert_eq!(&acc_0.fold(il..ir), acc_il.prefix_fold(..ir - il));
        }
    }
}
