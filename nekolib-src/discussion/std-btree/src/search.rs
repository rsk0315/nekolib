//! search の実装。
//!
//! [`SearchBound`] など構造体の定義も含むが、主には [`NodeRef`] への `impl` を行う。

use std::{
    borrow::Borrow,
    ops::{Bound, RangeBounds},
};

use crate::node::{marker, Handle, NodeRef};

pub enum SearchBound<T> {
    Included(T),
    Excluded(T),
    AllIncluded,
    AllExcluded,
}

impl<T> SearchBound<T> {
    pub fn from_range(range_bound: Bound<T>) -> Self { todo!() }
}

pub enum SearchResult<BorrowType, K, V, FoundType, GoDownType> {
    Found(Handle<NodeRef<BorrowType, K, V, FoundType>, marker::KV>),
    GoDown(Handle<NodeRef<BorrowType, K, V, GoDownType>, marker::Edge>),
}

pub enum IndexResult {
    KV(usize),
    Edge(usize),
}

impl<BorrowType: marker::BorrowType, K, V>
    NodeRef<BorrowType, K, V, marker::LeafOrInternal>
{
    pub fn search_tree<Q: ?Sized>(
        mut self,
        key: &Q,
    ) -> SearchResult<BorrowType, K, V, marker::LeafOrInternal, marker::Leaf>
    where
        Q: Ord,
        K: Borrow<Q>,
    {
        todo!()
    }

    pub fn search_tree_for_bifurcation<'r, Q: ?Sized, R>(
        mut self,
        range: &'r R,
    ) -> Result<
        (
            NodeRef<BorrowType, K, V, marker::LeafOrInternal>,
            usize,
            usize,
            SearchBound<&'r Q>,
            SearchBound<&'r Q>,
        ),
        Handle<NodeRef<BorrowType, K, V, marker::Leaf>, marker::Edge>,
    >
    where
        Q: Ord,
        K: Borrow<Q>,
        R: RangeBounds<Q>,
    {
        todo!()
    }

    pub fn find_lower_bound_edge<'r, Q>(
        self,
        bound: SearchBound<&'r Q>,
    ) -> (Handle<Self, marker::Edge>, SearchBound<&'r Q>)
    where
        Q: ?Sized + Ord,
        K: Borrow<Q>,
    {
        todo!()
    }
    pub fn find_upper_bound_edge<'r, Q>(
        self,
        bound: SearchBound<&'r Q>,
    ) -> (Handle<Self, marker::Edge>, SearchBound<&'r Q>)
    where
        Q: ?Sized + Ord,
        K: Borrow<Q>,
    {
        todo!()
    }
}

impl<BorrowType, K, V, Type> NodeRef<BorrowType, K, V, Type> {
    pub fn seaearch_node<Q: ?Sized>(
        self,
        key: &Q,
    ) -> SearchResult<BorrowType, K, V, Type, Type>
    where
        Q: Ord,
        K: Borrow<Q>,
    {
        todo!()
    }

    unsafe fn find_key_index<Q: ?Sized>(
        &self,
        key: &Q,
        start_index: usize,
    ) -> IndexResult
    where
        Q: Ord,
        K: Borrow<Q>,
    {
        todo!()
    }

    fn find_lower_bound_index<'r, Q>(
        &self,
        bound: SearchBound<&'r Q>,
    ) -> (usize, SearchBound<&'r Q>)
    where
        Q: ?Sized + Ord,
        K: Borrow<Q>,
    {
        todo!()
    }
    unsafe fn find_upper_bound_index<'r, Q>(
        &self,
        bound: SearchBound<&'r Q>,
        start_index: usize,
    ) -> (usize, SearchBound<&'r Q>)
    where
        Q: ?Sized + Ord,
        K: Borrow<Q>,
    {
        todo!()
    }
}
