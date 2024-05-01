use std::{
    fmt,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Index, IndexMut},
    ptr::{self, NonNull},
};

use array_insertion::{array_insert, array_splice};
use array_removal::array_remove;
use array_rotation::{array_rotate_2, array_rotate_3};

const B: usize = 4;
const CAPACITY: usize = 2 * B - 1;
const MIN_BUFLEN: usize = B - 1;

struct LeafNode<T> {
    buflen: u8,
    buf: [MaybeUninit<T>; CAPACITY],
    parent: Option<(NonNull<InternalNode<T>>, u8)>,
}

#[repr(C)]
struct InternalNode<T> {
    data: LeafNode<T>,
    treelen: usize,
    children: [MaybeUninit<NonNull<LeafNode<T>>>; CAPACITY + 1],
}

struct NodeRef<BorrowType, T, NodeType> {
    node: NonNull<LeafNode<T>>,
    height: u8,
    _marker: PhantomData<(BorrowType, T, NodeType)>,
}

enum ForceResult<BorrowType, T> {
    Leaf(NodeRef<BorrowType, T, marker::Leaf>),
    Internal(NodeRef<BorrowType, T, marker::Internal>),
}

pub struct BTreeSeq<T> {
    root: Option<OwnedNodeRef<T>>,
}

mod marker {
    use std::marker::PhantomData;

    pub enum Owned {}
    pub enum Dying {}
    pub struct Immut<'a>(PhantomData<&'a ()>);
    pub struct ValMut<'a>(PhantomData<&'a mut ()>);
    pub struct Mut<'a>(PhantomData<&'a mut ()>);

    pub enum Leaf {}
    pub enum Internal {}
    pub enum LeafOrInternal {}

    pub enum Value {}
    pub enum Edge {}
}

trait Traversable {}
impl Traversable for marker::Dying {}
impl<'a> Traversable for marker::Immut<'a> {}
impl<'a> Traversable for marker::ValMut<'a> {}
impl<'a> Traversable for marker::Mut<'a> {}

impl<T, NodeType> Copy for NodeRef<marker::Immut<'_>, T, NodeType> {}
impl<'a, T, NodeType> Clone for NodeRef<marker::Immut<'a>, T, NodeType> {
    fn clone(&self) -> Self { unsafe { self.cast() } }
}

type OwnedNodeRef<T> = NodeRef<marker::Owned, T, marker::LeafOrInternal>;
type MutNodeRef<'a, T> = NodeRef<marker::Mut<'a>, T, marker::LeafOrInternal>;
type MutLeafNodeRef<'a, T> = NodeRef<marker::Mut<'a>, T, marker::Leaf>;
type MutInternalNodeRef<'a, T> = NodeRef<marker::Mut<'a>, T, marker::Internal>;
type ImmutNodeRef<'a, T> =
    NodeRef<marker::Immut<'a>, T, marker::LeafOrInternal>;
type ValMutNodeRef<'a, T> =
    NodeRef<marker::ValMut<'a>, T, marker::LeafOrInternal>;
type DyingNodeRef<T> = NodeRef<marker::Dying, T, marker::LeafOrInternal>;

impl<T> LeafNode<T> {
    unsafe fn init(this: *mut Self) {
        unsafe {
            ptr::addr_of_mut!((*this).buflen).write(0);
            ptr::addr_of_mut!((*this).parent).write(None);
        }
    }
    pub fn new() -> Box<Self> {
        unsafe {
            let mut leaf = MaybeUninit::<Self>::uninit();
            LeafNode::init(leaf.as_mut_ptr());
            Box::new(leaf.assume_init())
        }
    }
}

impl<T> InternalNode<T> {
    unsafe fn init(this: *mut Self) {
        LeafNode::init(ptr::addr_of_mut!((*this).data));
        ptr::addr_of_mut!((*this).treelen).write(0);
    }
    /// # Safety
    /// An invariant of internal nodes is that they have at least one
    /// initialized and valid edge. This function does not set up such
    /// an edge.
    pub unsafe fn new() -> Box<Self> {
        unsafe {
            let mut internal = MaybeUninit::<Self>::uninit();
            InternalNode::init(internal.as_mut_ptr());
            Box::new(internal.assume_init())
        }
    }
}

impl<BorrowType, T, NodeType> NodeRef<BorrowType, T, NodeType> {
    unsafe fn cast<NewBorrowType, NewNodeType>(
        &self,
    ) -> NodeRef<NewBorrowType, T, NewNodeType> {
        NodeRef {
            node: self.node,
            height: self.height,
            _marker: PhantomData,
        }
    }
}

impl<T> NodeRef<marker::Owned, T, marker::Leaf> {
    pub fn new_leaf() -> Self { Self::from_new_leaf(LeafNode::new()) }
    fn from_new_leaf(leaf: Box<LeafNode<T>>) -> Self {
        NodeRef {
            node: NonNull::from(Box::leak(leaf)),
            height: 0,
            _marker: PhantomData,
        }
    }
}

impl<T> NodeRef<marker::Owned, T, marker::Internal> {
    /// # Safety
    /// `left.height == right.height`
    unsafe fn new_single_internal<NodeType>(
        elt: T,
        left: NodeRef<marker::Mut<'_>, T, NodeType>,
        right: NodeRef<marker::Mut<'_>, T, NodeType>,
    ) -> Self {
        debug_assert_eq!(left.height, right.height);
        let height = left.height;
        let mut node = unsafe { InternalNode::new() };
        node.data.buf[0].write(elt);
        node.children[0].write(left.node);
        node.children[1].write(right.node);
        node.data.buflen = 1;
        let node = NonNull::from(Box::leak(node)).cast();
        let mut this =
            NodeRef { height: height + 1, node, _marker: PhantomData };
        this.borrow_mut().correct_parent_children_invariant();
        this
    }
    /// # Safety
    /// The caller has to guarantee the unemptiness.
    unsafe fn new_empty_internal(height: u8) -> Self {
        let node =
            NonNull::from(Box::leak(unsafe { InternalNode::<T>::new() }));
        NodeRef { height, node: node.cast(), _marker: PhantomData }
    }
}

impl<BorrowType, T> NodeRef<BorrowType, T, marker::LeafOrInternal> {
    fn from_node(node: NonNull<LeafNode<T>>, height: u8) -> Self {
        NodeRef { node, height, _marker: PhantomData }
    }
    fn force(&self) -> ForceResult<BorrowType, T> {
        if self.height == 0 {
            ForceResult::Leaf(unsafe { self.cast() })
        } else {
            ForceResult::Internal(unsafe { self.cast() })
        }
    }
}

impl<BorrowType: Traversable, T>
    NodeRef<BorrowType, T, marker::LeafOrInternal>
{
    fn first_child(&self) -> Option<Self> {
        self.force().internal().map(|internal| unsafe {
            let ptr = internal.as_internal_ptr();
            let node = (*ptr).children[0].assume_init();
            NodeRef::from_node(node, self.height - 1)
        })
    }
    fn last_child(&self) -> Option<Self> {
        self.force().internal().map(|internal| unsafe {
            let ptr = internal.as_internal_ptr();
            let node =
                (*ptr).children[(*ptr).data.buflen as usize].assume_init();
            NodeRef::from_node(node, self.height - 1)
        })
    }

    fn first_leaf(
        &self,
    ) -> Handle<NodeRef<BorrowType, T, marker::Leaf>, marker::Edge> {
        use ForceResult::*;
        match self.force() {
            Leaf(leaf) => Handle::new(leaf, 0),
            Internal(internal) => {
                let child = internal.child(0).unwrap();
                child.first_leaf()
            }
        }
    }
    fn last_leaf(
        &self,
    ) -> Handle<NodeRef<BorrowType, T, marker::Leaf>, marker::Edge> {
        use ForceResult::*;
        let init_len = self.buflen();
        match self.force() {
            Leaf(leaf) => Handle::new(leaf, init_len as _),
            Internal(internal) => {
                let child = internal.child(init_len).unwrap();
                child.last_leaf()
            }
        }
    }

    /// # Safety
    /// `idx <= self.treelen()` and the `.treelen` invariant is met.
    unsafe fn select_leaf(
        &self,
        mut idx: usize,
    ) -> Handle<NodeRef<BorrowType, T, marker::Leaf>, marker::Edge> {
        use ForceResult::*;

        debug_assert!(idx <= self.treelen());
        match self.force() {
            Leaf(leaf) => Handle::new(leaf, idx),
            Internal(internal) => {
                let init_len = self.buflen();
                for i in 0..=init_len {
                    let child = internal.child(i).unwrap();
                    if idx <= child.treelen() {
                        return child.select_leaf(idx);
                    } else {
                        idx -= child.treelen() + 1;
                    }
                }
                unreachable!()
            }
        }
    }

    /// # Safety
    /// `idx < self.treelen()` and the `.treelen` invariant is met.
    unsafe fn select_value(
        &self,
        mut idx: usize,
    ) -> Handle<NodeRef<BorrowType, T, marker::LeafOrInternal>, marker::Value>
    {
        use ForceResult::*;
        debug_assert!(idx < self.treelen());
        match self.force() {
            Leaf(leaf) => Handle::new(leaf, idx).forget_node_type(),
            Internal(internal) => {
                let init_len = self.buflen();
                for i in 0..=init_len {
                    let child = internal.child(i).unwrap();
                    if idx < child.treelen() {
                        return child.select_value(idx);
                    } else if idx == child.treelen() {
                        return Handle {
                            node: internal.forget_type(),
                            idx: i as _,
                            _marker: PhantomData,
                        };
                    } else {
                        idx -= child.treelen() + 1;
                    }
                }
                unreachable!()
            }
        }
    }

    /// # Safety
    /// The `.treelen` invariant is met.
    unsafe fn bisect_index<F>(&self, predicate: F) -> usize
    where
        F: Fn(&T) -> bool,
    {
        use ForceResult::*;
        let init_len = self.buflen() as usize;
        match self.force() {
            Leaf(_) => {
                let ptr = self.node.as_ptr();
                (0..init_len)
                    .find(|&i| {
                        !predicate(unsafe { (*ptr).buf[i].assume_init_ref() })
                    })
                    .unwrap_or(init_len)
            }
            Internal(internal) => {
                let mut rank = 0;
                let ptr = internal.as_internal_ptr();
                for i in 0..init_len {
                    let elt = unsafe { (*ptr).data.buf[i].assume_init_ref() };
                    let child = internal.child(i as _).unwrap();
                    if predicate(&elt) {
                        rank += child.treelen() + 1;
                    } else {
                        return rank + child.bisect_index(predicate);
                    }
                }
                rank + internal
                    .child(init_len as _)
                    .unwrap()
                    .bisect_index(predicate)
            }
        }
    }
}

