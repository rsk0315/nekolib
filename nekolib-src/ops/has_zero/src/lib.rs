use std::iter::Sum;

pub trait HasZero {
    fn zero() -> Self;
}

impl<T: Sum> HasZero for T {
    fn zero() -> Self { None.into_iter().sum() }
}

#[test]
fn sanity_check() {
    assert_eq!(i32::zero(), 0);
    assert_eq!(f64::zero(), 0.0);
    assert_eq!(usize::zero(), 0);
}
