use std::marker::PhantomData;

use super::{marker, Handle, NodeRef};

impl<BorrowType, T, R, NodeType, HandleType>
    Handle<NodeRef<BorrowType, T, R, NodeType>, HandleType>
{
    pub(super) fn reborrow(
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
    pub(super) unsafe fn reborrow_mut(
        &mut self,
    ) -> Handle<NodeRef<marker::Mut<'_>, T, R, NodeType>, HandleType> {
        Handle {
            node: unsafe { self.node.reborrow_mut() },
            idx: self.idx,
            _marker: PhantomData,
        }
    }

    pub(super) fn dormant(
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
    pub(super) unsafe fn awaken<'a>(
        self,
    ) -> Handle<NodeRef<marker::Mut<'a>, T, R, NodeType>, HandleType> {
        Handle {
            node: unsafe { self.node.awaken() },
            idx: self.idx,
            _marker: PhantomData,
        }
    }
}