impl<BorrowType, T> NodeRef<BorrowType, T, marker::Leaf> {
    /// # Safety
    /// `node` points to an actual leaf.
    unsafe fn from_leaf(node: NonNull<LeafNode<T>>) -> Self {
        NodeRef { node, height: 0, _marker: PhantomData }
    }
    fn forget_type(self) -> NodeRef<BorrowType, T, marker::LeafOrInternal> {
        unsafe { self.cast() }
    }
}

impl<BorrowType, T> NodeRef<BorrowType, T, marker::Internal> {
    fn as_internal_ptr(&self) -> *mut InternalNode<T> {
        self.node.as_ptr() as *mut InternalNode<T>
    }
    /// # Safety
    /// `height > 0`
    unsafe fn from_internal(
        node: NonNull<InternalNode<T>>,
        height: u8,
    ) -> Self {
        debug_assert!(height > 0);
        NodeRef { node: node.cast(), height, _marker: PhantomData }
    }
    fn forget_type(self) -> NodeRef<BorrowType, T, marker::LeafOrInternal> {
        unsafe { self.cast() }
    }
}

impl<T, NodeType> NodeRef<marker::Owned, T, NodeType> {
    fn borrow(&self) -> NodeRef<marker::Immut<'_>, T, NodeType> {
        unsafe { self.cast() }
    }
    fn borrow_mut(&mut self) -> NodeRef<marker::Mut<'_>, T, NodeType> {
        unsafe { self.cast() }
    }
    fn borrow_valmut(&mut self) -> NodeRef<marker::ValMut<'_>, T, NodeType> {
        unsafe { self.cast() }
    }
    fn take(self) -> NodeRef<marker::Dying, T, NodeType> {
        unsafe { self.cast() }
    }
}

impl<'a, T, NodeType> NodeRef<marker::Mut<'a>, T, NodeType> {
    fn reborrow_mut(&mut self) -> NodeRef<marker::Mut<'a>, T, NodeType> {
        unsafe { self.cast() }
    }
    /// # Safety
    /// `self` has no parent.
    unsafe fn promote(&mut self) -> NodeRef<marker::Owned, T, NodeType> {
        debug_assert!(unsafe { (*self.node.as_ptr()).parent }.is_none());
        unsafe { self.cast() }
    }
}

impl<T> DyingNodeRef<T> {
    fn iter(self) -> IntoIterImpl<T> {
        let left = self.first_leaf().forget_node_type();
        let right = self.last_leaf().forget_node_type();
        IntoIterImpl::new(left, right)
    }
}

impl<'a, T> NodeRef<marker::Immut<'a>, T, marker::LeafOrInternal> {
    fn iter(&self) -> IterImpl<'a, T> {
        let left = self.first_leaf().forget_node_type();
        let right = self.last_leaf().forget_node_type();
        IterImpl::new(left, right)
    }
}

impl<'a, T: 'a> NodeRef<marker::ValMut<'a>, T, marker::LeafOrInternal> {
    fn iter(&mut self) -> IterMutImpl<'a, T> {
        let left = self.first_leaf().forget_node_type();
        let right = self.last_leaf().forget_node_type();
        IterMutImpl::new(left, right)
    }
}

impl<BorrowType, T, NodeType> NodeRef<BorrowType, T, NodeType> {
    fn buflen(&self) -> u8 { unsafe { (*self.node.as_ptr()).buflen } }
    fn is_underfull(&self) -> bool { usize::from(self.buflen()) < MIN_BUFLEN }
    fn treelen(&self) -> usize {
        if self.height > 0 {
            unsafe { (*self.node.cast::<InternalNode<T>>().as_ptr()).treelen }
        } else {
            self.buflen() as _
        }
    }
}

impl<BorrowType: Traversable, T> NodeRef<BorrowType, T, marker::Internal> {
    fn children_ref(&self) -> &[NonNull<LeafNode<T>>] {
        let init_len = self.buflen() as usize;
        unsafe {
            &*(&(*self.as_internal_ptr()).children[..=init_len]
                as *const [MaybeUninit<_>]
                as *const [NonNull<LeafNode<T>>])
        }
    }
    fn child(
        &self,
        i: u8,
    ) -> Option<NodeRef<BorrowType, T, marker::LeafOrInternal>> {
        let init_len = self.buflen();
        let ptr = self.as_internal_ptr();
        let node = (i <= init_len)
            .then(|| unsafe { (*ptr).children[i as usize].assume_init() })?;
        let height = self.height;
        Some(NodeRef::from_node(node, height - 1))
    }
    fn neighbors(
        &self,
    ) -> [Option<NodeRef<BorrowType, T, marker::Internal>>; 2] {
        if let Some(Handle { node: parent, idx, .. }) = self.parent() {
            let height = self.height;
            let parent_ptr = parent.as_internal_ptr();
            unsafe {
                let len = (*parent_ptr).data.buflen as usize;
                let left = (idx > 0)
                    .then(|| (*parent_ptr).children[idx - 1].assume_init());
                let right = (idx < len)
                    .then(|| (*parent_ptr).children[idx + 1].assume_init());
                [left, right].map(|o| {
                    o.map(|node| NodeRef::from_internal(node.cast(), height))
                })
            }
        } else {
            [None, None]
        }
    }
}

impl<BorrowType: Traversable, T> NodeRef<BorrowType, T, marker::Leaf> {
    fn neighbors(&self) -> [Option<NodeRef<BorrowType, T, marker::Leaf>>; 2] {
        if let Some(Handle { node: parent, idx, .. }) = self.parent() {
            let parent_ptr = parent.as_internal_ptr();
            unsafe {
                let len = (*parent_ptr).data.buflen as usize;
                let left = (idx > 0)
                    .then(|| (*parent_ptr).children[idx - 1].assume_init());
                let right = (idx < len)
                    .then(|| (*parent_ptr).children[idx + 1].assume_init());
                [left, right].map(|o| o.map(|node| NodeRef::from_leaf(node)))
            }
        } else {
            [None, None]
        }
    }
}

impl<BorrowType: Traversable, T, NodeType> NodeRef<BorrowType, T, NodeType> {
    fn parent(
        &self,
    ) -> Option<Handle<NodeRef<BorrowType, T, marker::Internal>, marker::Edge>>
    {
        let height = self.height;
        unsafe { (*self.node.as_ptr()).parent }.map(|(parent, idx)| unsafe {
            Handle {
                node: NodeRef::from_internal(parent, height + 1),
                idx: idx as _,
                _marker: PhantomData,
            }
        })
    }
    fn root(&self) -> NodeRef<BorrowType, T, marker::LeafOrInternal> {
        let mut cur = match self.parent() {
            Some(o) => o.node,
            None => return unsafe { self.cast() },
        };
        while let Some(Handle { node, .. }) = cur.parent() {
            cur = node;
        }
        cur.forget_type()
    }
}

impl<'a, T> NodeRef<marker::Mut<'a>, T, marker::Internal> {
    fn correct_parent_children_invariant(&mut self) {
        let init_len = self.buflen() as usize;
        let children_ref = self.children_ref();
        let ptr = self.node.cast();
        for i in 0..=init_len {
            let child = children_ref[i];
            unsafe { (*child.as_ptr()).parent = Some((ptr, i as _)) }
        }
        self.correct_treelen_invarant();
    }

    fn correct_treelen_invarant(&mut self) {
        let init_len = self.buflen() as usize;
        let mut treelen = init_len;
        let children_ref = self.children_ref();
        for i in 0..=init_len {
            let child = children_ref[i];
            let child_ref = ImmutNodeRef::from_node(child, self.height - 1);
            treelen += child_ref.treelen();
        }
        unsafe { (*self.as_internal_ptr()).treelen = treelen }
    }

