use std::{
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

use array_insertion::{array_insert, array_splice};
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
    unsafe fn new_single_internal(
        elt: T,
        left: MutNodeRef<T>,
        right: MutNodeRef<T>,
    ) -> Self {
        debug_assert_eq!(left.height, right.height);
        let height = left.height;
        let mut node = unsafe { InternalNode::new() };
        node.data.buf[0].write(elt);
        node.children[0].write(left.node);
        node.children[1].write(right.node);
        let node = NonNull::from(Box::leak(node)).cast();
        let mut this =
            NodeRef { height: height + 1, node, _marker: PhantomData };
        this.borrow_mut().correct_parent_children_invariant();
        this
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

impl<BorrowType, T> NodeRef<BorrowType, T, marker::Leaf> {
    fn forget_type(self) -> NodeRef<BorrowType, T, marker::LeafOrInternal> {
        unsafe { self.cast() }
    }
}

impl<BorrowType, T> NodeRef<BorrowType, T, marker::Internal> {
    fn as_internal_ptr(&self) -> *mut InternalNode<T> {
        self.node.as_ptr() as *mut InternalNode<T>
    }
    fn from_internal(node: NonNull<InternalNode<T>>, height: u8) -> Self {
        debug_assert!(height > 0);
        NodeRef { node: node.cast(), height, _marker: PhantomData }
    }
    fn forget_type(self) -> NodeRef<BorrowType, T, marker::LeafOrInternal> {
        unsafe { self.cast() }
    }
}

impl<T, NodeType> NodeRef<marker::Owned, T, NodeType> {
    fn borrow_mut(&mut self) -> NodeRef<marker::Mut<'_>, T, NodeType> {
        unsafe { self.cast() }
    }
}

impl<'a, T, NodeType> NodeRef<marker::Mut<'a>, T, NodeType> {
    fn reborrow_mut(&mut self) -> NodeRef<marker::Mut<'a>, T, NodeType> {
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
    fn children_mut(&mut self) -> &mut [NonNull<LeafNode<T>>] {
        let init_len = self.buflen() as usize;
        unsafe {
            &mut *(&mut (*self.as_internal_ptr()).children[..=init_len]
                as *mut [MaybeUninit<_>]
                as *mut [NonNull<LeafNode<T>>])
        }
    }
}

impl<BorrowType: Traversable, T, NodeType> NodeRef<BorrowType, T, NodeType> {
    fn parent(&self) -> Option<(NodeRef<BorrowType, T, marker::Internal>, u8)> {
        let height = self.height;
        unsafe { (*self.node.as_ptr()).parent }.map(|(parent, idx)| {
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
    }
}

impl<T> OwnedNodeRef<T> {
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
            let orphan = self.purge_and_insert(i, elt);
            if let Some((mut parent, par_i)) = self.parent() {
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
    ) -> NodeRef<marker::Owned, T, marker::Internal> {
        let mut orphan = NodeRef::new_leaf();
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
            NodeRef::new_single_internal(
                par_elt,
                left.forget_type(),
                right.forget_type(),
            )
        }
    }

    fn insert_fit(&mut self, i: u8, elt: T) {
        let ptr = self.node.as_ptr();
        unsafe {
            array_insert(&mut (*ptr).buf, i as _, (*ptr).buflen as _, elt);
            (*ptr).buflen += 1;
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
        todo!()
    }
}

#[cfg(test)]
mod debug;
