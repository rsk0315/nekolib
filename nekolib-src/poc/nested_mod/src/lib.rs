mod priv_inner;
pub mod pub_inner;

pub mod nested1;

pub use nested1::nested2::nested3::foo as nested_foo;
pub use priv_inner::foo;

#[test]
fn test_foo() {
    assert_eq!(priv_inner::foo(), 0);
    assert_eq!(pub_inner::foo(), 0);
    assert_eq!(nested_foo(), 0);
}
