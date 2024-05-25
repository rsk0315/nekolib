use std::{hint, ops::RangeBounds, ptr};

use super::{
    marker,
    ForceResult::{Internal, Leaf},
    Handle, NodeRef,
};

pub struct LeafRange<BorrowType, T, R> {
    front:
        Option<Handle<NodeRef<BorrowType, T, R, marker::Leaf>, marker::Edge>>,
    back: Option<Handle<NodeRef<BorrowType, T, R, marker::Leaf>, marker::Edge>>,
}

impl<'a, T: 'a, R: 'a> Clone for LeafRange<marker::Immut<'a>, T, R> {
    fn clone(&self) -> Self {
        LeafRange {
            front: self.front.clone(),
            back: self.back.clone(),
        }
    }
}

impl<BorrowType, T, R> Default for LeafRange<BorrowType, T, R> {
    fn default() -> Self { Self::none() }
}

impl<BorrowType, T, R> LeafRange<BorrowType, T, R> {
    pub fn none() -> Self { Self { front: None, back: None } }

    fn is_empty(&self) -> bool { self.front == self.back }

    pub fn reborrow(&self) -> LeafRange<marker::Immut<'_>, T, R> {
        LeafRange {
            front: self.front.as_ref().map(|h| h.reborrow()),
            back: self.back.as_ref().map(|h| h.reborrow()),
        }
    }
}

impl<'a, T, R: 'a> LeafRange<marker::Immut<'a>, T, R> {
    #[inline]
    pub fn next_checked(&mut self) -> Option<&'a T> {
        self.perform_next_checked(|h| h.into_val())
    }
    #[inline]
    pub fn next_back_checked(&mut self) -> Option<&'a T> {
        self.perform_next_back_checked(|h| h.into_val())
    }
}

impl<'a, T, R> LeafRange<marker::ValMut<'a>, T, R> {
    #[inline]
    pub fn next_checked(&mut self) -> Option<&'a mut T> {
        self.perform_next_checked(|h| unsafe { ptr::read(h) }.into_val_valmut())
    }
    #[inline]
    pub fn next_back_checked(&mut self) -> Option<&'a mut T> {
        self.perform_next_back_checked(|h| {
            unsafe { ptr::read(h) }.into_val_valmut()
        })
    }
}

impl<BorrowType: marker::Traversable, T, R> LeafRange<BorrowType, T, R> {
    fn perform_next_checked<F, Ret>(&mut self, f: F) -> Option<Ret>
    where
        F: Fn(
            &Handle<
                NodeRef<BorrowType, T, R, marker::LeafOrInternal>,
                marker::Value,
            >,
        ) -> Ret,
    {
        if self.is_empty() {
            None
        } else {
            super::super::mem::replace(self.front.as_mut().unwrap(), |front| {
                let h = front.next_value().ok().unwrap();
                let res = f(&h);
                (h.next_leaf_edge(), Some(res))
            })
        }
    }
    fn perform_next_back_checked<F, Ret>(&mut self, f: F) -> Option<Ret>
    where
        F: Fn(
            &Handle<
                NodeRef<BorrowType, T, R, marker::LeafOrInternal>,
                marker::Value,
            >,
        ) -> Ret,
    {
        if self.is_empty() {
            None
        } else {
            super::super::mem::replace(self.front.as_mut().unwrap(), |back| {
                let h = back.next_back_value().ok().unwrap();
                let res = f(&h);
                (h.next_back_leaf_edge(), Some(res))
            })
        }
    }
}

enum LazyLeafHandle<BorrowType, T, R> {
    Root(NodeRef<BorrowType, T, R, marker::LeafOrInternal>), // not yet descended
    Edge(Handle<NodeRef<BorrowType, T, R, marker::Leaf>, marker::Edge>),
}

impl<'a, T: 'a, R: 'a> Clone for LazyLeafHandle<marker::Immut<'a>, T, R> {
    fn clone(&self) -> Self {
        match self {
            LazyLeafHandle::Root(root) => LazyLeafHandle::Root(*root),
            LazyLeafHandle::Edge(edge) => LazyLeafHandle::Edge(*edge),
        }
    }
}

