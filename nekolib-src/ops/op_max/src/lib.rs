use has_minimum::HasMinimum;
use monoid::{Associative, BinaryOp, Commutative, Identity};

#[derive(Clone, Debug)]
pub struct OpMax<T>(std::marker::PhantomData<fn(&T) -> T>);

impl<T> Default for OpMax<T> {
    fn default() -> Self { Self(std::marker::PhantomData) }
}

impl<T> BinaryOp for OpMax<T>
where
    T: Ord + Eq + Clone,
{
    type Set = T;
    fn op(&self, lhs: &T, rhs: &T) -> T { lhs.max(rhs).clone() }
}

impl<T: Ord + Eq + Clone + HasMinimum> Identity for OpMax<T> {
    fn id(&self) -> Self::Set { <T as HasMinimum>::minimum() }
}

impl<T: Ord> Associative for OpMax<T> {}
impl<T: Ord> Commutative for OpMax<T> {}

#[test]
fn sanity_check() {
    let op_max: OpMax<i32> = Default::default();
    assert_eq!(op_max.op(&1, &2), 2);
    assert_eq!(op_max.id(), i32::MIN);
}