    fn correct_treelen_invarant_to_root(&mut self) {
        self.correct_treelen_invarant();
        let mut cur = self.parent();
        while let Some(mut handle) = cur {
            handle.node.correct_treelen_invarant();
            cur = handle.node.parent();
        }
    }
}

impl<'a, T> NodeRef<marker::Mut<'a>, T, marker::LeafOrInternal> {
    fn correct_treelen_invariant_subtree(&mut self) {
        if let Some(internal) = self.force().internal() {
            let init_len = self.buflen();
            let mut treelen = init_len as usize;
            for i in 0..=init_len {
                let mut child = internal.child(i as _).unwrap();
                child.correct_treelen_invariant_subtree();
                treelen += child.treelen();
            }
            unsafe { (*internal.as_internal_ptr()).treelen = treelen }
        }
    }
}

impl<T> OwnedNodeRef<T> {
    fn adjoin(mut self, mid: T, mut other: Self) -> Self {
        let mut left = self.borrow_mut();
        let mut right = other.borrow_mut();
        let mut root = if left.height != right.height {
            let Handle { node: mut parent, idx, .. } =
                if left.height < right.height {
                    while left.height < right.height {
                        // SAFETY: 0 <= left.height < right.height
                        right = right.first_child().unwrap();
                    }
                    right.parent().unwrap()
                } else {
                    while left.height > right.height {
                        left = left.last_child().unwrap();
                    }
                    left.parent().unwrap()
                };
            unsafe {
                let node = NodeRef::new_single_internal(mid, left, right);
                let _ = parent.insert(idx, node);
                parent.correct_treelen_invarant_to_root();
                if let Some(mut node) = (0..=parent.buflen())
                    .map(|i| parent.child(i).unwrap())
                    .find(|node| node.is_underfull())
                {
                    node.underflow().unwrap_or_else(|| node.root().promote())
                } else {
                    parent.root().promote()
                }
            }
        } else {
            if ((left.buflen() + right.buflen() + 1) as usize) <= CAPACITY {
                // Note that `left` and `right` are roots, so it is not
                // necessarily true that |left| == |right| == B - 1.
                // Anyway, we do not have to allocate a new node. We
                // merge them into one of them and deallocate the other.
                left.append(mid, right);
                unsafe { left.promote() }
            } else {
                // At most one of them may be underfull, but we can
                // resolve it by rotate properly.
                let mut node =
                    unsafe { NodeRef::new_single_internal(mid, left, right) };
                let node_mut = node.borrow_mut();
                let mut left = node_mut.child(0).unwrap();
                let mut right = node_mut.child(1).unwrap();
                left.rotate(&mut right);
                node.forget_type()
            }
        };
        root.borrow_mut()
            .force()
            .internal()
            .map(|mut internal| internal.correct_treelen_invarant());
        root
    }
    /// # Safety
    /// `i <= self.treelen()`
    unsafe fn split_off(mut self, i: usize) -> [Option<Self>; 2] {
        debug_assert!(i <= self.treelen());
        self.borrow_mut().select_leaf(i).split()
    }
    /// # Safety
    /// `self.treelen() == 1`
    unsafe fn take_single(self) -> T {
        use ForceResult::*;
        debug_assert_eq!(self.treelen(), 1);
        let res = unsafe { (*self.node.as_ptr()).buf[0].assume_init_read() };
        match self.force() {
            Leaf(leaf) => unsafe { drop(Box::from_raw(leaf.node.as_ptr())) },
            Internal(internal) => unsafe {
                drop(Box::from_raw(internal.as_internal_ptr()))
            },
        }
        res
    }
    fn drop_subtree(&mut self) {
        let dying: DyingNodeRef<_> = unsafe { self.cast() };
        dying.drop_subtree(false);
    }
}

impl<T> DyingNodeRef<T> {
    fn drop_subtree(self, elt_dropped: bool) {
        let init_len = self.buflen() as usize;
        let ptr = self.node.as_ptr();
        if !elt_dropped {
            unsafe {
                for e in &mut (*ptr).buf[..init_len] {
                    e.assume_init_drop()
                }
            }
        }
        match self.force() {
            ForceResult::Leaf(leaf) => unsafe {
                drop(Box::from_raw(leaf.node.as_ptr()));
            },
            ForceResult::Internal(internal) => {
                let ptr = internal.as_internal_ptr();
                unsafe {
                    for i in 0..=init_len {
                        let child = internal.child(i as _).unwrap();
                        child.drop_subtree(elt_dropped);
                    }
                    drop(Box::from_raw(ptr));
                }
            }
        }
    }
}

impl<BorrowType, T> ForceResult<BorrowType, T> {
    #[allow(dead_code)]
    fn leaf(self) -> Option<NodeRef<BorrowType, T, marker::Leaf>> {
        if let Self::Leaf(leaf) = self { Some(leaf) } else { None }
    }
    fn internal(self) -> Option<NodeRef<BorrowType, T, marker::Internal>> {
        if let Self::Internal(internal) = self { Some(internal) } else { None }
    }
}

impl<'a, T> MutNodeRef<'a, T> {
    fn underflow(
        &mut self,
    ) -> Option<NodeRef<marker::Owned, T, marker::LeafOrInternal>> {
        use ForceResult::*;
        unsafe {
            match self.force() {
                Leaf(mut leaf) => leaf.underflow(),
                Internal(mut internal) => internal.underflow(),
            }
        }
    }
    fn rotate(&mut self, other: &mut Self) {
        use ForceResult::*;

        match (self.force(), other.force()) {
            (Leaf(mut left), Leaf(mut right)) => left.rotate(&mut right),
            (Internal(mut left), Internal(mut right)) => {
                left.rotate(&mut right);
            }
            _ => unreachable!(),
        }
    }
    fn append(&mut self, mid: T, other: Self) {
        use ForceResult::*;

        match (self.force(), other.force()) {
            (Leaf(mut left), Leaf(right)) => left.append(mid, right),
            (Internal(mut left), Internal(right)) => left.append(mid, right),
            _ => unreachable!(),
        }
    }

    fn push_back(
        &mut self,
        elt: T,
    ) -> Option<NodeRef<marker::Owned, T, marker::Internal>> {
        unsafe {
            let Handle { mut node, idx, .. } = self.last_leaf();
            let root = node.insert(idx, elt);
            node.parent()
                .map(|mut o| o.node.correct_treelen_invarant_to_root());
            root
        }
    }
    fn push_front(
        &mut self,
        elt: T,
    ) -> Option<NodeRef<marker::Owned, T, marker::Internal>> {
        unsafe {
            let Handle { mut node, idx, .. } = self.first_leaf();
            let root = node.insert(idx, elt);
            node.parent()
                .map(|mut o| o.node.correct_treelen_invarant_to_root());
            root
        }
    }
}

impl<'a, T> MutLeafNodeRef<'a, T> {
    /// # Safety
    /// `i <= buflen`
    pub unsafe fn insert(
        &mut self,
        i: usize,
        elt: T,
    ) -> Option<NodeRef<marker::Owned, T, marker::Internal>> {
        // We do not maintain the invariant of `.treelen` to keep the
        // amortized complexity constant. This is preferable for
        // consecutive insertions like `.collect()` or `.extend()`.
        debug_assert!(i <= usize::from(self.buflen()));

        if (self.buflen() as usize) < CAPACITY {
            self.insert_fit(i, elt);
            None
        } else {
            let (orphan, new_parent) = self.purge_and_insert(i, elt);
            if let Some(Handle { node: mut parent, idx: par_i, .. }) =
                new_parent
            {
                parent.insert(par_i, orphan)
            } else {
                Some(orphan)
            }
        }
    }

    fn purge_and_insert(
        &mut self,
        i: usize,
        elt: T,
    ) -> (
        NodeRef<marker::Owned, T, marker::Internal>,
        Option<
            Handle<NodeRef<marker::Mut<'_>, T, marker::Internal>, marker::Edge>,
        >,
    ) {
        let mut orphan = NodeRef::new_leaf();
        let parent = self.parent();
        unsafe {
            let (left, right, leftlen, rightlen) = if i <= B {
                (self.reborrow_mut(), orphan.borrow_mut(), CAPACITY, 0)
            } else {
                (orphan.borrow_mut(), self.reborrow_mut(), 0, CAPACITY)
            };
            let left_ptr = left.node.as_ptr();
            let right_ptr = right.node.as_ptr();
            let left_buf = &mut (*left_ptr).buf;
            let right_buf = &mut (*right_ptr).buf;
            array_rotate_2(left_buf, right_buf, leftlen, rightlen, B);
            let par_elt = left_buf[B - 1].assume_init_read();
            if i <= B {
                array_insert(left_buf, i, B - 1, elt);
                (*left_ptr).buflen = B as _;
                (*right_ptr).buflen = (B - 1) as _;
            } else {
                array_insert(right_buf, i - B, B - 1, elt);
                (*left_ptr).buflen = (B - 1) as _;
                (*right_ptr).buflen = B as _;
            }
            (NodeRef::new_single_internal(par_elt, left, right), parent)
        }
    }

    fn insert_fit(&mut self, i: usize, elt: T) {
        let ptr = self.node.as_ptr();
        unsafe {
            array_insert(&mut (*ptr).buf, i, (*ptr).buflen as _, elt);
            (*ptr).buflen += 1;
        }
    }

