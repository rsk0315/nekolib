use std::{
    marker::PhantomData,
    mem::MaybeUninit,
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

struct RootNode<T> {
    node_ref: OwnedNodeRef<T>,
}

pub struct BTreeSeq<T> {
    root: Option<NonNull<RootNode<T>>>,
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
    fn new_internal(child: OwnedNodeRef<T>) -> Self {
        let mut new_node = unsafe { InternalNode::new() };
        new_node.children[0].write(child.node);
        unsafe { NodeRef::from_new_internal(new_node, child.height + 1) }
    }
    /// # Safety
    /// `height` must be greater than zero.
    unsafe fn from_new_internal(
        internal: Box<InternalNode<T>>,
        height: u8,
    ) -> Self {
        debug_assert!(height > 0);
        let node = NonNull::from(Box::leak(internal)).cast();
        let mut this = NodeRef { height, node, _marker: PhantomData };
        this.borrow_mut().correct_parent_children_invariant();
        this
    }
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
}

impl<'a, T, NodeType> NodeRef<marker::Mut<'a>, T, NodeType> {
    fn reborrow_mut(&mut self) -> NodeRef<marker::Mut<'a>, T, NodeType> {
        unsafe { self.cast() }
    }
    unsafe fn promote(&mut self) -> NodeRef<marker::Owned, T, NodeType> {
        unsafe { self.cast() }
    }
}

