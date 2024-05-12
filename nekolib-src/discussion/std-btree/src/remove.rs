//! remove の実装。
//!
//! 構造体などの定義はなく、[`Handle`] および [`NodeRef`] に対する `impl` をしている。

use crate::node::{marker, Handle, NodeRef};

impl<'a, K: 'a, V: 'a>
    Handle<NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal>, marker::KV>
{
    pub fn remove_kv_tracking<F: FnOnce()>(
        self,
        handle_emptied_internal_root: F,
    ) -> (
        (K, V),
        Handle<NodeRef<marker::Mut<'a>, K, V, marker::Leaf>, marker::Edge>,
    ) {
        todo!()
    }
}

impl<'a, K: 'a, V: 'a>
    Handle<NodeRef<marker::Mut<'a>, K, V, marker::Leaf>, marker::KV>
{
    fn remove_leaf_kv<F: FnOnce()>(
        self,
        handle_emptied_internal_root: F,
    ) -> (
        (K, V),
        Handle<NodeRef<marker::Mut<'a>, K, V, marker::Leaf>, marker::Edge>,
    ) {
        todo!()
    }
}

impl<'a, K: 'a, V: 'a>
    Handle<NodeRef<marker::Mut<'a>, K, V, marker::Internal>, marker::KV>
{
    fn remove_internal_kv<F: FnOnce()>(
        self,
        handle_emptied_internal_root: F,
    ) -> (
        (K, V),
        Handle<NodeRef<marker::Mut<'a>, K, V, marker::Leaf>, marker::Edge>,
    ) {
        todo!()
    }
}