    pub unsafe fn underflow(
        &mut self,
    ) -> Option<NodeRef<marker::Owned, T, marker::LeafOrInternal>> {
        // If it does not have a parent, then it is the root and nothing
        // has to be done.
        let Handle { node: mut parent, .. } = self.parent()?;
        let len = self.buflen();
        match self.neighbors() {
            [Some(mut left), _]
                if usize::from(left.buflen() + len) >= 2 * MIN_BUFLEN =>
            {
                left.rotate(self);
                None
            }
            [_, Some(mut right)]
                if usize::from(len + right.buflen()) >= 2 * MIN_BUFLEN =>
            {
                self.rotate(&mut right);
                None
            }
            [Some(left), _] => {
                // |left| + |self| + 1 < 2 * B - 1
                // Take an element from the parent, and check it.
                self.merge(left, false);
                if parent.buflen() == 0 {
                    // Now `self` is the new root, so deallocate it and
                    // promote `self`.
                    unsafe {
                        let _ = (*self.node.as_ptr()).parent.take();
                        drop(Box::from_raw(parent.as_internal_ptr()));
                        Some(self.promote().forget_type())
                    }
                } else {
                    parent.underflow()
                }
            }
            [_, Some(right)] => {
                self.merge(right, true);
                if parent.buflen() == 0 {
                    unsafe {
                        let _ = (*self.node.as_ptr()).parent.take();
                        drop(Box::from_raw(parent.as_internal_ptr()));
                        Some(self.promote().forget_type())
                    }
                } else {
                    parent.underflow()
                }
            }
            [None, None] => unreachable!(),
        }
    }

    fn rotate(&mut self, other: &mut Self) {
        if !self.is_underfull() && !other.is_underfull() {
            return;
        }
        let Handle { node: parent, idx, .. } = self.parent().unwrap();
        Self::rotate_leaf(
            self.node.as_ptr(),
            (parent.as_internal_ptr(), idx),
            other.node.as_ptr(),
        );
    }
    fn merge(&mut self, other: Self, self_left: bool) {
        let Handle { node: mut parent, idx, .. } = self.parent().unwrap();
        if self_left {
            Self::merge_leaf(
                self.node.as_ptr(),
                (parent.as_internal_ptr(), idx),
                other.node.as_ptr(),
            );
        } else {
            Self::merge_leaf(
                other.node.as_ptr(),
                (parent.as_internal_ptr(), idx - 1),
                self.node.as_ptr(),
            );
            unsafe {
                ptr::swap(self.node.as_ptr(), other.node.as_ptr());
                (*parent.as_internal_ptr()).children[idx - 1].write(self.node);
            }
        }
        parent.correct_parent_children_invariant();
        unsafe { drop(Box::from_raw(other.node.as_ptr())) }
    }
    fn append(&mut self, elt: T, other: Self) {
        Self::append_leaf(self.node.as_ptr(), elt, other.node.as_ptr());
        unsafe { drop(Box::from_raw(other.node.as_ptr())) }
    }

    fn rotate_leaf(
        left_ptr: *mut LeafNode<T>,
        (parent_ptr, idx): (*mut InternalNode<T>, usize),
        right_ptr: *mut LeafNode<T>,
    ) {
        // left: [A, B, C, D, E], parent: [.., F, ..], right: [G, H]
        // left: [A, B, C], parent: [.., D, ..], right: [E, F, G, H]
        unsafe {
            let left = &mut (*left_ptr).buf;
            let leftlen = (*left_ptr).buflen as usize;
            let mid = &mut (*parent_ptr).data.buf[idx];
            let right = &mut (*right_ptr).buf;
            let rightlen = (*right_ptr).buflen as usize;
            debug_assert!(leftlen + rightlen >= 2 * MIN_BUFLEN);
            let rightlen_new =
                array_rotate_3(left, mid, right, leftlen, rightlen, MIN_BUFLEN);
            (*left_ptr).buflen = MIN_BUFLEN as _;
            (*right_ptr).buflen = rightlen_new as _;
        }
    }

    fn merge_leaf(
        left_ptr: *mut LeafNode<T>,
        (parent_ptr, idx): (*mut InternalNode<T>, usize),
        right_ptr: *mut LeafNode<T>,
    ) {
        // left: [A, B, C, D, E], parent: [.., F, ..], right: [G, H]
        // left: [A, B, C, D, E, F, G, H], parent: [.., ..]
        unsafe {
            let left = &mut (*left_ptr).buf;
            let leftlen = (*left_ptr).buflen as usize;
            let mid = {
                let buf = &mut (*parent_ptr).data.buf;
                let len = (*parent_ptr).data.buflen as usize;
                let _ =
                    array_remove(&mut (*parent_ptr).children, idx + 1, len + 1);
                array_remove(buf, idx, len)
            };
            left[leftlen].write(mid);
            let leftlen = leftlen + 1;

            let right = &(*right_ptr).buf;
            let rightlen = (*right_ptr).buflen as usize;
            array_splice(left, leftlen, leftlen, right, rightlen);

            (*left_ptr).buflen += 1 + rightlen as u8;
            (*parent_ptr).data.buflen -= 1;
        }
    }

    fn append_leaf(
        left_ptr: *mut LeafNode<T>,
        elt: T,
        right_ptr: *mut LeafNode<T>,
    ) {
        // left: [A, B, C, D], right: [E, F, G]
        // left: [A, B, C, D, elt, E, F, G]
        unsafe {
            debug_assert!((*left_ptr).parent.is_none());
            debug_assert!((*right_ptr).parent.is_none());
            let left = &mut (*left_ptr).buf;
            let leftlen = (*left_ptr).buflen as usize;
            let right = &(*right_ptr).buf;
            let rightlen = (*right_ptr).buflen as usize;
            let newlen = leftlen + rightlen + 1;
            debug_assert!(newlen <= CAPACITY);
            left[leftlen].write(elt);
            let leftlen = leftlen + 1;

            array_splice(left, leftlen, leftlen, right, rightlen);
            (*left_ptr).buflen = newlen as _;
        }
    }
}

impl<'a, T> MutInternalNodeRef<'a, T> {
    /// # Safety
    /// `i <= buflen`
    unsafe fn insert(
        &mut self,
        i: usize,
        orphan: NodeRef<marker::Owned, T, marker::Internal>,
    ) -> Option<NodeRef<marker::Owned, T, marker::Internal>> {
        debug_assert!(i <= usize::from(self.buflen()));

        if (self.buflen() as usize) < CAPACITY {
            self.insert_fit(i, orphan);
            None
        } else {
            let (orphan, new_parent) = self.purge_and_insert(i, orphan);

            if let Some(Handle { node: mut parent, idx: par_i, .. }) =
                new_parent
            {
                parent.insert(par_i, orphan)
            } else {
                Some(orphan)
            }
        }
    }

    fn purge_and_insert(
        &mut self,
        i: usize,
        node: NodeRef<marker::Owned, T, marker::Internal>,
    ) -> (
        NodeRef<marker::Owned, T, marker::Internal>,
        Option<
            Handle<NodeRef<marker::Mut<'_>, T, marker::Internal>, marker::Edge>,
        >,
    ) {
        let parent = self.parent();
        let node_ptr = node.as_internal_ptr();
        unsafe {
            let mut orphan = NodeRef::new_empty_internal(self.height);
            let elt = (*node_ptr).data.buf[0].assume_init_read();
            let left_child = (*node_ptr).children[0].assume_init_read();
            let right_child = (*node_ptr).children[1].assume_init_read();
            let (mut left, mut right, par_elt) = if i <= B {
                let left_ptr = self.as_internal_ptr();
                let right_ptr = orphan.as_internal_ptr();
                let left_buf = &mut (*left_ptr).data.buf;
                let right_buf = &mut (*right_ptr).data.buf;
                array_rotate_2(left_buf, right_buf, CAPACITY, 0, B);
                let left_children = &mut (*left_ptr).children;
                let right_children = &mut (*right_ptr).children;
                array_rotate_2(left_children, right_children, 2 * B, 0, B);
                let par_elt = left_buf[B - 1].assume_init_read();
                array_insert(left_buf, i, B - 1, elt);
                left_children[i].write(left_child);
                array_insert(left_children, i + 1, B, right_child);
                (*left_ptr).data.buflen = B as _;
                (*right_ptr).data.buflen = (B - 1) as _;
                (self.reborrow_mut(), orphan.borrow_mut(), par_elt)
            } else {
                let left_ptr = orphan.as_internal_ptr();
                let right_ptr = self.as_internal_ptr();
                let left_buf = &mut (*left_ptr).data.buf;
                let right_buf = &mut (*right_ptr).data.buf;
                array_rotate_2(left_buf, right_buf, 0, CAPACITY, B);
                let left_children = &mut (*left_ptr).children;
                let right_children = &mut (*right_ptr).children;
                array_rotate_2(left_children, right_children, 0, 2 * B, B);
                let par_elt = left_buf[B - 1].assume_init_read();
                array_insert(right_buf, i - B, B - 1, elt);
                right_children[i - B].write(left_child);
                array_insert(right_children, i + 1 - B, B, right_child);
                (*left_ptr).data.buflen = (B - 1) as _;
                (*right_ptr).data.buflen = B as _;
                (orphan.borrow_mut(), self.reborrow_mut(), par_elt)
            };
            left.correct_parent_children_invariant();
            right.correct_parent_children_invariant();

            drop(Box::from_raw(node.as_internal_ptr()));
            (NodeRef::new_single_internal(par_elt, left, right), parent)
        }
    }

