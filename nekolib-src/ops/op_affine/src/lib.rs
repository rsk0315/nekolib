use std::ops::{Add, Mul};

use has_one::HasOne;
use has_zero::HasZero;
use monoid::{Associative, BinaryOp, Identity};

#[derive(Clone, Debug)]
pub struct OpAffine<T>(std::marker::PhantomData<fn(&T) -> T>);

impl<T> Default for OpAffine<T> {
    fn default() -> Self { Self(std::marker::PhantomData) }
}

impl<T: Eq> BinaryOp for OpAffine<T>
where
    for<'a> &'a T: Add<&'a T, Output = T> + Mul<&'a T, Output = T>,
{
    type Set = (T, T);
    fn op(&self, (a0, a1): &(T, T), (b0, b1): &(T, T)) -> (T, T) {
        // c + d(a+bx) = (ad+c) + (bd)x
        let z0 = &(a0 * b1) + b0;
        let z1 = a1 * b1;
        (z0, z1)
    }
}

impl<T: Eq + HasZero + HasOne> Identity for OpAffine<T>
where
    for<'a> &'a T: Add<&'a T, Output = T> + Mul<&'a T, Output = T>,
{
    // x = 0+1x
    fn id(&self) -> (T, T) { (T::zero(), T::one()) }
}

impl<T> Associative for OpAffine<T> {}

#[test]
fn sanity_check() {
    let op_affine: OpAffine<i32> = Default::default();
    assert_eq!(op_affine.op(&(2, 3), &(5, 7)), (19, 21));
    assert_eq!(op_affine.id(), (0, 1));
}
