//! `std` の B-tree の構成を理解することを目指す。
//!
//! `Allocator` など unstable な箇所に関しては、それを用いない形式に書き換えてある。
//! また、`Cursor`\[`Mut`\[`Key`\]\] などの unstable API についても対象外とする。

mod append;
mod borrow;
mod dedup_sorted_iter;
mod fix;
pub mod map;
mod mem;
mod merge_iter;
mod navigate;
mod node;
mod remove;
mod search;
pub mod set;
mod set_val;
mod split;

pub trait Recover<Q: ?Sized> {
    type Key;

    fn get(&self, key: &Q) -> Option<&Self::Key>;
    fn take(&mut self, key: &Q) -> Option<Self::Key>;
    fn replace(&mut self, key: Self::Key) -> Option<Self::Key>;
}