    fn insert_fit(
        &mut self,
        i: usize,
        orphan: NodeRef<marker::Owned, T, marker::Internal>,
    ) {
        let orphan_ptr = orphan.as_internal_ptr();
        let this = self.as_internal_ptr();
        // As `orphan` is purged from the subtree, we do not have to
        // update the `.treelen`. Note that adding one to leaf-to-root
        // is the job for the caller.
        unsafe {
            let buflen = (*this).data.buflen as usize;
            let elt = (*orphan_ptr).data.buf[0].assume_init_read();
            let left = (*orphan_ptr).children[0].assume_init_read();
            let right = (*orphan_ptr).children[1].assume_init_read();
            array_insert(&mut (*this).data.buf, i, buflen, elt);
            (*this).children[i].write(left);
            array_insert(&mut (*this).children, i + 1, buflen + 1, right);
            (*this).data.buflen += 1;
            drop(Box::from_raw(orphan_ptr));
        }
        self.correct_parent_children_invariant();
    }

    unsafe fn underflow(
        &mut self,
    ) -> Option<NodeRef<marker::Owned, T, marker::LeafOrInternal>> {
        let Handle { node: mut parent, .. } = self.parent()?;
        let len = self.buflen();
        match self.neighbors() {
            [Some(mut left), _]
                if usize::from(left.buflen() + len) >= 2 * MIN_BUFLEN =>
            {
                left.rotate(self);
                None
            }
            [_, Some(mut right)]
                if usize::from(len + right.buflen()) >= 2 * MIN_BUFLEN =>
            {
                self.rotate(&mut right);
                None
            }
            [Some(left), _] => {
                self.merge(left, false);
                if parent.buflen() == 0 {
                    unsafe {
                        let _ = (*self.node.as_ptr()).parent.take();
                        drop(Box::from_raw(parent.as_internal_ptr()));
                        Some(self.promote().forget_type())
                    }
                } else {
                    parent.underflow()
                }
            }
            [_, Some(right)] => {
                self.merge(right, true);
                if parent.buflen() == 0 {
                    unsafe {
                        let _ = (*self.node.as_ptr()).parent.take();
                        drop(Box::from_raw(parent.as_internal_ptr()));
                        Some(self.promote().forget_type())
                    }
                } else {
                    parent.underflow()
                }
            }
            [None, None] => unreachable!(),
        }
    }

    fn rotate(&mut self, other: &mut Self) {
        if !self.is_underfull() && !other.is_underfull() {
            return;
        }
        let Handle { node: parent, idx, .. } = self.parent().unwrap();
        Self::rotate_internal(
            self.as_internal_ptr(),
            (parent.as_internal_ptr(), idx),
            other.as_internal_ptr(),
        );
        self.correct_parent_children_invariant();
        other.correct_parent_children_invariant();
    }
    fn merge(&mut self, other: Self, self_left: bool) {
        let Handle { node: mut parent, idx, .. } = self.parent().unwrap();
        if self_left {
            Self::merge_internal(
                self.as_internal_ptr(),
                (parent.as_internal_ptr(), idx),
                other.as_internal_ptr(),
            );
        } else {
            Self::merge_internal(
                other.as_internal_ptr(),
                (parent.as_internal_ptr(), idx - 1),
                self.as_internal_ptr(),
            );
            unsafe {
                ptr::swap(self.as_internal_ptr(), other.as_internal_ptr());
                (*parent.as_internal_ptr()).children[idx - 1].write(self.node);
            }
        }
        self.correct_parent_children_invariant();
        parent.correct_parent_children_invariant();
        unsafe { drop(Box::from_raw(other.as_internal_ptr())) }
    }
    fn append(&mut self, elt: T, other: Self) {
        Self::append_internal(
            self.as_internal_ptr(),
            elt,
            other.as_internal_ptr(),
        );
        self.correct_parent_children_invariant();
        unsafe { drop(Box::from_raw(other.as_internal_ptr())) }
    }

    fn rotate_internal(
        left_ptr: *mut InternalNode<T>,
        (parent_ptr, idx): (*mut InternalNode<T>, usize),
        right_ptr: *mut InternalNode<T>,
    ) {
        unsafe {
            let left = &mut (*left_ptr).children;
            let leftlen = (*left_ptr).data.buflen as usize + 1;
            let right = &mut (*right_ptr).children;
            let rightlen = (*right_ptr).data.buflen as usize + 1;
            array_rotate_2(left, right, leftlen, rightlen, MIN_BUFLEN + 1);
        }
        MutLeafNodeRef::<T>::rotate_leaf(
            left_ptr.cast(),
            (parent_ptr, idx),
            right_ptr.cast(),
        );
    }

    fn merge_internal(
        left_ptr: *mut InternalNode<T>,
        (parent_ptr, idx): (*mut InternalNode<T>, usize),
        right_ptr: *mut InternalNode<T>,
    ) {
        unsafe {
            let left = &mut (*left_ptr).children;
            let leftlen = (*left_ptr).data.buflen as usize + 1;
            let right = &(*right_ptr).children;
            let rightlen = (*right_ptr).data.buflen as usize + 1;
            array_splice(left, leftlen, leftlen, right, rightlen);
        }
        MutLeafNodeRef::<T>::merge_leaf(
            left_ptr.cast(),
            (parent_ptr, idx),
            right_ptr.cast(),
        );
    }

    fn append_internal(
        left_ptr: *mut InternalNode<T>,
        mid: T,
        right_ptr: *mut InternalNode<T>,
    ) {
        unsafe {
            let left = &mut (*left_ptr).children;
            let leftlen = (*left_ptr).data.buflen as usize + 1;
            let right = &(*right_ptr).children;
            let rightlen = (*right_ptr).data.buflen as usize + 1;
            array_splice(left, leftlen, leftlen, right, rightlen);
        }
        NodeRef::append_leaf(left_ptr.cast(), mid, right_ptr.cast());
    }
}

struct Handle<Node, Type> {
    node: Node,
    idx: usize,
    _marker: PhantomData<Type>,
}

impl<Node: Copy, Type> Copy for Handle<Node, Type> {}
impl<Node: Copy, Type> Clone for Handle<Node, Type> {
    fn clone(&self) -> Self { *self }
}

impl<Node, Type> Handle<Node, Type> {
    fn new(node: Node, idx: usize) -> Self {
        Self { node, idx, _marker: PhantomData }
    }
}
impl<BorrowType, T, Type> Handle<NodeRef<BorrowType, T, marker::Leaf>, Type> {
    fn forget_node_type(
        self,
    ) -> Handle<NodeRef<BorrowType, T, marker::LeafOrInternal>, Type> {
        Handle {
            node: self.node.forget_type(),
            idx: self.idx,
            _marker: PhantomData,
        }
    }
}
impl<BorrowType, T, Type>
    Handle<NodeRef<BorrowType, T, marker::Internal>, Type>
{
    #[allow(dead_code)]
    fn forget_node_type(
        self,
    ) -> Handle<NodeRef<BorrowType, T, marker::LeafOrInternal>, Type> {
        Handle {
            node: self.node.forget_type(),
            idx: self.idx,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: 'a, NodeType>
    Handle<NodeRef<marker::Immut<'a>, T, NodeType>, marker::Value>
{
    unsafe fn get(&self) -> &'a T {
        let Self { node, idx, .. } = self;
        let ptr = node.node.as_ptr();
        debug_assert!(*idx < usize::from((*ptr).buflen));
        unsafe { (*ptr).buf[*idx].assume_init_ref() }
    }
}
impl<'a, T: 'a, NodeType>
    Handle<NodeRef<marker::ValMut<'a>, T, NodeType>, marker::Value>
{
    unsafe fn get_mut(&self) -> &'a mut T {
        let Self { node, idx, .. } = self;
        let ptr = node.node.as_ptr();
        debug_assert!(*idx < usize::from((*ptr).buflen));
        unsafe { (*ptr).buf[*idx].assume_init_mut() }
    }
}

struct LeafSplit<'a, T> {
    left: Option<OwnedNodeRef<T>>,
    right: Option<OwnedNodeRef<T>>,
    parent: Option<Handle<MutInternalNodeRef<'a, T>, marker::Edge>>,
}

impl<'a, T> Handle<MutLeafNodeRef<'a, T>, marker::Edge> {
    fn split_ascend(self) -> LeafSplit<'a, T> {
        // [A, B, C, D, E, F, G] => [A, B, C, D], [E, F, G]
        let Self { mut node, idx, .. } = self;
        let init_len = node.buflen() as usize;
        let parent = node.parent();
        let _ = unsafe { (*node.node.as_ptr()).parent.take() };
        let (left, right) = if idx == 0 {
            (None, Some(unsafe { node.promote() }))
        } else if idx == init_len {
            (Some(unsafe { node.promote() }), None)
        } else {
            let new = NodeRef::new_leaf();
            let left_ptr = node.node.as_ptr();
            let right_ptr = new.node.as_ptr();
            unsafe {
                let left = &mut (*left_ptr).buf;
                let right = &mut (*right_ptr).buf;
                let rightlen = array_rotate_2(left, right, init_len, 0, idx);
                (*left_ptr).buflen = idx as _;
                (*right_ptr).buflen = rightlen as _;
                (Some(node.promote()), Some(new))
            }
        };
        let left = left.map(|o| o.forget_type());
        let right = right.map(|o| o.forget_type());
        LeafSplit { left, right, parent }
    }
}

