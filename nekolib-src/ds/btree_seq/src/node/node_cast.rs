use std::marker::PhantomData;

use super::{marker, InternalNode, LeafNode, NodeRef};

impl<BorrowType, T, R, Type> NodeRef<BorrowType, T, R, Type> {
    pub fn reborrow(&self) -> NodeRef<marker::Immut<'_>, T, R, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<'a, T, R, Type> NodeRef<marker::Mut<'a>, T, R, Type> {
    pub(super) unsafe fn reborrow_mut(
        &mut self,
    ) -> NodeRef<marker::Mut<'_>, T, R, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
    pub fn dormant(&self) -> NodeRef<marker::DormantMut, T, R, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<T, R, Type> NodeRef<marker::DormantMut, T, R, Type> {
    /// # Safety
    /// The reborrow must have ended, i.e., the reference returned by
    /// `new` and all pointers and references derived from it, must not
    /// be used anymore.
    pub unsafe fn awaken<'a>(self) -> NodeRef<marker::Mut<'a>, T, R, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<T, R, Type> NodeRef<marker::Owned, T, R, Type> {
    pub fn borrow_mut(&mut self) -> NodeRef<marker::Mut<'_>, T, R, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
    pub fn borrow_val_mut(
        &mut self,
    ) -> NodeRef<marker::ValMut<'_>, T, R, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
    pub fn into_dying(self) -> NodeRef<marker::Dying, T, R, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<BorrowType, T, R> NodeRef<BorrowType, T, R, marker::Leaf> {
    pub fn forget_type(
        self,
    ) -> NodeRef<BorrowType, T, R, marker::LeafOrInternal> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<BorrowType, T, R> NodeRef<BorrowType, T, R, marker::Internal> {
    pub fn forget_type(
        self,
    ) -> NodeRef<BorrowType, T, R, marker::LeafOrInternal> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<BorrowType, T, R> NodeRef<BorrowType, T, R, marker::LeafOrInternal> {
    pub(super) unsafe fn cast_to_leaf_unchecked(
        self,
    ) -> NodeRef<BorrowType, T, R, marker::Leaf> {
        debug_assert!(self.height == 0);
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }

    pub(super) unsafe fn cast_to_internal_unchecked(
        self,
    ) -> NodeRef<BorrowType, T, R, marker::Internal> {
        debug_assert!(self.height > 0);
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<BorrowType, T, R, Type> NodeRef<BorrowType, T, R, Type> {
    pub fn as_leaf_ptr(this: &Self) -> *mut LeafNode<T, R> {
        this.node.as_ptr()
    }
}

impl<BorrowType, T, R> NodeRef<BorrowType, T, R, marker::Internal> {
    pub(super) fn as_internal_ptr(this: &Self) -> *mut InternalNode<T, R> {
        this.node.as_ptr() as *mut InternalNode<T, R>
    }
}

impl<'a, T, R> NodeRef<marker::Mut<'a>, T, R, marker::Internal> {
    pub(super) fn as_internal_mut(&mut self) -> &mut InternalNode<T, R> {
        let ptr = Self::as_internal_ptr(self);
        unsafe { &mut *ptr }
    }
}

impl<'a, T, R, Type> NodeRef<marker::Mut<'a>, T, R, Type> {
    pub(super) fn as_leaf_mut(&mut self) -> &mut LeafNode<T, R> {
        let ptr = Self::as_leaf_ptr(self);
        unsafe { &mut *ptr }
    }
}
