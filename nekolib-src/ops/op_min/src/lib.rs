use has_maximum::HasMaximum;
use monoid::{Associative, BinaryOp, Commutative, Identity};

#[derive(Clone, Debug)]
pub struct OpMin<T>(std::marker::PhantomData<fn(&T) -> T>);

impl<T> Default for OpMin<T> {
    fn default() -> Self { Self(std::marker::PhantomData) }
}

impl<T> BinaryOp for OpMin<T>
where
    T: Ord + Eq + Clone,
{
    type Set = T;
    fn op(&self, lhs: &T, rhs: &T) -> T { lhs.min(rhs).clone() }
}

impl<T: Ord + Eq + Clone + HasMaximum> Identity for OpMin<T> {
    fn id(&self) -> Self::Set { <T as HasMaximum>::maximum() }
}

impl<T: Ord> Associative for OpMin<T> {}
impl<T: Ord> Commutative for OpMin<T> {}

#[test]
fn sanity_check() {
    let op_min: OpMin<i32> = Default::default();
    assert_eq!(op_min.op(&1, &2), 1);
    assert_eq!(op_min.id(), i32::MAX);
}
