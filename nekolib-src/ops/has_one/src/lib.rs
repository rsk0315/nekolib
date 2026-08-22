use std::iter::Product;

pub trait HasOne {
    fn one() -> Self;
}

impl<T: Product> HasOne for T {
    fn one() -> Self { None.into_iter().product() }
}

#[test]
fn sanity_check() {
    assert_eq!(i32::one(), 1);
    assert_eq!(f64::one(), 1.0);
    assert_eq!(usize::one(), 1);
}