impl<BorrowType, T, R> LazyLeafHandle<BorrowType, T, R> {
    fn reborrow(&self) -> LazyLeafHandle<marker::Immut<'_>, T, R> {
        match self {
            LazyLeafHandle::Root(root) => LazyLeafHandle::Root(root.reborrow()),
            LazyLeafHandle::Edge(edge) => LazyLeafHandle::Edge(edge.reborrow()),
        }
    }
}

pub struct LazyLeafRange<BorrowType, T, R> {
    front: Option<LazyLeafHandle<BorrowType, T, R>>,
    back: Option<LazyLeafHandle<BorrowType, T, R>>,
}

impl<BorrowType, T, R> Default for LazyLeafRange<BorrowType, T, R> {
    fn default() -> Self { Self::none() }
}

impl<'a, T: 'a, R: 'a> Clone for LazyLeafRange<marker::Immut<'a>, T, R> {
    fn clone(&self) -> Self {
        Self {
            front: self.front.clone(),
            back: self.back.clone(),
        }
    }
}

impl<BorrowType, T, R> LazyLeafRange<BorrowType, T, R> {
    pub fn none() -> Self { Self { front: None, back: None } }

    pub fn reborrow(&self) -> LazyLeafRange<marker::Immut<'_>, T, R> {
        LazyLeafRange {
            front: self.front.as_ref().map(|h| h.reborrow()),
            back: self.back.as_ref().map(|h| h.reborrow()),
        }
    }
}

impl<'a, T, R: 'a> LazyLeafRange<marker::Immut<'a>, T, R> {
    #[inline]
    pub unsafe fn next_unchecked(&mut self) -> &'a T {
        unsafe { self.init_front().unwrap().next_unchecked() }
    }
    #[inline]
    pub unsafe fn next_back_unchecked(&mut self) -> &'a T {
        unsafe { self.init_back().unwrap().next_back_unchecked() }
    }
}

impl<'a, T, R> LazyLeafRange<marker::ValMut<'a>, T, R> {
    #[inline]
    pub unsafe fn next_unchecked(&mut self) -> &'a mut T {
        unsafe { self.init_front().unwrap().next_unchecked() }
    }
    #[inline]
    pub unsafe fn next_back_unchecked(&mut self) -> &'a mut T {
        unsafe { self.init_back().unwrap().next_back_unchecked() }
    }
}

impl<T, R> LazyLeafRange<marker::Dying, T, R> {
    fn take_front(
        &mut self,
    ) -> Option<Handle<NodeRef<marker::Dying, T, R, marker::Leaf>, marker::Edge>>
    {
        match self.front.take()? {
            LazyLeafHandle::Root(root) => Some(root.first_leaf_edge()),
            LazyLeafHandle::Edge(edge) => Some(edge),
        }
    }

    #[inline]
    pub unsafe fn deallocating_next_unchecked(
        &mut self,
    ) -> Handle<
        NodeRef<marker::Dying, T, R, marker::LeafOrInternal>,
        marker::Value,
    > {
        debug_assert!(self.front.is_some());
        let front = self.init_front().unwrap();
        unsafe { front.deallocating_next_unchecked() }
    }
    #[inline]
    pub unsafe fn deallocating_next_back_unchecked(
        &mut self,
    ) -> Handle<
        NodeRef<marker::Dying, T, R, marker::LeafOrInternal>,
        marker::Value,
    > {
        debug_assert!(self.back.is_some());
        let back = self.init_back().unwrap();
        unsafe { back.deallocating_next_back_unchecked() }
    }
    #[inline]
    pub fn deallocating_end(&mut self) {
        if let Some(front) = self.take_front() {
            front.deallocating_end()
        }
    }
}