impl<BorrowType, T, NodeType> NodeRef<BorrowType, T, NodeType> {
    fn buflen(&self) -> u8 { unsafe { (*self.node.as_ptr()).buflen } }
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
        if let Some((parent, idx)) = self.parent() {
            let idx = idx as usize;
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
        if let Some((parent, idx)) = self.parent() {
            let idx = idx as usize;
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
    fn parent(&self) -> Option<(NodeRef<BorrowType, T, marker::Internal>, u8)> {
        let height = self.height;
        unsafe { (*self.node.as_ptr()).parent }.map(|(parent, idx)| unsafe {
            (NodeRef::from_internal(parent, height + 1), idx)
        })
    }
}

impl<'a, T> NodeRef<marker::Mut<'a>, T, marker::Internal> {
    fn correct_parent_children_invariant(&mut self) {
        let init_len = self.buflen() as usize;
        let mut treelen = init_len;
        let children_ref = self.children_ref();
        let ptr = self.node.cast();
        for i in 0..=init_len {
            let child = children_ref[i];
            unsafe { (*child.as_ptr()).parent = Some((ptr, i as _)) }
            let child_ref = MutNodeRef::from_node(child, self.height - 1);
            treelen += child_ref.treelen();
        }
        unsafe { (*ptr.as_ptr()).treelen = treelen }

        // TBD: When we split nodes and correct the invariant of their
        // links, other invariants may be messed up if we update
        // `.treelen` (probably). If the caller of this function does
        // `.buflen -= 1` properly, then are we happy? Maybe NO.
        //
        // Note that this function invalidates some references if any.
    }
}

impl<T> OwnedNodeRef<T> {
    fn adjoin(mut self, med: T, mut other: Self) -> Self {
        let mut left = self.borrow_mut();
        let mut right = other.borrow_mut();

        if left.height < right.height {
            while left.height < right.height {
                // SAFETY: 0 <= left.height < right.height
                right = right.first_child().unwrap();
            }
            let (mut parent, idx) = right.parent().unwrap();
            unsafe {
                let node = NodeRef::new_single_internal(med, left, right);
                let root1 = parent.insert(idx, node).map(|o| o.forget_type());
                let root2 = parent.child(idx).unwrap().underflow();
                root2.or_else(|| root1).unwrap_or_else(|| other)
            }
        } else if left.height > right.height {
            while left.height > right.height {
                left = left.last_child().unwrap();
            }
            let (mut parent, idx) = left.parent().unwrap();
            unsafe {
                let node = NodeRef::new_single_internal(med, left, right);
                let root1 = parent.insert(idx, node).map(|o| o.forget_type());
                let root2 = parent.child(idx).unwrap().underflow();
                root2.or_else(|| root1).unwrap_or_else(|| self)
            }
        } else {
            if ((left.buflen() + right.buflen() + 1) as usize) <= CAPACITY {
                // Note that `left` and `right` are roots, so it is not
                // necessarily true that |left| == |right| == B - 1.
                // Anyway, we do not have to allocate a new node. We
                // merge them into one of them and deallocate the other.
                left.append(right);
                unsafe { left.promote() }
            } else {
                // At most one of them may be underfull, but we can
                // resolve it by rotate properly.
                let mut node =
                    unsafe { NodeRef::new_single_internal(med, left, right) };
                let mut node_mut = node.borrow_mut();
                let mut left = node_mut.child(0).unwrap();
                let mut right = node_mut.child(1).unwrap();
                left.rotate(&mut right);
                node.forget_type()
            }
        }
    }
    fn drop_subtree(&mut self) {
        let dying: DyingNodeRef<_> = unsafe { self.cast() };
        dying.drop_subtree();
    }
}

impl<T> DyingNodeRef<T> {
    fn drop_subtree(self) {
        let init_len = self.buflen() as usize;
        let ptr = self.node.as_ptr();
        unsafe {
            for e in &mut (*ptr).buf[..init_len] {
                e.assume_init_drop()
            }
        }
        match self.force() {
            ForceResult::Leaf(leaf) => unsafe {
                drop(Box::from_raw(leaf.node.as_ptr()));
            },
            ForceResult::Internal(internal) => {
                let ptr = internal.as_internal_ptr();
                unsafe {
                    for e in &mut (*ptr).children[..=init_len] {
                        let child = DyingNodeRef {
                            node: e.assume_init(),
                            height: self.height - 1,
                            _marker: PhantomData,
                        };
                        child.drop_subtree();
                    }
                    drop(Box::from_raw(ptr));
                }
            }
        }
    }
}

impl<BorrowType, T> ForceResult<BorrowType, T> {
    fn leaf(self) -> Option<NodeRef<BorrowType, T, marker::Leaf>> {
        if let Self::Leaf(leaf) = self { Some(leaf) } else { None }
    }
    fn internal(self) -> Option<NodeRef<BorrowType, T, marker::Internal>> {
        if let Self::Internal(internal) = self { Some(internal) } else { None }
    }
}

struct InsertResult<'a, T> {
    // The caller may use this to fix up the invariant of `.treelen`.
    leaf: MutLeafNodeRef<'a, T>,
    new_root: Option<OwnedNodeRef<T>>,
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
        unsafe {
            match (self.force(), other.force()) {
                (Leaf(mut left), Leaf(mut right)) => left.rotate(&mut right),
                (Internal(mut left), Internal(mut right)) => {
                    left.rotate(&mut right);
                }
                _ => unreachable!(),
            };
        }
    }
    fn append(&mut self, other: Self) {
        use ForceResult::*;
        unsafe {
            match (self.force(), other.force()) {
                (Leaf(mut left), Leaf(mut right)) => left.append(right),
                (Internal(mut left), Internal(mut right)) => {
                    left.append(right);
                }
                _ => unreachable!(),
            };
        }
    }
}

impl<'a, T> MutLeafNodeRef<'a, T> {
    /// # Safety
    /// `i <= buflen`
    pub unsafe fn insert(
        &mut self,
        i: u8,
        elt: T,
    ) -> Option<NodeRef<marker::Owned, T, marker::Internal>> {
        // We do not maintain the invariant of `.treelen` to keep the
        // amortized complexity constant. This is preferable for
        // consecutive insertions like `.collect()` or `.extend()`.
        debug_assert!(i <= self.buflen());

        if (self.buflen() as usize) < CAPACITY {
            self.insert_fit(i, elt);
            None
        } else {
            let (orphan, new_parent) = self.purge_and_insert(i, elt);
            if let Some((mut parent, par_i)) = new_parent {
                parent.insert(par_i, orphan)
            } else {
                Some(orphan)
            }
        }
    }

