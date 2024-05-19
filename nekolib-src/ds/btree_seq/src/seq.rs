use std::marker::PhantomData;

use super::node::Root;

pub struct BTreeSeq<T, R, Fi, Fr> {
    root: Option<Root<T, R>>,
    fn_init: Fi,   // impl Fn() -> R
    fn_reduce: Fr, // impl Fn(R, T) -> R
    // For dropck; the `Box` acoid making the `Unpin` impl more strict
    // than before.
    _marker: PhantomData<Box<(T, R)>>,
}