impl<BorrowType: marker::Traversable, T, R> LazyLeafRange<BorrowType, T, R> {
    fn init_front(
        &mut self,
    ) -> Option<
        &mut Handle<NodeRef<BorrowType, T, R, marker::Leaf>, marker::Edge>,
    > {
        if let Some(LazyLeafHandle::Root(root)) = &self.front {
            self.front = Some(LazyLeafHandle::Edge(
                unsafe { ptr::read(root) }.first_leaf_edge(),
            ));
        }
        match &mut self.front {
            None => None,
            Some(LazyLeafHandle::Edge(edge)) => Some(edge),
            Some(LazyLeafHandle::Root(_)) => unsafe {
                hint::unreachable_unchecked()
            },
        }
    }
    fn init_back(
        &mut self,
    ) -> Option<
        &mut Handle<NodeRef<BorrowType, T, R, marker::Leaf>, marker::Edge>,
    > {
        if let Some(LazyLeafHandle::Root(root)) = &self.back {
            self.back = Some(LazyLeafHandle::Edge(
                unsafe { ptr::read(root) }.last_leaf_edge(),
            ));
        }
        match &mut self.back {
            None => None,
            Some(LazyLeafHandle::Edge(edge)) => Some(edge),
            Some(LazyLeafHandle::Root(_)) => unsafe {
                hint::unreachable_unchecked()
            },
        }
    }
}

impl<BorrowType: marker::Traversable, T, R>
    NodeRef<BorrowType, T, R, marker::LeafOrInternal>
{
    // used by `BTreeSeq::range(..)`.
    unsafe fn find_leaf_edges_spanning_range<B>(
        self,
        range: B,
    ) -> LeafRange<BorrowType, T, R>
    where
        B: RangeBounds<usize>,
    {
        todo!()
    }
}

fn full_range<BorrowType: marker::Traversable, T, R>(
    root1: NodeRef<BorrowType, T, R, marker::LeafOrInternal>,
    root2: NodeRef<BorrowType, T, R, marker::LeafOrInternal>,
) -> LazyLeafRange<BorrowType, T, R> {
    LazyLeafRange {
        front: Some(LazyLeafHandle::Root(root1)),
        back: Some(LazyLeafHandle::Root(root2)),
    }
}

impl<'a, T: 'a, R: 'a>
    NodeRef<marker::Immut<'a>, T, R, marker::LeafOrInternal>
{
    pub fn range_search<B>(self, range: B) -> LeafRange<marker::Immut<'a>, T, R>
    where
        B: RangeBounds<usize>,
    {
        unsafe { self.find_leaf_edges_spanning_range(range) }
    }
    pub fn full_range(self) -> LazyLeafRange<marker::Immut<'a>, T, R> {
        full_range(self, self)
    }
}

impl<'a, T: 'a, R: 'a>
    NodeRef<marker::ValMut<'a>, T, R, marker::LeafOrInternal>
{
    pub fn range_search<B>(
        self,
        range: B,
    ) -> LeafRange<marker::ValMut<'a>, T, R>
    where
        B: RangeBounds<usize>,
    {
        unsafe { self.find_leaf_edges_spanning_range(range) }
    }
    pub fn full_range(self) -> LazyLeafRange<marker::ValMut<'a>, T, R> {
        // We duplicate the root NodeRef here -- we will never visit the
        // same value twice, and never end up with overlapping value
        // references.
        let self2 = unsafe { ptr::read(&self) };
        full_range(self, self2)
    }
}

impl<T, R> NodeRef<marker::Dying, T, R, marker::LeafOrInternal> {
    pub fn full_range(self) -> LazyLeafRange<marker::Dying, T, R> {
        // We duplicate the root NodeRef here -- we will never access it
        // in a way that overlaps references obtained from the root.
        let self2 = unsafe { ptr::read(&self) };
        full_range(self, self2)
    }
}