    fn purge_and_insert(
        &mut self,
        i: u8,
        elt: T,
    ) -> (
        NodeRef<marker::Owned, T, marker::Internal>,
        Option<(NodeRef<marker::Mut<'_>, T, marker::Internal>, u8)>,
    ) {
        let mut orphan = NodeRef::new_leaf();
        let parent = self.parent();
        let i = i as usize;
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

    fn insert_fit(&mut self, i: u8, elt: T) {
        let ptr = self.node.as_ptr();
        unsafe {
            array_insert(&mut (*ptr).buf, i as _, (*ptr).buflen as _, elt);
            (*ptr).buflen += 1;
        }
    }

    pub unsafe fn underflow(
        &mut self,
    ) -> Option<NodeRef<marker::Owned, T, marker::LeafOrInternal>> {
        // If it does not have a parent, then it is the root and nothing
        // has to be done.
        let (mut parent, idx) = self.parent()?;
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
            [Some(mut left), _] => {
                // |left| + |self| + 1 < 2 * B - 1
                // Take an element from the parent, and check it.
                left.merge(self, false);
                if parent.buflen() == 0 {
                    // Now `self` is the new root, so deallocate it and
                    // promote `self`.
                    unsafe { drop(Box::from_raw(parent.as_internal_ptr())) }
                    Some(self.promote().forget_type())
                } else {
                    parent.underflow()
                }
            }
            [_, Some(mut right)] => {
                self.merge(&mut right, true);
                if parent.buflen() == 0 {
                    unsafe { drop(Box::from_raw(parent.as_internal_ptr())) }
                    Some(self.promote().forget_type())
                } else {
                    parent.underflow()
                }
            }
            [None, None] => unreachable!(),
        }
    }

    fn rotate(&mut self, other: &mut Self) {
        let (parent, idx) =
            if let Some(o) = self.parent() { o } else { return };
        let idx = idx as usize;
        let left_ptr = self.node.as_ptr();
        let right_ptr = other.node.as_ptr();
        let parent_ptr = parent.as_internal_ptr();
        unsafe {
            let left_buf = &mut (*left_ptr).buf;
            let right_buf = &mut (*right_ptr).buf;
            let mid = &mut (*parent_ptr).data.buf[idx];
            let leftlen = (*left_ptr).buflen as usize;
            let rightlen = (*right_ptr).buflen as usize;
            debug_assert!(leftlen + rightlen >= 2 * MIN_BUFLEN);
            let rightlen_new = array_rotate_3(
                left_buf, mid, right_buf, leftlen, rightlen, MIN_BUFLEN,
            );
            (*left_ptr).buflen = MIN_BUFLEN as _;
            (*right_ptr).buflen = rightlen_new as _;
        }
    }
    fn merge(&mut self, other: &mut Self, self_left: bool) {
        debug_assert!(self.parent().is_none());
        if self_left {
            let left_ptr = self.node.as_ptr();
            let right_ptr = other.node.as_ptr();
            unsafe {
                let (parent, idx) = (*left_ptr).parent.unwrap();
                let idx = idx as usize;
                let parent_buf = &mut (*parent.as_ptr()).data.buf;
                let parent_len = (*parent.as_ptr()).data.buflen as usize;
                let par_elt = array_remove(parent_buf, idx, parent_len);
                let parent_children = &mut (*parent.as_ptr()).children;
                let _ = array_remove(parent_children, idx + 1, parent_len + 1);
                (*parent.as_ptr()).data.buflen -= 1;
                let left_buf = &mut (*left_ptr).buf;
                let right_buf = &(*right_ptr).buf;
                let leftlen = (*left_ptr).buflen as usize;
                let rightlen = (*right_ptr).buflen as usize;
                left_buf[leftlen].write(par_elt);
                let leftlen = leftlen + 1;
                array_splice(left_buf, leftlen, leftlen, right_buf, rightlen);
                (*left_ptr).buflen = (leftlen + rightlen) as _;
                drop(Box::from_raw(right_ptr));
            }
        } else {
            let left_ptr = other.node.as_ptr();
            let right_ptr = self.node.as_ptr();
            unsafe {
                let (parent, idx) = (*left_ptr).parent.unwrap();
                let idx = idx as usize;
                let parent_buf = &mut (*parent.as_ptr()).data.buf;
                let parent_len = (*parent.as_ptr()).data.buflen as usize;
                let par_elt = array_remove(parent_buf, idx, parent_len);
                let parent_children = &mut (*parent.as_ptr()).children;
                let _ = array_remove(parent_children, idx + 1, parent_len + 1);
                (*parent.as_ptr()).data.buflen -= 1;
                let left_buf = &(*left_ptr).buf;
                let right_buf = &mut (*right_ptr).buf;
                let leftlen = (*left_ptr).buflen as usize;
                let rightlen = (*right_ptr).buflen as usize;
                array_insert(right_buf, 0, rightlen, par_elt);
                let rightlen = rightlen + 1;
                array_splice(right_buf, 0, rightlen, left_buf, leftlen);
                (*right_ptr).buflen = (leftlen + rightlen) as _;
                drop(Box::from_raw(left_ptr));
            }
        }
    }
    fn append(&mut self, mut other: Self) {
        let left_ptr = self.node.as_ptr();
        let right_ptr = other.node.as_ptr();
        unsafe {
            let left_buf = &mut (*left_ptr).buf;
            let right_buf = &(*right_ptr).buf;
            let leftlen = (*left_ptr).buflen as usize;
            let rightlen = (*right_ptr).buflen as usize;
            let newlen = leftlen + rightlen;
            debug_assert!(leftlen + rightlen <= CAPACITY);
            array_splice(left_buf, leftlen, leftlen, right_buf, rightlen);
            (*left_ptr).buflen = newlen as _;
            drop(Box::from_raw(other.node.as_ptr()))
        }
    }
}

impl<'a, T> MutInternalNodeRef<'a, T> {
    /// # Safety
    /// `i <= buflen`
    unsafe fn insert(
        &mut self,
        i: u8,
        orphan: NodeRef<marker::Owned, T, marker::Internal>,
    ) -> Option<NodeRef<marker::Owned, T, marker::Internal>> {
        debug_assert!(i <= self.buflen());

        if (self.buflen() as usize) < CAPACITY {
            self.insert_fit(i, orphan);
            None
        } else {
            let (orphan, new_parent) = self.purge_and_insert(i, orphan);

            if let Some((mut parent, par_i)) = new_parent {
                parent.insert(par_i, orphan)
            } else {
                Some(orphan)
            }
        }
    }

