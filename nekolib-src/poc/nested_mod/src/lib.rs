mod priv_inner;
pub mod pub_inner;

pub use priv_inner::foo;

#[test]
fn test_foo() {
    assert_eq!(priv_inner::foo(), 0);
    assert_eq!(pub_inner::foo(), 0);
}