struct InternalSplit<'a, T> {
    left: Option<(OwnedNodeRef<T>, T)>,
    right: Option<(OwnedNodeRef<T>, T)>,
    parent: Option<Handle<MutInternalNodeRef<'a, T>, marker::Edge>>,
}

impl<'a, T> Handle<MutInternalNodeRef<'a, T>, marker::Edge> {
    fn split_ascend(self) -> InternalSplit<'a, T> {
        let Self { mut node, idx, .. } = self;
        let init_len = node.buflen() as usize;
        let parent = node.parent();
        let _ = unsafe { (*node.node.as_ptr()).parent.take() };
        let (left, right) = if idx == 0 {
            // (A), [B, C, D, E]
            unsafe {
                let ptr = node.as_internal_ptr();
                let buf = &mut (*ptr).data.buf;
                let elt = array_remove(buf, 0, init_len);
                let children = &mut (*ptr).children;
                let _ = array_remove(children, 0, init_len + 1);
                (*ptr).data.buflen -= 1;
                node.correct_parent_children_invariant();
                (None, Some((node.promote(), elt)))
            }
        } else if idx == init_len {
            unsafe {
                let ptr = node.as_internal_ptr();
                let buf = &mut (*ptr).data.buf;
                let elt = array_remove(buf, init_len - 1, init_len);
                let children = &mut (*ptr).children;
                let _ = array_remove(children, init_len, init_len + 1);
                (*ptr).data.buflen -= 1;
                node.correct_parent_children_invariant();
                (Some((node.promote(), elt)), None)
            }
        } else {
            // [A, B, C, D, |E], [F|, G, H]
            // [A, B, C, D], (E), (F), [G, H]
            unsafe {
                let mut new = NodeRef::new_empty_internal(node.height);
                let left_ptr = node.as_internal_ptr();
                let right_ptr = new.as_internal_ptr();
                let left_buf = &mut (*left_ptr).data.buf;
                let right_buf = &mut (*right_ptr).data.buf;
                let rightlen =
                    array_rotate_2(left_buf, right_buf, init_len, 0, idx + 1);
                let left_elt = left_buf[idx - 1].assume_init_read();
                let right_elt = left_buf[idx].assume_init_read();
                let left_children = &mut (*left_ptr).children;
                let right_children = &mut (*right_ptr).children;
                array_rotate_2(
                    left_children,
                    right_children,
                    init_len + 1,
                    0,
                    idx + 1,
                );
                (*left_ptr).data.buflen = (idx - 1) as _;
                (*right_ptr).data.buflen = rightlen as _;
                node.correct_parent_children_invariant();
                new.borrow_mut().correct_parent_children_invariant();
                let left = node.promote();
                let right = new;
                (Some((left, left_elt)), Some((right, right_elt)))
            }
        };
        // If an internal node has only one element and no child, then
        // we must return its child node instead. Deallocating the newly
        // allocated node is undesirable, but it reduces cases.
        let left = left.map(|(mut tree, elt)| {
            if tree.buflen() == 0 {
                unsafe {
                    let mut child = tree.borrow_mut().child(0).unwrap();
                    let _ = (*child.node.as_ptr()).parent.take();
                    let res = (child.promote(), elt);
                    drop(Box::from_raw(tree.as_internal_ptr()));
                    res
                }
            } else {
                (tree.forget_type(), elt)
            }
        });
        let right = right.map(|(mut tree, elt)| {
            if tree.buflen() == 0 {
                unsafe {
                    let mut child = tree.borrow_mut().child(0).unwrap();
                    let _ = (*child.node.as_ptr()).parent.take();
                    let res = (child.promote(), elt);
                    drop(Box::from_raw(tree.as_internal_ptr()));
                    res
                }
            } else {
                (tree.forget_type(), elt)
            }
        });

        InternalSplit { left, right, parent }
    }
}

impl<'a, T> Handle<MutLeafNodeRef<'a, T>, marker::Edge> {
    fn split(self) -> [Option<OwnedNodeRef<T>>; 2] {
        let [mut left_inner, mut right_inner]: [Option<T>; 2] = [None, None];
        let LeafSplit {
            left: mut left_tree,
            right: mut right_tree,
            parent: mut node,
        } = self.split_ascend();

        while let Some(cur) = node.take() {
            let InternalSplit { left, right, parent } = cur.split_ascend();
            match (left_tree, left) {
                (Some(left_lo), Some((left_hi, elt))) => {
                    left_tree = Some(left_hi.adjoin(elt, left_lo));
                }
                (None, Some((tree, elt))) => {
                    left_inner = Some(elt);
                    left_tree = Some(tree);
                }
                (o, None) => left_tree = o,
            }
            match (right_tree, right) {
                (Some(right_lo), Some((right_hi, elt))) => {
                    right_tree = Some(right_lo.adjoin(elt, right_hi));
                }
                (None, Some((tree, elt))) => {
                    right_inner = Some(elt);
                    right_tree = Some(tree);
                }
                (o, None) => right_tree = o,
            }
            node = parent;
        }
        if let (Some(old), Some(elt)) = (left_tree.as_mut(), left_inner) {
            if let Some(new) = old.borrow_mut().push_back(elt) {
                left_tree = Some(new.forget_type());
            }
        }
        if let (Some(old), Some(elt)) = (right_tree.as_mut(), right_inner) {
            if let Some(new) = old.borrow_mut().push_front(elt) {
                right_tree = Some(new.forget_type());
            }
        }
        [left_tree, right_tree]
    }
}

impl<BorrowType: Traversable, T>
    Handle<NodeRef<BorrowType, T, marker::LeafOrInternal>, marker::Edge>
{
    fn next(&mut self) {
        use ForceResult::*;
        let Self { node, idx, .. } = self;
        match node.force() {
            Leaf(_) => {
                let buflen = node.buflen() as usize;
                *idx += 1;
                if *idx < buflen {
                    return;
                }
                let mut parent = node.parent();
                while let Some(handle) = parent.as_ref() {
                    if handle.idx < usize::from(handle.node.buflen()) {
                        self.node.node = handle.node.node;
                        self.node.height = handle.node.height;
                        self.idx = handle.idx;
                        return;
                    }
                    parent = handle.node.parent();
                }
                // We have reached the last edge; nothing can be done.
            }
            Internal(internal) => {
                debug_assert!(*idx < usize::from(internal.buflen()));
                *idx += 1;
                let leaf = internal.child(*idx as _).unwrap().first_leaf();
                *self = leaf.forget_node_type();
            }
        }
    }
    fn next_back(&mut self) {
        use ForceResult::*;
        let Self { node, idx, .. } = self;
        match node.force() {
            Leaf(_) => {
                *idx -= 1;
                if *idx > 0 {
                    return;
                }
                let mut parent = node.parent();
                while let Some(handle) = parent.as_ref() {
                    if handle.idx > 0 {
                        self.node.node = handle.node.node;
                        self.node.height = handle.node.height;
                        self.idx = handle.idx;
                        return;
                    }
                    parent = handle.node.parent();
                }
                // We have reached the first edge; nothing can be done.
            }
            Internal(internal) => {
                debug_assert!(*idx > 0);
                *idx -= 1;
                let leaf = internal.child(*idx as _).unwrap().last_leaf();
                *self = leaf.forget_node_type();
            }
        }
    }
    fn eq(&self, other: &Self) -> bool {
        self.node.node == other.node.node && self.idx == other.idx
    }
}

impl<T>
    Handle<NodeRef<marker::Dying, T, marker::LeafOrInternal>, marker::Edge>
{
    /// # Safety
    /// The element must not be taken twice.
    unsafe fn take_next(&self) -> T {
        let Self { node, idx, .. } = self;
        let ptr = node.node.as_ptr();
        unsafe { (*ptr).buf[*idx].assume_init_read() }
    }
    unsafe fn take_prev(&self) -> T {
        let Self { node, idx, .. } = self;
        let ptr = node.node.as_ptr();
        unsafe { (*ptr).buf[idx - 1].assume_init_read() }
    }
}
impl<'a, T>
    Handle<NodeRef<marker::Immut<'a>, T, marker::LeafOrInternal>, marker::Edge>
{
    fn get_next(&self) -> &'a T {
        let Self { node, idx, .. } = self;
        let ptr = node.node.as_ptr();
        unsafe { (*ptr).buf[*idx].assume_init_ref() }
    }
    fn get_prev(&self) -> &'a T {
        let Self { node, idx, .. } = self;
        let ptr = node.node.as_ptr();
        unsafe { (*ptr).buf[idx - 1].assume_init_ref() }
    }
}
impl<'a, T>
    Handle<NodeRef<marker::ValMut<'a>, T, marker::LeafOrInternal>, marker::Edge>
{
    fn get_mut_next(&self) -> &'a mut T {
        let Self { node, idx, .. } = self;
        let ptr = node.node.as_ptr();
        unsafe { (*ptr).buf[*idx].assume_init_mut() }
    }
    fn get_mut_prev(&self) -> &'a mut T {
        let Self { node, idx, .. } = self;
        let ptr = node.node.as_ptr();
        unsafe { (*ptr).buf[idx - 1].assume_init_mut() }
    }
}