    fn purge_and_insert(
        &mut self,
        i: u8,
        node: NodeRef<marker::Owned, T, marker::Internal>,
    ) -> (
        NodeRef<marker::Owned, T, marker::Internal>,
        Option<(NodeRef<marker::Mut<'_>, T, marker::Internal>, u8)>,
    ) {
        let i = i as usize;
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
        i: u8,
        orphan: NodeRef<marker::Owned, T, marker::Internal>,
    ) {
        let orphan_ptr = orphan.as_internal_ptr();
        let this = self.as_internal_ptr();
        let i = i as usize;
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
        let (mut parent, idx) = self.parent()?;
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
            [Some(mut left), _] => {
                left.merge(self, false);
                if parent.buflen() == 0 {
                    unsafe { drop(Box::from_raw(parent.as_internal_ptr())) }
                    Some(self.promote().forget_type())
                } else {
                    parent.underflow()
                }
            }
            [_, Some(mut right)] => {
                self.merge(&mut right, true);
                if parent.buflen() == 0 {
                    unsafe { drop(Box::from_raw(parent.as_internal_ptr())) }
                    Some(self.promote().forget_type())
                } else {
                    parent.underflow()
                }
            }
            [None, None] => unreachable!(),
        }
    }

    fn rotate(&mut self, other: &mut Self) { todo!() }
    fn merge(&mut self, other: &mut Self, self_left: bool) { todo!() }
    fn append(&mut self, mut other: Self) {
        todo!();
        unsafe { drop(Box::from_raw(other.as_internal_ptr())) }
    }
}

#[cfg(test)]
mod debug;

#[cfg(test)]
mod tests {
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
                    trace_root.insert(new_root);
                }
            }
        }

        let mut root = trace_root
            .map(|r| r.forget_type())
            .unwrap_or_else(|| root.forget_type());
        unsafe { root.drop_subtree() }
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
                if let Some(new_root) = mut_node.insert(mut_node.buflen(), i) {
                    trace_root.insert(new_root);
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

        unsafe { root.drop_subtree() }
    }
}
