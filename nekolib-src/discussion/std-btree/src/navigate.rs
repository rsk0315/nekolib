//! [`LeafRange`] と [`LazyLeafRange`] の定義。

use std::{borrow::Borrow, ops::RangeBounds};

use crate::{
    node::{marker, Handle, NodeRef},
    search::SearchBound,
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

impl<BorrowType: marker::BorrowType, K, V> LazyLeafRange<BorrowType, K, V> {
    fn init_front(
        &mut self,
    ) -> Option<
        &mut Handle<NodeRef<BorrowType, K, V, marker::Leaf>, marker::Edge>,
    > {
        todo!()
    }

    fn init_back(
        &mut self,
    ) -> Option<
        &mut Handle<NodeRef<BorrowType, K, V, marker::Leaf>, marker::Edge>,
    > {
        todo!()
    }
}

impl<BorrowType: marker::BorrowType, K, V>
    NodeRef<BorrowType, K, V, marker::LeafOrInternal>
{
    unsafe fn find_leaf_edges_spanning_range<Q: ?Sized, R>(
        self,
        range: R,
    ) -> LeafRange<BorrowType, K, V>
    where
        Q: Ord,
        K: Borrow<Q>,
        R: RangeBounds<Q>,
    {
        todo!()
    }
}

fn full_range<BorrowType: marker::BorrowType, K, V>(
    root1: NodeRef<BorrowType, K, V, marker::LeafOrInternal>,
    root2: NodeRef<BorrowType, K, V, marker::LeafOrInternal>,
) -> LazyLeafRange<BorrowType, K, V> {
    todo!()
}

impl<'a, K: 'a, V: 'a>
    NodeRef<marker::Immut<'a>, K, V, marker::LeafOrInternal>
{
    pub fn range_search<Q, R>(
        self,
        range: R,
    ) -> LeafRange<marker::Immut<'a>, K, V>
    where
        Q: ?Sized + Ord,
        K: Borrow<Q>,
        R: RangeBounds<Q>,
    {
        todo!()
    }

    pub fn full_range(self) -> LazyLeafRange<marker::Immut<'a>, K, V> {
        todo!()
    }
}

impl<'a, K: 'a, V: 'a>
    NodeRef<marker::ValMut<'a>, K, V, marker::LeafOrInternal>
{
    pub fn range_search<Q, R>(
        self,
        range: R,
    ) -> LeafRange<marker::ValMut<'a>, K, V>
    where
        Q: ?Sized + Ord,
        K: Borrow<Q>,
        R: RangeBounds<Q>,
    {
        todo!()
    }

    pub fn full_range(self) -> LazyLeafRange<marker::ValMut<'a>, K, V> {
        todo!()
    }
}

impl<K, V> NodeRef<marker::Dying, K, V, marker::LeafOrInternal> {
    pub fn full_range(self) -> LazyLeafRange<marker::Dying, K, V> { todo!() }
}

impl<BorrowType: marker::BorrowType, K, V>
    Handle<NodeRef<BorrowType, K, V, marker::Leaf>, marker::Edge>
{
    pub fn next_kv(
        self,
    ) -> Result<
        Handle<NodeRef<BorrowType, K, V, marker::LeafOrInternal>, marker::KV>,
        NodeRef<BorrowType, K, V, marker::LeafOrInternal>,
    > {
        todo!()
    }

    pub fn next_back_kv(
        self,
    ) -> Result<
        Handle<NodeRef<BorrowType, K, V, marker::LeafOrInternal>, marker::KV>,
        NodeRef<BorrowType, K, V, marker::LeafOrInternal>,
    > {
        todo!()
    }
}

impl<BorrowType: marker::BorrowType, K, V>
    Handle<NodeRef<BorrowType, K, V, marker::Internal>, marker::Edge>
{
    fn next_kv(
        self,
    ) -> Result<
        Handle<NodeRef<BorrowType, K, V, marker::Internal>, marker::KV>,
        NodeRef<BorrowType, K, V, marker::Internal>,
    > {
        todo!()
    }
}

impl<K, V> Handle<NodeRef<marker::Dying, K, V, marker::Leaf>, marker::Edge> {
    unsafe fn deallocating_next(
        self,
    ) -> Option<(
        Self,
        Handle<
            NodeRef<marker::Dying, K, V, marker::LeafOrInternal>,
            marker::KV,
        >,
    )> {
        todo!()
    }

