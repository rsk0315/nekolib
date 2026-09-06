use std::ops::{Add, Div, Mul, Neg};

use has_one::HasOne;
use has_zero::HasZero;
use monoid::{Associative, BinaryOp, Identity, Recip};

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

impl<T: Eq + HasZero + HasOne> Recip for OpAffine<T>
where
    for<'a> &'a T: Add<&'a T, Output = T>
        + Neg<Output = T>
        + Mul<&'a T, Output = T>
        + Div<&'a T, Output = T>,
{
    fn recip(&self, (a0, a1): &Self::Set) -> Self::Set {
        // (ad+c) + (bd)x = 0+1x
        let b1 = &T::one() / &a1;
        let b0 = -(&(a0 * &b1));
        (b0, b1)
    }
}

#[test]
fn sanity_check() {
    let op_affine: OpAffine<i32> = Default::default();
    assert_eq!(op_affine.op(&(2, 3), &(5, 7)), (19, 21));
    assert_eq!(op_affine.id(), (0, 1));
}

#[test]
fn recip() {
    type Mi = modint::ModInt998244353;

    let op_affine: OpAffine<Mi> = Default::default();
    let f = (Mi::new(3), Mi::new(5));
    let f_recip = op_affine.recip(&f);

    assert_eq!(op_affine.op(&f, &f_recip), op_affine.id());
    assert_eq!(op_affine.op(&f_recip, &f), op_affine.id());
}
