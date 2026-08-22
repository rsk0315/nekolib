use std::ops::{Add, Neg};

use has_zero::HasZero;
use monoid::{Associative, BinaryOp, Commutative, Identity, Recip};

#[derive(Clone, Debug)]
pub struct OpAddDeg1<T>(std::marker::PhantomData<fn(&T) -> T>);

impl<T> Default for OpAddDeg1<T> {
    fn default() -> Self { Self(std::marker::PhantomData) }
}

impl<T: Eq> BinaryOp for OpAddDeg1<T>
where
    for<'a> &'a T: Add<&'a T, Output = T>,
{
    type Set = (T, T);
    fn op(&self, (a0, a1): &(T, T), (b0, b1): &(T, T)) -> (T, T) {
        (a0 + b0, a1 + b1)
    }
}

impl<T: Eq + HasZero> Identity for OpAddDeg1<T>
where
    for<'a> &'a T: Add<&'a T, Output = T>,
{
    fn id(&self) -> (T, T) { (T::zero(), T::zero()) }
}

impl<T: Eq> Recip for OpAddDeg1<T>
where
    for<'a> &'a T: Add<&'a T, Output = T> + Neg<Output = T>,
{
    fn recip(&self, (a0, a1): &(T, T)) -> (T, T) { (-a0, -a1) }
}

impl<T: Eq> Associative for OpAddDeg1<T> where
    for<'a> &'a T: Add<&'a T, Output = T>
{
}
impl<T: Eq> Commutative for OpAddDeg1<T> where
    for<'a> &'a T: Add<&'a T, Output = T>
{
}

#[test]
fn sanity_check() {
    let op_add_deg1: OpAddDeg1<i32> = Default::default();
    assert_eq!(op_add_deg1.op(&(2, 3), &(5, 7)), (7, 10));
    assert_eq!(op_add_deg1.id(), (0, 0));
    assert_eq!(op_add_deg1.recip(&(1, 2)), (-1, -2));
}