    unsafe fn deallocating_next_back(
        self,
    ) -> Option<(
        Self,
        Handle<
            NodeRef<marker::Dying, K, V, marker::LeafOrInternal>,
            marker::KV,
        >,
    )> {
        todo!()
    }

    fn deallocating_end(self) { todo!() }
}

impl<'a, K, V>
    Handle<NodeRef<marker::Immut<'a>, K, V, marker::Leaf>, marker::Edge>
{
    unsafe fn next_unchecked(&mut self) -> (&'a K, &'a V) { todo!() }
    unsafe fn next_back_unchecked(&mut self) -> (&'a K, &'a V) { todo!() }
}

impl<'a, K, V>
    Handle<NodeRef<marker::ValMut<'a>, K, V, marker::Leaf>, marker::Edge>
{
    unsafe fn next_unchecked(&mut self) -> (&'a K, &'a mut V) { todo!() }
    unsafe fn next_back_unchecked(&mut self) -> (&'a K, &'a mut V) { todo!() }
}

impl<'a, K, V>
    Handle<NodeRef<marker::Dying, K, V, marker::Leaf>, marker::Edge>
{
    unsafe fn deallocacting_next_unchecked(
        &mut self,
    ) -> Handle<NodeRef<marker::Dying, K, V, marker::LeafOrInternal>, marker::KV>
    {
        todo!()
    }
    unsafe fn deallocacting_next_back_unchecked(
        &mut self,
    ) -> Handle<NodeRef<marker::Dying, K, V, marker::LeafOrInternal>, marker::KV>
    {
        todo!()
    }
}

impl<BorrowType: marker::BorrowType, K, V>
    NodeRef<BorrowType, K, V, marker::LeafOrInternal>
{
    pub fn first_leaf_edge(
        self,
    ) -> Handle<NodeRef<BorrowType, K, V, marker::Leaf>, marker::Edge> {
        todo!()
    }
    pub fn last_leaf_edge(
        self,
    ) -> Handle<NodeRef<BorrowType, K, V, marker::Leaf>, marker::Edge> {
        todo!()
    }
}

pub enum Position<BorrowType, K, V> {
    Leaf(NodeRef<BorrowType, K, V, marker::Leaf>),
    Internal(NodeRef<BorrowType, K, V, marker::Internal>),
    InternalKV(Handle<NodeRef<BorrowType, K, V, marker::Internal>, marker::KV>),
}

impl<'a, K: 'a, V: 'a>
    NodeRef<marker::Immut<'a>, K, V, marker::LeafOrInternal>
{
    pub fn visit_nodes_in_order<F>(self, mut visit: F)
    where
        F: FnMut(Position<marker::Immut<'a>, K, V>),
    {
        todo!()
    }

    pub fn calc_length(self) -> usize { todo!() }
}

impl<BorrowType: marker::BorrowType, K, V>
    Handle<NodeRef<BorrowType, K, V, marker::LeafOrInternal>, marker::KV>
{
    pub fn next_leaf_edge(
        self,
    ) -> Handle<NodeRef<BorrowType, K, V, marker::Leaf>, marker::Edge> {
        todo!()
    }
    pub fn next_back_leaf_edge(
        self,
    ) -> Handle<NodeRef<BorrowType, K, V, marker::Leaf>, marker::Edge> {
        todo!()
    }
}

impl<BorrowType: marker::BorrowType, K, V>
    NodeRef<BorrowType, K, V, marker::LeafOrInternal>
{
    pub fn lower_bound<Q: ?Sized>(
        self,
        mut bound: SearchBound<&Q>,
    ) -> Handle<NodeRef<BorrowType, K, V, marker::Leaf>, marker::Edge>
    where
        Q: Ord,
        K: Borrow<Q>,
    {
        todo!()
    }
    pub fn upper_bound<Q: ?Sized>(
        self,
        mut bound: SearchBound<&Q>,
    ) -> Handle<NodeRef<BorrowType, K, V, marker::Leaf>, marker::Edge>
    where
        Q: Ord,
        K: Borrow<Q>,
    {
        todo!()
    }
}
