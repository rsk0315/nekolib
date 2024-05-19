use std::{marker::PhantomData, ptr::NonNull};

use super::{marker, InternalNode, LeafNode, NodeRef, Root};

impl<T, R> NodeRef<marker::Owned, T, R, marker::Leaf> {
    pub fn new_leaf() -> Self { Self::from_new_leaf(LeafNode::new()) }

    pub(super) fn from_new_leaf(leaf: Box<LeafNode<T, R>>) -> Self {
        NodeRef {
            height: 0,
            node: NonNull::from(Box::leak(leaf)),
            _marker: PhantomData,
        }
    }
}

impl<T, R> NodeRef<marker::Owned, T, R, marker::Internal> {
    fn new_internal(child: Root<T, R>) -> Self {
        let mut new_node = unsafe { InternalNode::new() };
        new_node.edges[0].write(child.node);
        unsafe { NodeRef::from_new_internal(new_node, child.height + 1) }
    }

    /// # Safety
    /// `height > 0`.
    pub(super) unsafe fn from_new_internal(
        internal: Box<InternalNode<T, R>>,
        height: u8,
    ) -> Self {
        debug_assert!(height > 0);
        let node = NonNull::from(Box::leak(internal)).cast();
        let mut this = NodeRef { height, node, _marker: PhantomData };
        this.borrow_mut().correct_all_childrens_parent_links();
        this
    }
}

impl<T, R> Root<T, R> {
    pub fn new() -> Self { NodeRef::new_leaf().forget_type() }

    pub fn push_internal_level(
        &mut self,
    ) -> NodeRef<marker::Mut<'_>, T, R, marker::Internal> {
        super::super::mem::take_mut(self, |old_root| {
            NodeRef::new_internal(old_root).forget_type()
        });
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }

    pub fn pop_internal_level(&mut self) {
        assert!(self.height > 0);

        let top = self.node;
        let internal_self =
            unsafe { self.borrow_mut().cast_to_internal_unchecked() };
        let internal_node =
            unsafe { &mut *NodeRef::as_internal_ptr(&internal_self) };
        self.node = unsafe { internal_node.edges[0].assume_init_read() };
        self.height -= 1;
        self.clear_parent_link();

        unsafe { drop(Box::from_raw(internal_node)) };
    }

    fn clear_parent_link(&mut self) {
        let mut root_node = self.borrow_mut();
        let leaf = root_node.as_leaf_mut();
        leaf.parent = None;
    }
}
