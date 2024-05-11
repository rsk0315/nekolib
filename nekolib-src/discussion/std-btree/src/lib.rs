//! `std` の B-tree。
//!
//! `Allocator` など unstable な箇所に関しては、それを用いない形式に書き換えてある。

mod mem;
pub mod node;

pub trait Recover<Q: ?Sized> {
    type Key;

    fn get(&self, key: &Q) -> Option<&Self::Key>;
    fn take(&mut self, key: &Q) -> Option<Self::Key>;
    fn replace(&mut self, key: Self::Key) -> Option<Self::Key>;
}
