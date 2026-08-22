use std::ops::{Add, Neg};

use has_zero::HasZero;
use monoid::{Associative, BinaryOp, Commutative, Identity, Recip};

#[derive(Clone, Debug)]
pub struct OpAdd<T>(std::marker::PhantomData<fn(&T) -> T>);

impl<T> Default for OpAdd<T> {
    fn default() -> Self { Self(std::marker::PhantomData) }
}

impl<T: Eq> BinaryOp for OpAdd<T>
where
    for<'a> &'a T: Add<&'a T, Output = T>,
{
    type Set = T;
    fn op(&self, lhs: &T, rhs: &T) -> T { lhs + rhs }
}

impl<T: Eq + HasZero> Identity for OpAdd<T>
where
    for<'a> &'a T: Add<&'a T, Output = T>,
{
    fn id(&self) -> T { T::zero() }
}

impl<T: Eq> Recip for OpAdd<T>
where
    for<'a> &'a T: Add<&'a T, Output = T> + Neg<Output = T>,
{
    fn recip(&self, elt: &T) -> T { -elt }
}

impl<T: Eq> Associative for OpAdd<T> where for<'a> &'a T: Add<&'a T, Output = T> {}
impl<T: Eq> Commutative for OpAdd<T> where for<'a> &'a T: Add<&'a T, Output = T> {}

#[test]
fn sanity_check() {
    let op_add: OpAdd<i32> = Default::default();
    assert_eq!(op_add.op(&1, &2), 3);
    assert_eq!(op_add.id(), 0);
    assert_eq!(op_add.recip(&1), -1);
}
