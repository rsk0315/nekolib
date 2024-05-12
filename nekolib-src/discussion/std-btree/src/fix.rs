//! fix の実装。
//!
//! 構造体などの定義はなく、[`Handle`] および [`NodeRef`] に対する `impl` をしている。

use crate::node::{marker, Handle, NodeRef, Root};

impl<'a, K: 'a, V: 'a> NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal> {
    fn fix_node_through_parent(
        self,
    ) -> Result<Option<NodeRef<marker::Mut<'a>, K, V, marker::Internal>>, Self>
    {
        todo!()
    }
}

impl<'a, K: 'a, V: 'a> NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal> {
    pub fn fix_node_and_affected_ancestors(mut self) -> bool { todo!() }
}

impl<K, V> Root<K, V> {
    pub fn fix_top(&mut self) { todo!() }

    pub fn fix_right_border(&mut self) { todo!() }

    pub fn fix_left_border(&mut self) { todo!() }

    pub fn fix_right_border_of_plentiful(&mut self) { todo!() }
}

impl<'a, K: 'a, V: 'a>
    Handle<NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal>, marker::KV>
{
    fn fix_left_border_of_left_edge(mut self) { todo!() }

    fn fix_right_border_of_right_edge(mut self) { todo!() }
}

impl<'a, K: 'a, V: 'a>
    Handle<NodeRef<marker::Mut<'a>, K, V, marker::Internal>, marker::KV>
{
    fn fix_left_child(
        self,
    ) -> NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal> {
        todo!()
    }
    fn fix_right_child(
        self,
    ) -> NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal> {
        todo!()
    }
}