struct IterImpl<'a, T> {
    left: Handle<ImmutNodeRef<'a, T>, marker::Edge>,
    right: Handle<ImmutNodeRef<'a, T>, marker::Edge>,
}

impl<'a, T> IterImpl<'a, T> {
    fn new(
        left: Handle<ImmutNodeRef<'a, T>, marker::Edge>,
        right: Handle<ImmutNodeRef<'a, T>, marker::Edge>,
    ) -> Self {
        Self { left, right }
    }
}
impl<'a, T: 'a> Iterator for IterImpl<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        (!self.left.eq(&self.right)).then(|| {
            let res = self.left.get_next();
            self.left.next();
            res
        })
    }
}
impl<'a, T: 'a> DoubleEndedIterator for IterImpl<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        (!self.left.eq(&self.right)).then(|| {
            let res = self.right.get_prev();
            self.right.next_back();
            res
        })
    }
}

struct IterMutImpl<'a, T> {
    left: Handle<ValMutNodeRef<'a, T>, marker::Edge>,
    right: Handle<ValMutNodeRef<'a, T>, marker::Edge>,
}

impl<'a, T: 'a> IterMutImpl<'a, T> {
    fn new(
        left: Handle<ValMutNodeRef<'a, T>, marker::Edge>,
        right: Handle<ValMutNodeRef<'a, T>, marker::Edge>,
    ) -> Self {
        Self { left, right }
    }
}
impl<'a, T: 'a> Iterator for IterMutImpl<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        (!self.left.eq(&self.right)).then(|| {
            let res = self.left.get_mut_next();
            self.left.next();
            res
        })
    }
}
impl<'a, T: 'a> DoubleEndedIterator for IterMutImpl<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        (!self.left.eq(&self.right)).then(|| {
            let res = self.right.get_mut_prev();
            self.right.next_back();
            res
        })
    }
}

struct IntoIterImpl<T> {
    left: Handle<DyingNodeRef<T>, marker::Edge>,
    right: Handle<DyingNodeRef<T>, marker::Edge>,
}

impl<T> IntoIterImpl<T> {
    fn new(
        left: Handle<DyingNodeRef<T>, marker::Edge>,
        right: Handle<DyingNodeRef<T>, marker::Edge>,
    ) -> Self {
        Self { left, right }
    }
}
impl<T> Iterator for IntoIterImpl<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        (!self.left.eq(&self.right)).then(|| {
            let res = unsafe { self.left.take_next() };
            self.left.next();
            res
        })
    }
}
impl<T> DoubleEndedIterator for IntoIterImpl<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        (!self.left.eq(&self.right)).then(|| {
            let res = unsafe { self.right.take_prev() };
            self.right.next_back();
            res
        })
    }
}

pub struct Iter<'a, T>(Option<IterImpl<'a, T>>);
pub struct IterMut<'a, T>(Option<IterMutImpl<'a, T>>);
pub struct IntoIter<T>(Option<IntoIterImpl<T>>);

impl<'a, T> Iter<'a, T> {
    fn new(root: Option<&'a OwnedNodeRef<T>>) -> Self {
        Self(root.map(|root| root.borrow().iter()))
    }
}

impl<'a, T> IterMut<'a, T> {
    fn new(root: Option<&'a mut OwnedNodeRef<T>>) -> Self {
        Self(root.map(|root| root.borrow_valmut().iter()))
    }
}

impl<T> IntoIter<T> {
    fn new(root: Option<OwnedNodeRef<T>>) -> Self {
        Self(root.map(|root| root.take().iter()))
    }
}
impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        if let Some(mut iter) = self.0.take() {
            while let Some(_) = iter.next() {}
            iter.left.node.root().drop_subtree(true);
        }
    }
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|iter| iter.next())
    }
}
impl<'a, T: 'a> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|iter| iter.next_back())
    }
}
impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|iter| iter.next())
    }
}
impl<'a, T: 'a> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|iter| iter.next_back())
    }
}
impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|iter| iter.next())
    }
}
impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|iter| iter.next_back())
    }
}

impl<T> BTreeSeq<T> {
    pub fn new() -> Self { Self { root: None } }

    pub fn len(&self) -> usize {
        self.root.as_ref().map(|root| root.treelen()).unwrap_or(0)
    }
    pub fn is_empty(&self) -> bool { self.root.is_none() }

    pub fn push_back(&mut self, elt: T) {
        if let Some(root) = self.root.as_mut() {
            root.borrow_mut().push_back(elt);
        } else {
            *self = Self::singleton(elt);
        }
    }
    pub fn push_front(&mut self, elt: T) {
        if let Some(root) = self.root.as_mut() {
            root.borrow_mut().push_front(elt);
        } else {
            *self = Self::singleton(elt);
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        let len = self.len();
        self.root.take().map(|root| unsafe {
            let [left, right] = root.split_off(len - 1);
            self.root = left;
            right.unwrap().take_single()
        })
    }
    pub fn pop_front(&mut self) -> Option<T> {
        self.root.take().map(|root| unsafe {
            let [left, right] = root.split_off(1);
            self.root = right;
            left.unwrap().take_single()
        })
    }

    pub fn adjoin(&mut self, elt: T, mut other: Self) {
        self.root = match (self.root.take(), other.root.take()) {
            (Some(left), Some(right)) => Some(left.adjoin(elt, right)),
            (Some(mut left), None) => left
                .borrow_mut()
                .push_back(elt)
                .map(|o| o.forget_type())
                .or_else(|| Some(left)),
            (None, Some(mut right)) => right
                .borrow_mut()
                .push_front(elt)
                .map(|o| o.forget_type())
                .or_else(|| Some(right)),
            (None, None) => Self::singleton(elt).root.take(),
        };
    }
    pub fn append(&mut self, mut other: Self) {
        if let Some(elt) = other.pop_front() {
            self.adjoin(elt, other)
        }
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        #[cold]
        fn assert_failed(at: usize, len: usize) -> ! {
            panic!("`at` split index (is {at}) should be <= len (is {len})");
        }
        if at > self.len() {
            assert_failed(at, self.len());
        }

        if at == 0 {
            Self { root: self.root.take() }
        } else if at == self.len() {
            Self::new()
        } else {
            let [left, right] =
                unsafe { self.root.take().unwrap().split_off(at) };
            self.root = left;
            Self { root: right }
        }
    }

    fn singleton(elt: T) -> Self {
        let mut root = NodeRef::new_leaf();
        unsafe {
            root.borrow_mut().insert(0, elt);
            Self { root: Some(root.forget_type()) }
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, T> { Iter::new(self.root.as_ref()) }
    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, T> {
        IterMut::new(self.root.as_mut())
    }
    pub fn into_iter(mut self) -> IntoIter<T> {
        IntoIter::new(self.root.take())
    }

    pub fn bisect<'a, F>(&'a self, predicate: F) -> (Option<&'a T>, usize)
    where
        F: Fn(&T) -> bool,
    {
        let len = self.len();
        self.root.as_ref().map_or((None, 0), |root| unsafe {
            let idx = root.borrow().bisect_index(predicate);
            ((idx < len).then(|| root.borrow().select_value(idx).get()), idx)
        })
    }
    pub fn bisect_mut<'a, F>(
        &'a mut self,
        predicate: F,
    ) -> (Option<&'a mut T>, usize)
    where
        F: Fn(&T) -> bool,
    {
        let len = self.len();
        self.root.as_mut().map_or((None, 0), |root| unsafe {
            let idx = root.borrow_valmut().bisect_index(predicate);
            (
                (idx < len)
                    .then(|| root.borrow_valmut().select_value(idx).get_mut()),
                idx,
            )
        })
    }

    pub fn insert(&mut self, at: usize, elt: T) {
        #[cold]
        fn assert_failed(at: usize, len: usize) -> ! {
            panic!("`at` insert index (is {at}) should be <= len (is {len})");
        }
        if at > self.len() {
            assert_failed(at, self.len());
        }
        let tmp = self.split_off(at);
        self.adjoin(elt, tmp);
    }
    pub fn remove(&mut self, at: usize) -> T {
        #[cold]
        fn assert_failed(at: usize, len: usize) -> ! {
            panic!("`at` remove index (is {at}) should be < len (is {len})");
        }
        if at >= self.len() {
            assert_failed(at, self.len());
        }
        let tmp = self.split_off(at + 1);
        let rm = self.pop_back().unwrap();
        self.append(tmp);
        rm
    }
    pub fn rotate(&mut self, new_first: usize) {
        #[cold]
        fn assert_failed(at: usize, len: usize) -> ! {
            panic!("`at` rotate index (is {at}) should be < len (is {len})");
        }
        if new_first >= self.len() {
            assert_failed(new_first, self.len());
        }
        let mut new_left = self.split_off(new_first);
        let new_right = self.split_off(0);
        new_left.append(new_right);
        self.root = new_left.root.take();
    }
}

impl<T> Default for BTreeSeq<T> {
    fn default() -> Self { Self::new() }
}
impl<T: Clone> Clone for BTreeSeq<T> {
    fn clone(&self) -> Self { self.iter().cloned().collect() }
}

impl<T: PartialEq> PartialEq for BTreeSeq<T> {
    fn eq(&self, other: &Self) -> bool { self.iter().eq(other.iter()) }
}
impl<T: Eq> Eq for BTreeSeq<T> {}

impl<T: PartialOrd> PartialOrd for BTreeSeq<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}
impl<T: Ord> Ord for BTreeSeq<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T: fmt::Debug> fmt::Debug for BTreeSeq<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_list().entries(self.iter()).finish()
    }
}

