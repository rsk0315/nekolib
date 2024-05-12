//! [`SetValZST`] の定義。

pub struct SetValZST;

pub trait IsSetVal {
    fn is_set_val() -> bool { false }
}

impl IsSetVal for SetValZST {
    fn is_set_val() -> bool { true }
}