impl<BorrowType: marker::Traversable, T, R>
    Handle<NodeRef<BorrowType, T, R, marker::Leaf>, marker::Edge>
{
    pub fn next_value(
        self,
    ) -> Result<
        Handle<
            NodeRef<BorrowType, T, R, marker::LeafOrInternal>,
            marker::Value,
        >,
        NodeRef<BorrowType, T, R, marker::LeafOrInternal>,
    > {
        let mut edge = self.forget_node_type();
        loop {
            edge = match edge.right_value() {
                Ok(value) => return Ok(value),
                Err(last_edge) => match last_edge.into_node().ascend() {
                    Ok(parent_edge) => parent_edge.forget_node_type(),
                    Err(root) => return Err(root),
                },
            }
        }
    }
    pub fn next_back_value(
        self,
    ) -> Result<
        Handle<
            NodeRef<BorrowType, T, R, marker::LeafOrInternal>,
            marker::Value,
        >,
        NodeRef<BorrowType, T, R, marker::LeafOrInternal>,
    > {
        let mut edge = self.forget_node_type();
        loop {
            edge = match edge.left_value() {
                Ok(value) => return Ok(value),
                Err(first_edge) => match first_edge.into_node().ascend() {
                    Ok(parent_edge) => parent_edge.forget_node_type(),
                    Err(root) => return Err(root),
                },
            }
        }
    }
}

impl<BorrowType: marker::Traversable, T, R>
    Handle<NodeRef<BorrowType, T, R, marker::Internal>, marker::Edge>
{
    fn next_value(
        self,
    ) -> Result<
        Handle<NodeRef<BorrowType, T, R, marker::Internal>, marker::Value>,
        NodeRef<BorrowType, T, R, marker::Internal>,
    > {
        let mut edge = self;
        loop {
            edge = match edge.right_value() {
                Ok(internal_value) => return Ok(internal_value),
                Err(last_edge) => match last_edge.into_node().ascend() {
                    Ok(parent_edge) => parent_edge,
                    Err(root) => return Err(root),
                },
            }
        }
    }
}

impl<T, R> Handle<NodeRef<marker::Dying, T, R, marker::Leaf>, marker::Edge> {
    unsafe fn deallocating_next(
        self,
    ) -> Option<(
        Self,
        Handle<
            NodeRef<marker::Dying, T, R, marker::LeafOrInternal>,
            marker::Value,
        >,
    )> {
        let mut edge = self.forget_node_type();
        loop {
            edge = match edge.right_value() {
                Ok(value) => {
                    return Some((
                        unsafe { ptr::read(&value) }.next_leaf_edge(),
                        value,
                    ));
                }
                Err(last_edge) => match unsafe {
                    last_edge.into_node().deallocate_and_ascend()
                } {
                    Some(parent_edge) => parent_edge.forget_node_type(),
                    None => return None,
                },
            }
        }
    }
    unsafe fn deallocating_next_back(
        self,
    ) -> Option<(
        Self,
        Handle<
            NodeRef<marker::Dying, T, R, marker::LeafOrInternal>,
            marker::Value,
        >,
    )> {
        let mut edge = self.forget_node_type();
        loop {
            edge = match edge.left_value() {
                Ok(value) => {
                    return Some((
                        unsafe { ptr::read(&value) }.next_back_leaf_edge(),
                        value,
                    ));
                }
                Err(last_edge) => match unsafe {
                    last_edge.into_node().deallocate_and_ascend()
                } {
                    Some(parent_edge) => parent_edge.forget_node_type(),
                    None => return None,
                },
            }
        }
    }
    fn deallocating_end(self) {
        let mut edge = self.forget_node_type();
        while let Some(parent_edge) =
            unsafe { edge.into_node().deallocate_and_ascend() }
        {
            edge = parent_edge.forget_node_type();
        }
    }
}

impl<'a, T, R: 'a>
    Handle<NodeRef<marker::Immut<'a>, T, R, marker::Leaf>, marker::Edge>
{
    /// # Safety
    /// There must be another value in the direction travelled.
    unsafe fn next_unchecked(&mut self) -> &'a T {
        super::super::mem::replace(self, |leaf_edge| {
            let h = leaf_edge.next_value().ok().unwrap();
            (h.next_leaf_edge(), h.into_val())
        })
    }

    /// # Safety
    /// There must be another value in the direction travelled.
    unsafe fn next_back_unchecked(&mut self) -> &'a T {
        super::super::mem::replace(self, |leaf_edge| {
            let h = leaf_edge.next_back_value().ok().unwrap();
            (h.next_back_leaf_edge(), h.into_val())
        })
    }
}

