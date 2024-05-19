//! `std` の B-tree の構成を理解することを目指す。
//!
//! `Allocator` など unstable な箇所に関しては、それを用いない形式に書き換えてある。
//! また、`Cursor`\[`Mut`\[`Key`\]\] などの unstable API についても対象外とする。
//!
//! `unsafe` の概念自体や variance、Stacked/Tree Borrows
//! に関してはすでに習得済みの前提とする。
//!
//! # Outline
//!
//! まず、[`node`] を見てノードの構造を把握するのがよい。木の作成、ノードの
//! allocate、辺を辿る処理など、低レイヤの処理の多くはここで実装されている。[`mem`]
//! で定義された関数も使われているので、併せて読むとよい。値の挿入に関しても、本質パートは
//! [`node`] で実装されている。
//!
//! 値の削除やイテレータの内部実装、検索などの高レイヤの処理は、[`remove`], [`navigate`],
//! [`search`] で実装されている。不変条件を管理するための処理は [`fix`]
//! に書かれている。`split_off()` の実装は [`split`] にある。
//!
//! [`Entry`] は [`map`] 内のモジュールで定義されている。これを読む過程で
//! [`borrow`] を読むことになる。
//!
//! [`map`] および [`set`] では、主に [`BTreeMap`], [`BTreeSet`] の本体と
//! iterator 関連の多数の boilerplate が書かれている。[`set_val`] も合わせて読む。
//! iterator の内部では [`navigate`] で定義されている構造体が用いられる。
//!
//! 残りの [`append`], [`merge_iter`], [`dedup_sorted_iter`]
//! は、append を行う際に用いる構造体が定義されている。
//!
//! [`BTreeMap`]: map::BTreeMap
//! [`BTreeSet`]: set::BTreeSet
//! [`Entry`]: map::Entry

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
