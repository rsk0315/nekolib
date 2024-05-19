use std::{
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

const B: usize = 6;
pub const CAPACITY: usize = 2 * B - 1;
pub const MIN_LEN_AFTER_SPLIT: usize = B - 1;

mod marker;

struct LeafNode<T, R> {
    parent: Option<NonNull<InternalNode<T, R>>>,
    parent_idx: MaybeUninit<u8>,
    len: u8,
    vals: [MaybeUninit<T>; CAPACITY],
    reduced: MaybeUninit<R>,
}

impl<T, R> LeafNode<T, R> {
    // Initializes a new `LeafNode` in-place.
    //
    // # Safety
    unsafe fn init(this: *mut Self) {
        unsafe {
            ptr::addr_of_mut!((*this).parent).write(None);
            ptr::addr_of_mut!((*this).len).write(0);
        }
    }

    fn new() -> Box<Self> {
        unsafe {
            let mut leaf = MaybeUninit::<Self>::uninit();
            LeafNode::init(leaf.as_mut_ptr());
            Box::new(leaf.assume_init())
        }
    }
}

#[repr(C)]
struct InternalNode<T, R> {
    data: LeafNode<T, R>,
    count: usize,
    edges: [MaybeUninit<BoxedNode<T, R>>; 2 * B],
}

impl<T, R> InternalNode<T, R> {
    // # Safety
    // An invariant of internal edges is that they have at least one
    // initialized and valid edge. This function does not set up such an
    // edge.
    unsafe fn new() -> Box<Self> {
        unsafe {
            let mut node = MaybeUninit::<Self>::uninit();
            LeafNode::init(ptr::addr_of_mut!((*node.as_mut_ptr()).data));
            ptr::addr_of_mut!((*node.as_mut_ptr()).count).write(0);
            Box::new(node.assume_init())
        }
    }
}

type BoxedNode<T, R> = NonNull<LeafNode<T, R>>;

pub struct NodeRef<BorrowType, T, R, Type> {
    height: u8,
    node: NonNull<LeafNode<T, R>>,
    _marker: PhantomData<(BorrowType, Type)>,
}

pub type Root<T, R> = NodeRef<marker::Owned, T, R, marker::LeafOrInternal>;

impl<'a, T: 'a, R: 'a, Type> Copy for NodeRef<marker::Immut<'a>, T, R, Type> {}
impl<'a, T: 'a, R: 'a, Type> Clone for NodeRef<marker::Immut<'a>, T, R, Type> {
    fn clone(&self) -> Self { *self }
}

unsafe impl<BorrowType, T: Sync, R: Sync, Type> Sync
    for NodeRef<BorrowType, T, R, Type>
{
}

unsafe impl<T: Sync, R: Sync, Type> Send
    for NodeRef<marker::Immut<'_>, T, R, Type>
{
}
unsafe impl<T: Send, R: Send, Type> Send
    for NodeRef<marker::Mut<'_>, T, R, Type>
{
}
unsafe impl<T: Send, R: Send, Type> Send
    for NodeRef<marker::ValMut<'_>, T, R, Type>
{
}
unsafe impl<T: Send, R: Send, Type> Send
    for NodeRef<marker::Owned, T, R, Type>
{
}
unsafe impl<T: Send, R: Send, Type> Send
    for NodeRef<marker::Dying, T, R, Type>
{
}

impl<T, R> NodeRef<marker::Owned, T, R, marker::Leaf> {
    pub fn new_leaf() -> Self { Self::from_new_leaf(LeafNode::new()) }

    fn from_new_leaf(leaf: Box<LeafNode<T, R>>) -> Self {
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
    unsafe fn from_new_internal(
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

impl<BorrowType, T, R, Type> NodeRef<BorrowType, T, R, Type> {
    pub fn len(&self) -> usize {
        unsafe { usize::from((*Self::as_leaf_ptr(self)).len) }
    }
    pub fn height(&self) -> u8 { self.height }
}

impl<'a, T, R> NodeRef<marker::Mut<'a>, T, R, marker::Internal> {
    /// # Safety
    /// Every item returned by `range` is a valid edge index for the
    /// node.
    unsafe fn correct_childrens_parent_links<I: Iterator<Item = usize>>(
        &mut self,
        range: I,
    ) {
        for i in range {
            debug_assert!(i <= self.len());
            // unsafe { Handle::new_edge(self.reborrow_mut(), i) }
            //     .correct_parent_link();
            todo!()
        }
    }

    fn correct_all_childrens_parent_links(&mut self) {
        let len = self.len();
        unsafe { self.correct_childrens_parent_links(0..=len) };
    }
}

impl<'a, T: 'a, R: 'a> NodeRef<marker::Mut<'a>, T, R, marker::LeafOrInternal> {
    fn set_parent_link(
        &mut self,
        parent: NonNull<InternalNode<T, R>>,
        parent_idx: usize,
    ) {
        let leaf = Self::as_leaf_ptr(self);
        unsafe { (*leaf).parent = Some(parent) };
        unsafe { (*leaf).parent_idx.write(parent_idx as _) };
    }
}

mod node_cast;

pub struct Handle<Node, Type> {
    node: Node,
    idx: usize,
    _marker: PhantomData<Type>,
}

mod handle;
