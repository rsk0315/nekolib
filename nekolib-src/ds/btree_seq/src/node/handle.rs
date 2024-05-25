use std::{marker::PhantomData, ptr::NonNull};

use super::{marker, Handle, NodeRef};

impl<Node: Copy, Type> Copy for Handle<Node, Type> {}
impl<Node: Copy, Type> Clone for Handle<Node, Type> {
    fn clone(&self) -> Self { *self }
}

impl<BorrowType, T, R, NodeType, HandleType> PartialEq
    for Handle<NodeRef<BorrowType, T, R, NodeType>, HandleType>
{
    fn eq(&self, other: &Self) -> bool {
        let Self { node, idx, .. } = self;
        node.eq(&other.node) && *idx == other.idx
    }
}

impl<BorrowType, T, R, NodeType>
    Handle<NodeRef<BorrowType, T, R, NodeType>, marker::Value>
{
    pub(super) unsafe fn new_value(
        node: NodeRef<BorrowType, T, R, NodeType>,
        idx: usize,
    ) -> Self {
        debug_assert!(idx < node.len());
        Handle { node, idx, _marker: PhantomData }
    }

    pub fn left_edge(
        self,
    ) -> Handle<NodeRef<BorrowType, T, R, NodeType>, marker::Edge> {
        unsafe { Handle::new_edge(self.node, self.idx) }
    }
    pub fn right_edge(
        self,
    ) -> Handle<NodeRef<BorrowType, T, R, NodeType>, marker::Edge> {
        unsafe { Handle::new_edge(self.node, self.idx + 1) }
    }
}

impl<BorrowType, T, R, NodeType>
    Handle<NodeRef<BorrowType, T, R, NodeType>, marker::Edge>
{
    pub(super) unsafe fn new_edge(
        node: NodeRef<BorrowType, T, R, NodeType>,
        idx: usize,
    ) -> Self {
        debug_assert!(idx <= node.len());
        Handle { node, idx, _marker: PhantomData }
    }

    pub fn left_value(
        self,
    ) -> Result<Handle<NodeRef<BorrowType, T, R, NodeType>, marker::Value>, Self>
    {
        if self.idx > 0 {
            Ok(unsafe { Handle::new_value(self.node, self.idx - 1) })
        } else {
            Err(self)
        }
    }
    pub fn right_value(
        self,
    ) -> Result<Handle<NodeRef<BorrowType, T, R, NodeType>, marker::Value>, Self>
    {
        if self.idx < self.node.len() {
            Ok(unsafe { Handle::new_value(self.node, self.idx) })
        } else {
            Err(self)
        }
    }
}

impl<'a, T, R>
    Handle<NodeRef<marker::Mut<'a>, T, R, marker::Internal>, marker::Edge>
{
    pub(super) fn correct_parent_link(self) {
        let ptr = unsafe {
            NonNull::new_unchecked(NodeRef::as_internal_ptr(&self.node))
        };
        let idx = self.idx;
        let mut child = self.descend();
        child.set_parent_link(ptr, idx);
    }
}

impl<BorrowType: marker::Traversable, T, R>
    Handle<NodeRef<BorrowType, T, R, marker::Internal>, marker::Edge>
{
    pub fn descend(self) -> NodeRef<BorrowType, T, R, marker::LeafOrInternal> {
        let parent_ptr = NodeRef::as_internal_ptr(&self.node);
        let node = unsafe {
            (*parent_ptr).edges.get_unchecked(self.idx).assume_init_read()
        };
        NodeRef {
            height: self.node.height - 1,
            node,
            _marker: PhantomData,
        }
    }
}

mod handle_cast;
