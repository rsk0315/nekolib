use std::marker::PhantomData;

use super::super::{marker, Handle, NodeRef};
use crate::node::ForceResult;

impl<BorrowType, T, R, NodeType, HandleType>
    Handle<NodeRef<BorrowType, T, R, NodeType>, HandleType>
{
    pub(in super::super) fn reborrow(
        &self,
    ) -> Handle<NodeRef<marker::Immut<'_>, T, R, NodeType>, HandleType> {
        Handle {
            node: self.node.reborrow(),
            idx: self.idx,
            _marker: PhantomData,
        }
    }
}

impl<'a, T, R, NodeType, HandleType>
    Handle<NodeRef<marker::Mut<'a>, T, R, NodeType>, HandleType>
{
    pub(in super::super) unsafe fn reborrow_mut(
        &mut self,
    ) -> Handle<NodeRef<marker::Mut<'_>, T, R, NodeType>, HandleType> {
        Handle {
            node: unsafe { self.node.reborrow_mut() },
            idx: self.idx,
            _marker: PhantomData,
        }
    }

    pub(in super::super) fn dormant(
        &self,
    ) -> Handle<NodeRef<marker::DormantMut, T, R, NodeType>, HandleType> {
        Handle {
            node: self.node.dormant(),
            idx: self.idx,
            _marker: PhantomData,
        }
    }
}

impl<T, R, NodeType, HandleType>
    Handle<NodeRef<marker::DormantMut, T, R, NodeType>, HandleType>
{
    pub(in super::super) unsafe fn awaken<'a>(
        self,
    ) -> Handle<NodeRef<marker::Mut<'a>, T, R, NodeType>, HandleType> {
        Handle {
            node: unsafe { self.node.awaken() },
            idx: self.idx,
            _marker: PhantomData,
        }
    }
}

impl<BorrowType, T, R>
    Handle<NodeRef<BorrowType, T, R, marker::Leaf>, marker::Edge>
{
    pub fn forget_node_type(
        self,
    ) -> Handle<NodeRef<BorrowType, T, R, marker::LeafOrInternal>, marker::Edge>
    {
        unsafe { Handle::new_edge(self.node.forget_type(), self.idx) }
    }
}

impl<BorrowType, T, R>
    Handle<NodeRef<BorrowType, T, R, marker::Internal>, marker::Edge>
{
    pub fn forget_node_type(
        self,
    ) -> Handle<NodeRef<BorrowType, T, R, marker::LeafOrInternal>, marker::Edge>
    {
        unsafe { Handle::new_edge(self.node.forget_type(), self.idx) }
    }
}

impl<BorrowType, T, R>
    Handle<NodeRef<BorrowType, T, R, marker::Leaf>, marker::Value>
{
    pub fn forget_node_type(
        self,
    ) -> Handle<NodeRef<BorrowType, T, R, marker::LeafOrInternal>, marker::Value>
    {
        unsafe { Handle::new_value(self.node.forget_type(), self.idx) }
    }
}

impl<BorrowType, T, R, Type>
    Handle<NodeRef<BorrowType, T, R, marker::LeafOrInternal>, Type>
{
    pub fn force(
        self,
    ) -> ForceResult<
        Handle<NodeRef<BorrowType, T, R, marker::Leaf>, Type>,
        Handle<NodeRef<BorrowType, T, R, marker::Internal>, Type>,
    > {
        use ForceResult::*;
        match self.node.force() {
            Leaf(node) => {
                Leaf(Handle { node, idx: self.idx, _marker: PhantomData })
            }
            Internal(node) => {
                Internal(Handle { node, idx: self.idx, _marker: PhantomData })
            }
        }
    }
}

impl<Node, Type> Handle<Node, Type> {
    pub fn into_node(self) -> Node { self.node }
    pub fn idx(&self) -> usize { self.idx }
}

impl<'a, T: 'a, R: 'a, NodeType>
    Handle<NodeRef<marker::Immut<'a>, T, R, NodeType>, marker::Value>
{
    pub fn into_val(self) -> &'a T {
        debug_assert!(self.idx < self.node.len());
        let leaf = self.node.into_leaf();
        unsafe { leaf.vals.get_unchecked(self.idx).assume_init_ref() }
    }
}

impl<'a, T, R, NodeType>
    Handle<NodeRef<marker::ValMut<'a>, T, R, NodeType>, marker::Value>
{
    pub fn into_val_valmut(self) -> &'a mut T {
        unsafe { self.node.into_val_mut_at(self.idx) }
    }
}