impl<'a, T, R>
    Handle<NodeRef<marker::ValMut<'a>, T, R, marker::Leaf>, marker::Edge>
{
    /// # Safety
    /// There must be another value in the direction travelled.
    unsafe fn next_unchecked(&mut self) -> &'a mut T {
        let h = super::super::mem::replace(self, |leaf_edge| {
            let h = leaf_edge.next_value().ok().unwrap();
            (unsafe { ptr::read(&h) }.next_leaf_edge(), h)
        });
        h.into_val_valmut()
    }

    /// # Safety
    /// There must be another value in the direction travelled.
    unsafe fn next_back_unchecked(&mut self) -> &'a mut T {
        let h = super::super::mem::replace(self, |leaf_edge| {
            let h = leaf_edge.next_back_value().ok().unwrap();
            (unsafe { ptr::read(&h) }.next_back_leaf_edge(), h)
        });
        h.into_val_valmut()
    }
}

impl<T, R> Handle<NodeRef<marker::Dying, T, R, marker::Leaf>, marker::Edge> {
    /// # Safety
    /// - There must be another value in the direction travelled.
    /// - That value was not previously returned by counterpart
    ///   `deallocating_next_back_unchecked` on any copy of the handles
    ///   being used to traverse the tree.
    unsafe fn deallocating_next_unchecked(
        &mut self,
    ) -> Handle<
        NodeRef<marker::Dying, T, R, marker::LeafOrInternal>,
        marker::Value,
    > {
        super::super::mem::replace(self, |leaf_edge| unsafe {
            leaf_edge.deallocating_next().unwrap()
        })
    }
    /// # Safety
    /// - There must be another value in the direction travelled.
    /// - That value was not previously returned by counterpart
    ///   `deallocating_next_unchecked` on any copy of the handles being
    ///   used to traverse the tree.
    unsafe fn deallocating_next_back_unchecked(
        &mut self,
    ) -> Handle<
        NodeRef<marker::Dying, T, R, marker::LeafOrInternal>,
        marker::Value,
    > {
        super::super::mem::replace(self, |leaf_edge| unsafe {
            leaf_edge.deallocating_next_back().unwrap()
        })
    }
}

impl<BorrowType: marker::Traversable, T, R>
    NodeRef<BorrowType, T, R, marker::LeafOrInternal>
{
    #[inline]
    pub fn first_leaf_edge(
        self,
    ) -> Handle<NodeRef<BorrowType, T, R, marker::Leaf>, marker::Edge> {
        let mut node = self;
        loop {
            match node.force() {
                Leaf(leaf) => return leaf.first_edge(),
                Internal(internal) => node = internal.first_edge().descend(),
            }
        }
    }
    #[inline]
    pub fn last_leaf_edge(
        self,
    ) -> Handle<NodeRef<BorrowType, T, R, marker::Leaf>, marker::Edge> {
        let mut node = self;
        loop {
            match node.force() {
                Leaf(leaf) => return leaf.last_edge(),
                Internal(internal) => node = internal.last_edge().descend(),
            }
        }
    }
}

impl<BorrowType: marker::Traversable, T, R>
    Handle<NodeRef<BorrowType, T, R, marker::LeafOrInternal>, marker::Value>
{
    pub fn next_leaf_edge(
        self,
    ) -> Handle<NodeRef<BorrowType, T, R, marker::Leaf>, marker::Edge> {
        match self.force() {
            Leaf(leaf) => leaf.right_edge(),
            Internal(internal_value) => {
                let next_internal_value = internal_value.right_edge();
                next_internal_value.descend().first_leaf_edge()
            }
        }
    }
    pub fn next_back_leaf_edge(
        self,
    ) -> Handle<NodeRef<BorrowType, T, R, marker::Leaf>, marker::Edge> {
        match self.force() {
            Leaf(leaf) => leaf.left_edge(),
            Internal(internal_value) => {
                let next_back_internal_value = internal_value.left_edge();
                next_back_internal_value.descend().last_leaf_edge()
            }
        }
    }
}

// Note: when we create the `ValMut` iterator [first..last) and drop it,
// ending up with [first_end..last_end), we should maintain `reduced`
// field in range [first..first_end) and [last_end..last).