impl<T> FromIterator<T> for BTreeSeq<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut root = NodeRef::new_leaf();
        let mut mut_node = root.borrow_mut();
        let mut trace_root = None;
        for elt in iter {
            if let Some(new_root) =
                unsafe { mut_node.insert(mut_node.buflen() as _, elt) }
            {
                let _ = trace_root.insert(new_root);
            }
        }
        let mut root = trace_root
            .map(|node| node.forget_type())
            .unwrap_or_else(|| root.forget_type());
        root.borrow_mut().correct_treelen_invariant_subtree();

        if root.treelen() == 0 {
            unsafe { drop(Box::from_raw(root.node.as_ptr())) }
            Self::new()
        } else {
            Self { root: Some(root) }
        }
    }
}
impl<T> Extend<T> for BTreeSeq<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let tmp: BTreeSeq<T> = iter.into_iter().collect();
        self.append(tmp);
    }
}

impl<T> Index<usize> for BTreeSeq<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        #[cold]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("`index` index (is {index}) should be < len (is {len})");
        }
        if index >= self.len() {
            assert_failed(index, self.len());
        }
        debug_assert!(self.root.is_some());
        let root = self.root.as_ref().unwrap();
        unsafe { root.borrow().select_value(index).get() }
    }
}
impl<T> IndexMut<usize> for BTreeSeq<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        #[cold]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("`index` index (is {index}) should be < len (is {len})");
        }
        if index >= self.len() {
            assert_failed(index, self.len());
        }
        debug_assert!(self.root.is_some());
        let root = self.root.as_mut().unwrap();
        unsafe { root.borrow_valmut().select_value(index).get_mut() }
    }
}

impl<T> Drop for BTreeSeq<T> {
    fn drop(&mut self) { self.root.take().map(|mut root| root.drop_subtree()); }
}

#[cfg(test)]
mod debug;

#[cfg(test)]
mod tests_node {
    use super::*;

    #[test]
    fn push_front() {
        let mut root = NodeRef::new_leaf();
        let mut mut_node = root.borrow_mut();
        let mut trace_root = None;

        let start = 0;
        let end = 300;
        unsafe {
            for i in (start..end).rev() {
                if let Some(new_root) = mut_node.insert(0, i) {
                    let _ = trace_root.insert(new_root);
                }
            }
        }

        let mut root = trace_root
            .map(|r| r.forget_type())
            .unwrap_or_else(|| root.forget_type());
        root.drop_subtree()
    }
    #[test]
    fn push_back() {
        let mut root = NodeRef::new_leaf();
        let mut mut_node = root.borrow_mut();
        let mut trace_root = None;

        let start = 0;
        let end = 300;
        unsafe {
            for i in start..end {
                if let Some(new_root) =
                    mut_node.insert(mut_node.buflen() as _, i)
                {
                    let _ = trace_root.insert(new_root);
                }

                // if let Some(r) = trace_root.as_ref() {
                //     eprintln!();
                //     debug::visualize(r.borrow().forget_type());
                // }
            }
        }

        let mut root = trace_root
            .map(|r| r.forget_type())
            .unwrap_or_else(|| root.forget_type());

        eprintln!();
        debug::visualize(root.borrow());

        root.drop_subtree()
    }

    fn from_iter<T>(iter: impl IntoIterator<Item = T>) -> OwnedNodeRef<T> {
        let mut root = NodeRef::new_leaf();
        let mut mut_node = root.borrow_mut();
        let mut trace_root = None;
        for elt in iter {
            if let Some(new_root) =
                unsafe { mut_node.insert(mut_node.buflen() as _, elt) }
            {
                let _ = trace_root.insert(new_root);
            }
        }
        let mut root = trace_root
            .map(|r| r.forget_type())
            .unwrap_or_else(|| root.forget_type());
        root.borrow_mut().correct_treelen_invariant_subtree();
        root
    }

    fn from_iter_rev<T, I: Iterator<Item = T> + DoubleEndedIterator>(
        iter: impl IntoIterator<IntoIter = I>,
    ) -> OwnedNodeRef<T> {
        let mut root = NodeRef::new_leaf();
        let mut mut_node = root.borrow_mut();
        let mut trace_root = None;
        for elt in iter.into_iter().rev() {
            if let Some(new_root) = unsafe { mut_node.insert(0, elt) } {
                let _ = trace_root.insert(new_root);
            }
        }
        let mut root = trace_root
            .map(|r| r.forget_type())
            .unwrap_or_else(|| root.forget_type());
        root.borrow_mut().correct_treelen_invariant_subtree();
        root
    }

    fn test_adjoin(lens: &[usize]) {
        for &leftlen in lens {
            for &rightlen in lens {
                let left_iter = 0..=leftlen - 1;
                let mid_elt = leftlen;
                let right_iter = leftlen + 1..=leftlen + rightlen;

                let left = from_iter(left_iter);
                let right = from_iter_rev(right_iter);
                let mut root = left.adjoin(mid_elt, right);
                debug::assert_invariants(root.borrow());

                let actual: Vec<_> = root.borrow().iter().copied().collect();
                let expected: Vec<_> = (0..=leftlen + rightlen).collect();
                assert_eq!(actual, expected);
                root.drop_subtree()
            }
        }
    }

    #[test]
    #[cfg(not(miri))]
    fn adjoin_many() {
        let lens: Vec<_> = (1..=300).collect();
        test_adjoin(&lens);
        test_adjoin(&[100_000]);
    }

    #[test]
    fn adjoin_corner() {
        let lens = [
            1,
            B - 1,
            B,
            2 * B - 1,
            2 * B,
            (2 * B - 1) * B + (B - 1),
            (2 * B - 1) * (B + 1),
            (2 * B + 1) * B,
            (2 * B - 1) * (B * B + B + 1),
            B * (2 * B * B + B + 1),
        ];
        test_adjoin(&lens);
    }

    fn test_split(lens: &[usize]) {
        for &len in lens {
            for i in 1..len {
                eprintln!("testing: {:?}", (i, len));

                let tree = from_iter(0..len);
                let [left, right] =
                    unsafe { tree.split_off(i).map(|o| o.unwrap()) };

                assert!(left.borrow().iter().copied().eq(0..i));
                assert!(right.borrow().iter().copied().eq(i..len));

                debug::assert_invariants(left.borrow());
                debug::assert_invariants(right.borrow());

                let [mut left, mut right] = [left, right];
                left.drop_subtree();
                right.drop_subtree();
            }
        }
    }

    #[test]
    fn split_corner() {
        let lens = [
            1,
            B - 1,
            B,
            2 * B - 1,
            2 * B,
            (2 * B - 1) * B + (B - 1),
            (2 * B - 1) * (B + 1),
            (2 * B + 1) * B,
            (2 * B - 1) * (B * B + B + 1),
            B * (2 * B * B + B + 1),
        ];
        test_split(&lens);
    }

    #[test]
    #[cfg(not(miri))]
    fn split_many() {
        let lens: Vec<_> = (1..=300).collect();
        test_split(&lens);
    }

    fn test_iter(n: usize) {
        let tree = from_iter(0..n);

        assert!(tree.borrow().iter().copied().eq(0..n));
        assert!(tree.borrow().iter().copied().rev().eq((0..n).rev()));

        let mut tree = tree;
        tree.drop_subtree()
    }

    #[test]
    fn iter_once() { test_iter(300); }

    #[test]
    #[cfg(not(miri))]
    fn iter_many() {
        for n in 1..=5000 {
            test_iter(n);
        }
        test_iter(1_000_000);
    }
}

#[cfg(test)]
mod tests_tree {
    use super::*;

    #[test]
    fn sanity_check() {
        let a = BTreeSeq::<()>::default();
        assert!(a.is_empty());
        assert_eq!(a.len(), 0);
        assert!(a.iter().eq(None::<&()>));
        assert_eq!(a, a);
        assert_eq!(a, a.clone());

        let mut a: BTreeSeq<_> = Some(0).into_iter().collect();
        assert!(!a.is_empty());
        assert_eq!(a.len(), 1);
        assert!(a.iter().eq(Some(&0)));
        assert_eq!(a, a);
        assert_eq!(a, a.clone());
        assert_eq!(a[0], 0);
        a[0] = 1;
        assert_eq!(a[0], 1);

        a.push_front(0);
        a.extend(2..30);
        assert_eq!(a.len(), 30);
        assert_eq!(a.bisect(|&x| x < 20), (Some(&20), 20));
    }
}
