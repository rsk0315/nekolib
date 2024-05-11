use crate::node::{
    marker::{self, BorrowType},
    Handle, NodeRef,
};

pub struct LeafRange<BorrowType, K, V> {
    front:
        Option<Handle<NodeRef<BorrowType, K, V, marker::Leaf>, marker::Edge>>,
    back: Option<Handle<NodeRef<BorrowType, K, V, marker::Leaf>, marker::Edge>>,
}

impl<'a, K: 'a, V: 'a> Clone for LeafRange<marker::Immut<'a>, K, V> {
    fn clone(&self) -> Self { todo!() }
}

impl<B, K, V> Default for LeafRange<B, K, V> {
    fn default() -> Self { todo!() }
}

impl<BorrowType, K, V> LeafRange<BorrowType, K, V> {
    pub fn none() -> Self { todo!() }
    fn is_empty(&self) -> bool { todo!() }
    pub fn reborrow(&self) -> LeafRange<marker::Immut<'_>, K, V> { todo!() }
}

impl<'a, K, V> LeafRange<marker::Immut<'a>, K, V> {
    pub fn next_checked(&mut self) -> Option<(&'a K, &'a V)> { todo!() }
    pub fn next_back_checked(&mut self) -> Option<(&'a K, &'a V)> { todo!() }
}

impl<'a, K, V> LeafRange<marker::ValMut<'a>, K, V> {
    pub fn next_checked(&mut self) -> Option<(&'a K, &'a mut V)> { todo!() }
    pub fn next_back_checked(&mut self) -> Option<(&'a K, &'a mut V)> {
        todo!()
    }
}

impl<BorrowType: marker::BorrowType, K, V> LeafRange<BorrowType, K, V> {
    fn perform_next_checked<F, R>(&mut self, f: F) -> Option<R>
    where
        F: Fn(
            &Handle<
                NodeRef<BorrowType, K, V, marker::LeafOrInternal>,
                marker::KV,
            >,
        ) -> R,
    {
        todo!()
    }

    fn perform_next_back_checked<F, R>(&mut self, f: F) -> Option<R>
    where
        F: Fn(
            &Handle<
                NodeRef<BorrowType, K, V, marker::LeafOrInternal>,
                marker::KV,
            >,
        ) -> R,
    {
        todo!()
    }
}

enum LazyLeafHandle<BorrowType, K, V> {
    Root(NodeRef<BorrowType, K, V, marker::LeafOrInternal>),
    Edge(Handle<NodeRef<BorrowType, K, V, marker::Leaf>, marker::Edge>),
}

impl<'a, K: 'a, V: 'a> Clone for LazyLeafHandle<marker::Immut<'a>, K, V> {
    fn clone(&self) -> Self { todo!() }
}

impl<BorrowType, K, V> LazyLeafHandle<BorrowType, K, V> {
    fn reborrow(&self) -> LazyLeafHandle<marker::Immut<'_>, K, V> { todo!() }
}

pub struct LazyLeafRange<BorrowType, K, V> {
    front: Option<LazyLeafHandle<BorrowType, K, V>>,
    back: Option<LazyLeafHandle<BorrowType, K, V>>,
}

impl<B, K, V> Default for LazyLeafRange<B, K, V> {
    fn default() -> Self { todo!() }
}

impl<'a, K: 'a, V: 'a> Clone for LazyLeafRange<marker::Immut<'a>, K, V> {
    fn clone(&self) -> Self { todo!() }
}

impl<BorrowType, K, V> LazyLeafRange<BorrowType, K, V> {
    pub fn none() -> Self { todo!() }
    pub fn reborrow(&self) -> LeafRange<marker::Immut<'_>, K, V> { todo!() }
}

impl<'a, K, V> LazyLeafRange<marker::Immut<'a>, K, V> {
    pub unsafe fn next_unchecked(&mut self) -> (&'a K, &'a V) { todo!() }
    pub unsafe fn next_back_unchecked(&mut self) -> (&'a K, &'a V) { todo!() }
}

impl<'a, K, V> LazyLeafRange<marker::ValMut<'a>, K, V> {
    pub unsafe fn next_unchecked(&mut self) -> (&'a K, &'a mut V) { todo!() }
    pub unsafe fn next_back_unchecked(&mut self) -> (&'a K, &'a mut V) {
        todo!()
    }
}

impl<K, V> LazyLeafRange<marker::Dying, K, V> {
    fn take_front(
        &mut self,
    ) -> Option<Handle<NodeRef<marker::Dying, K, V, marker::Leaf>, marker::Edge>>
    {
        todo!()
    }

    pub unsafe fn deallocating_next_unchecked(
        &mut self,
    ) -> Handle<NodeRef<marker::Dying, K, V, marker::LeafOrInternal>, marker::KV>
    {
        todo!()
    }

    pub unsafe fn deallocating_next_back_unchecked(
        &mut self,
    ) -> Handle<NodeRef<marker::Dying, K, V, marker::LeafOrInternal>, marker::KV>
    {
        todo!()
    }

    pub fn deallocating_end(&mut self) { todo!() }
}
