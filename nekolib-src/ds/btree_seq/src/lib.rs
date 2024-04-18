use std::{
    marker::PhantomData,
    mem::MaybeUninit,
    ops::RangeBounds,
    ptr::{self, NonNull},
};

const B: usize = 4;
const CAPACITY: usize = 2 * B - 1;
const MIN_BUFLEN: usize = B - 1;

struct LeafNode<T> {
    buflen: u8,
    buf: [MaybeUninit<T>; CAPACITY],
    parent: Option<NonNull<InternalNode<T>>>,
    parent_idx: MaybeUninit<u8>,
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

struct RootNode<T> {
    node_ref: OwnedNodeRef<T>,
}

enum Todo {}

pub struct BTreeSeq<T> {
    root: Option<NonNull<RootNode<T>>>,
}

pub struct Iter<'a, T>(PhantomData<&'a T>);
pub struct IterMut<'a, T>(PhantomData<&'a mut T>);
pub struct IntoIter<T>(PhantomData<T>);

pub struct Range<'a, T>(PhantomData<&'a T>);
pub struct RangeMut<'a, T>(PhantomData<&'a mut T>);

mod marker {
    use std::marker::PhantomData;

    pub enum Owned {}
    pub enum Dying {}
    pub struct Immut<'a>(PhantomData<&'a ()>);
    pub struct Mut<'a>(PhantomData<&'a mut ()>);

    pub enum Leaf {}
    pub enum Internal {}
    pub enum LeafOrInternal {}
}

trait Traversable {}
impl Traversable for marker::Dying {}
impl<'a> Traversable for marker::Immut<'a> {}
impl<'a> Traversable for marker::Mut<'a> {}

impl<T, NodeType> Copy for NodeRef<marker::Immut<'_>, T, NodeType> {}
impl<'a, T, NodeType> Clone for NodeRef<marker::Immut<'a>, T, NodeType> {
    fn clone(&self) -> Self { self.cast() }
}

fn shr<T>(slice: &mut [T], k: usize) {
    unsafe {
        let src = ptr::addr_of!(slice[0]);
        let dst = ptr::addr_of_mut!(slice[0]).add(k);
        ptr::copy(src, dst, slice.len());
    }
}

impl<T> LeafNode<T> {
    fn new() -> NonNull<Self> {
        let mut node_uninit = MaybeUninit::<Self>::uninit();
        let ptr = node_uninit.as_mut_ptr();
        let node = unsafe {
            ptr::addr_of_mut!((*ptr).buflen).write(0);
            ptr::addr_of_mut!((*ptr).parent).write(None);
            node_uninit.assume_init()
        };
        NonNull::from(Box::leak(Box::new(node)))
    }
    fn singleton(elt: T) -> NonNull<Self> {
        let node = Self::new();
        Self::push(node, elt);
        node
    }
    fn push(node: NonNull<Self>, elt: T) {
        unsafe {
            let ptr = node.as_ptr();
            let init_len = (*ptr).buflen;
            let i = init_len as usize;
            (*ptr).buf[i].write(elt);
            (*ptr).buflen += 1;
        }
    }

    fn split_half(node: NonNull<Self>) -> ([NonNull<Self>; 2], T) {
        // The field `.parent` of split nodes should be repaired by the
        // caller.
        unsafe {
            debug_assert_eq!((*node.as_ptr()).buflen as usize, CAPACITY);
            let left = node;
            let right = Self::new();
            let half = B - 1; // == `(CAPACITY - 1) / 2`
            let src = ptr::addr_of!((*left.as_ptr()).buf[half + 1]);
            let dst = ptr::addr_of_mut!((*right.as_ptr()).buf[0]);
            ptr::copy_nonoverlapping(src, dst, half);
            let pop = (*left.as_ptr()).buf[half].assume_init_read();
            (*left.as_ptr()).buflen = half as _;
            (*right.as_ptr()).buflen = half as _;
            ([left, right], pop)
        }
    }
}

impl<T> InternalNode<T> {
    fn new() -> NonNull<Self> {
        let mut node_uninit = MaybeUninit::<Self>::uninit();
        let ptr = node_uninit.as_mut_ptr();
        let node = unsafe {
            ptr::addr_of_mut!((*ptr).data.buflen).write(0);
            ptr::addr_of_mut!((*ptr).data.parent).write(None);
            ptr::addr_of_mut!((*ptr).treelen).write(0);
            node_uninit.assume_init()
        };
        NonNull::from(Box::leak(Box::new(node)))
    }
    fn single_child(
        child: NonNull<LeafNode<T>>,
        child_treelen: usize,
    ) -> NonNull<Self> {
        let node = Self::new();
        unsafe {
            let ptr = node.as_ptr();
            let child_ptr = child.as_ptr();
            (*child_ptr).parent = Some(node);
            (*child_ptr).parent_idx.write(0);
            (*ptr).children[0].write(child);
            (*ptr).treelen = child_treelen;
        }
        node
    }
    fn push(
        node: NonNull<Self>,
        elt: T,
        child: NonNull<LeafNode<T>>,
        child_treelen: usize,
    ) {
        unsafe {
            let ptr = node.as_ptr();
            let child_ptr = child.as_ptr();
            (*ptr).treelen += 1 + child_treelen;
            let init_len = (*ptr).data.buflen;
            let i = init_len as usize;
            (*child_ptr).parent = Some(node);
            (*child_ptr).parent_idx.write(init_len + 1);
            (*ptr).children[i + 1].write(child);
            (*ptr).data.buf[i].write(elt);
            (*ptr).data.buflen += 1;
        }
    }
    fn as_leaf_ptr(this: NonNull<Self>) -> NonNull<LeafNode<T>> {
        NonNull::new(this.as_ptr() as *mut LeafNode<T>).unwrap()
    }

    fn split_half(node: NonNull<Self>) -> ([NonNull<Self>; 2], T) {
        unsafe {
            debug_assert_eq!((*node.as_ptr()).data.buflen as usize, CAPACITY);
            let left = node;
            let right = Self::new();
            let half = B - 1; // == `(CAPACITY - 1) / 2`
            let src_b = ptr::addr_of!((*left.as_ptr()).data.buf[half + 1]);
            let dst_b = ptr::addr_of_mut!((*right.as_ptr()).data.buf[0]);
            ptr::copy_nonoverlapping(src_b, dst_b, half);
            let src_e = ptr::addr_of!((*left.as_ptr()).children[half + 1]);
            let dst_e = ptr::addr_of_mut!((*right.as_ptr()).children[0]);
            ptr::copy_nonoverlapping(src_e, dst_e, half + 1);
            let pop = (*left.as_ptr()).data.buf[half].assume_init_read();
            (*left.as_ptr()).data.buflen = half as _;
            (*right.as_ptr()).data.buflen = half as _;
            ([left, right], pop)
        }
    }
}

type OwnedNodeRef<T> = NodeRef<marker::Owned, T, marker::LeafOrInternal>;
type DyingNodeRef<T> = NodeRef<marker::Dying, T, marker::LeafOrInternal>;
type ImmutNodeRef<'a, T> =
    NodeRef<marker::Immut<'a>, T, marker::LeafOrInternal>;
type MutNodeRef<'a, T> = NodeRef<marker::Mut<'a>, T, marker::LeafOrInternal>;
type MutLeafNodeRef<'a, T> = NodeRef<marker::Mut<'a>, T, marker::Leaf>;
type MutInternalNodeRef<'a, T> = NodeRef<marker::Mut<'a>, T, marker::Internal>;

impl<T> OwnedNodeRef<T> {
    fn new_leaf(elt: T) -> Self {
        Self {
            node: LeafNode::<T>::singleton(elt),
            height: 0,
            _marker: PhantomData,
        }
    }

    fn first_leaf_mut<'a>(&'a self) -> MutLeafNodeRef<'a, T> {
        let mut node: MutNodeRef<'a, T> = self.cast();
        while let Some(child) = node.first_child() {
            node = child;
        }
        node.cast()
    }
    fn last_leaf_mut<'a>(&'a self) -> MutLeafNodeRef<'a, T> {
        let mut node: MutNodeRef<'a, T> = self.cast();
        while let Some(child) = node.last_child() {
            node = child;
        }
        node.cast()
    }

    fn drop_subtree(&mut self) {
        let mut dying: DyingNodeRef<_> = self.cast();
        dying.drop_subtree();
    }
}

impl<BorrowType, T> NodeRef<BorrowType, T, marker::LeafOrInternal> {
    fn get_internal_ptr(&self) -> Option<*mut InternalNode<T>> {
        (self.is_internal()).then(|| self.node.as_ptr() as *mut InternalNode<T>)
    }
    fn treelen(&self) -> usize {
        unsafe {
            if let Some(internal_ptr) = self.get_internal_ptr() {
                (*internal_ptr).treelen
            } else {
                (*self.node.as_ptr()).buflen as _
            }
        }
    }
}

impl<BorrowType, T> NodeRef<BorrowType, T, marker::Internal> {
    fn as_internal_ptr(&self) -> *mut InternalNode<T> {
        self.node.as_ptr() as *mut InternalNode<T>
    }
}

impl<BorrowType: Traversable, T>
    NodeRef<BorrowType, T, marker::LeafOrInternal>
{
    fn first_child(&self) -> Option<Self> {
        let ptr = self.get_internal_ptr()?;
        let node = unsafe { (*ptr).children[0].assume_init() };
        let height = self.height - 1;
        Some(Self { node, height, _marker: PhantomData })
    }
    fn last_child(&self) -> Option<Self> {
        let ptr = self.get_internal_ptr()?;
        let node = unsafe {
            let init_len = (*ptr).data.buflen as usize;
            (*ptr).children[init_len].assume_init()
        };
        let height = self.height - 1;
        Some(Self { node, height, _marker: PhantomData })
    }
    fn select_child(&self, i: usize) -> Option<Self> {
        let ptr = self.get_internal_ptr()?;
        let node = unsafe {
            let init_len = (*ptr).data.buflen as usize;
            (i <= init_len).then(|| (*ptr).children[i].assume_init())?
        };
        let height = self.height - 1;
        Some(Self { node, height, _marker: PhantomData })
    }
}

impl<BorrowType: Traversable, T> NodeRef<BorrowType, T, marker::Internal> {
    fn first_child(&self) -> NodeRef<BorrowType, T, marker::LeafOrInternal> {
        let ptr = self.as_internal_ptr();
        let node = unsafe { (*ptr).children[0].assume_init() };
        let height = self.height - 1;
        NodeRef { node, height, _marker: PhantomData }
    }
    fn last_child(&self) -> NodeRef<BorrowType, T, marker::LeafOrInternal> {
        let ptr = self.as_internal_ptr();
        let node = unsafe {
            let init_len = (*ptr).data.buflen as usize;
            (*ptr).children[init_len].assume_init()
        };
        let height = self.height - 1;
        NodeRef { node, height, _marker: PhantomData }
    }
    fn select_child(
        &self,
        i: usize,
    ) -> Option<NodeRef<BorrowType, T, marker::LeafOrInternal>> {
        let ptr = self.as_internal_ptr();
        let node = unsafe {
            let init_len = (*ptr).data.buflen as usize;
            (i <= init_len).then(|| (*ptr).children[i].assume_init())?
        };
        let height = self.height - 1;
        Some(NodeRef { node, height, _marker: PhantomData })
    }
}

impl<BorrowType: Traversable, T, NodeType> NodeRef<BorrowType, T, NodeType> {
    fn parent(&self) -> Option<NodeRef<BorrowType, T, marker::Internal>> {
        let ptr = self.node.as_ptr();
        let node = unsafe { InternalNode::as_leaf_ptr((*ptr).parent?) };
        let height = self.height + 1;
        Some(NodeRef { node, height, _marker: PhantomData })
    }
    fn parent_with_index(
        &self,
    ) -> Option<(NodeRef<BorrowType, T, marker::Internal>, u8)> {
        self.parent().map(|parent| unsafe {
            (parent, (*self.node.as_ptr()).parent_idx.assume_init())
        })
    }
    fn left_sibling(&self) -> Option<NodeRef<BorrowType, T, NodeType>> {
        let (parent, parent_idx) = self.parent_with_index()?;
        let left_idx = parent_idx.checked_sub(1)? as usize;
        // The node type of siblings are same as the that of `self`.
        parent.select_child(left_idx).map(|o| o.cast())
    }
    fn right_sibling(&self) -> Option<NodeRef<BorrowType, T, NodeType>> {
        let (parent, parent_idx) = self.parent_with_index()?;
        parent.select_child((parent_idx + 1) as _).map(|o| o.cast())
    }
    fn left_shareable(&self) -> Option<NodeRef<BorrowType, T, NodeType>> {
        let left_sibling = self.left_sibling()?;
        let leftlen = unsafe { (*left_sibling.node.as_ptr()).buflen as usize };
        let rightlen = unsafe { (*self.node.as_ptr()).buflen as usize };
        (leftlen + rightlen >= 2 * MIN_BUFLEN).then(|| left_sibling)
    }
    fn right_shareable(&self) -> Option<NodeRef<BorrowType, T, NodeType>> {
        let right_sibling = self.right_sibling()?;
        let leftlen = unsafe { (*self.node.as_ptr()).buflen as usize };
        let rightlen =
            unsafe { (*right_sibling.node.as_ptr()).buflen as usize };
        (leftlen + rightlen >= 2 * MIN_BUFLEN).then(|| right_sibling)
    }
    fn left_mergeable(&self) -> Option<NodeRef<BorrowType, T, NodeType>> {
        let left_sibling = self.left_sibling()?;
        let leftlen = unsafe { (*left_sibling.node.as_ptr()).buflen as usize };
        let rightlen = unsafe { (*self.node.as_ptr()).buflen as usize };
        (leftlen + rightlen + 1 <= CAPACITY).then(|| left_sibling)
    }
    fn right_mergeable(&self) -> Option<NodeRef<BorrowType, T, NodeType>> {
        let right_sibling = self.right_sibling()?;
        let leftlen = unsafe { (*self.node.as_ptr()).buflen as usize };
        let rightlen =
            unsafe { (*right_sibling.node.as_ptr()).buflen as usize };
        (leftlen + rightlen + 1 <= CAPACITY).then(|| right_sibling)
    }
}

impl<T> DyingNodeRef<T> {
    fn drop_subtree(&mut self) {
        let height = self.height;
        let leaf_ptr = self.node.as_ptr();
        unsafe {
            let init_len = (*leaf_ptr).buflen as usize;
            for e in &mut (*leaf_ptr).buf[..init_len] {
                e.assume_init_drop();
            }
            if let Some(internal_ptr) = self.get_internal_ptr() {
                for e in &(*internal_ptr).children[..=init_len] {
                    let mut child = DyingNodeRef {
                        node: e.assume_init(),
                        height: height - 1,
                        _marker: PhantomData,
                    };
                    child.drop_subtree();
                }
                drop(Box::from_raw(internal_ptr));
            } else {
                drop(Box::from_raw(leaf_ptr));
            }
        }
    }
}

impl<'a, T> ImmutNodeRef<'a, T> {
    #[cfg(test)]
    fn assert_invariant(self) {
        fn dfs<'a, T>(node_ref: ImmutNodeRef<'a, T>) {
            let init_len = unsafe { (*node_ref.node.as_ptr()).buflen as usize };
            let height = node_ref.height;
            if !node_ref.is_root() {
                assert!(!node_ref.is_underfull(), "non-root is underfull");
            }

            if height > 0 {
                let actual = (0..=init_len)
                    .map(|i| node_ref.select_child(i).unwrap().treelen())
                    .sum::<usize>()
                    + init_len;
                let expected = node_ref.treelen();
                assert_eq!(actual, expected, "treelen is inconsistent");

                for i in 0..=init_len {
                    let child = node_ref.select_child(i).unwrap();
                    let actual = child.parent_with_index().unwrap().1 as usize;
                    assert_eq!(actual, i, "parent_idx is inconsistent");
                }

                for i in 0..=init_len {
                    let child = node_ref.select_child(i).unwrap();
                    dfs(child);
                }
            }
        }

        dfs(self);
    }

    #[cfg(test)]
    fn visualize(self)
    where
        T: std::fmt::Debug,
    {
        struct State {
            path: Vec<Kind>,
        }
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Kind {
            IntPre,
            IntFst,
            IntMid,
            IntLst,
            IntSgl,
            LeafFst,
            LeafMid,
            LeafLst,
        }
        use Kind::*;
        impl State {
            fn display<T: std::fmt::Debug>(&self, elt: &T) {
                let mut prefix = "".to_owned();
                for i in 0..self.path.len() - 1 {
                    let k0 = self.path[i];
                    let k1 = self.path[i + 1];
                    let term = i + 1 == self.path.len() - 1;
                    prefix += match (k0, k1) {
                        (IntPre, IntPre) => "    ",
                        (IntPre, IntFst) if term => "┌── ",
                        (IntPre, IntFst | IntMid | IntLst) => "│   ",
                        (IntPre, LeafFst) => "┌── ",
                        (IntPre, LeafMid | LeafLst) => "├── ",
                        (IntFst, IntFst) if term => "├── ",
                        (IntFst, IntPre | IntFst | IntMid | IntLst) => "│   ",
                        (IntFst, LeafFst | LeafMid | LeafLst) => "├── ",
                        (IntMid, IntFst) if term => "├── ",
                        (IntMid, IntPre | IntFst | IntMid | IntLst) => "│   ",
                        (IntMid, LeafFst | LeafMid | LeafLst) => "├── ",
                        (IntLst, IntPre) => "│   ",
                        (IntLst, IntFst) if term => "└── ",
                        (IntLst, IntFst | IntMid | IntLst) => "    ",
                        (IntLst, LeafFst | LeafMid) => "├── ",
                        (IntLst, LeafLst) => "└── ",
                        (IntSgl, IntPre) => "│   ",
                        (IntSgl, IntFst) if term => "└── ",
                        (IntSgl, IntFst | IntMid | IntLst) => "    ",
                        (IntSgl, LeafFst | LeafMid) => "├── ",
                        (IntSgl, LeafLst) => "└── ",
                        (LeafFst | LeafMid | LeafLst, _) => unreachable!(),
                        (_, IntSgl) => unreachable!(),
                    };
                }
                eprintln!("{prefix}{elt:?}");
            }
            fn push(&mut self, k: Kind) { self.path.push(k); }
            fn pop(&mut self) { self.path.pop(); }
        }

        fn dfs<'a, T: std::fmt::Debug>(
            node_ref: ImmutNodeRef<'a, T>,
            state: &mut State,
        ) {
            unsafe {
                let ptr = node_ref.node.as_ptr();
                let init_len = (*ptr).buflen as usize;
                let height = node_ref.height;
                if node_ref.is_internal() {
                    {
                        state.push(Kind::IntPre);
                        dfs(node_ref.first_child().unwrap(), state);
                        state.pop();
                    }
                    for i in 0..init_len {
                        if init_len == 1 {
                            state.push(Kind::IntSgl);
                        } else if i == 0 {
                            state.push(Kind::IntFst);
                        } else if i == init_len - 1 {
                            state.push(Kind::IntLst);
                        } else {
                            state.push(Kind::IntMid);
                        }
                        state.display((*ptr).buf[i].assume_init_ref());
                        dfs(node_ref.select_child(i + 1).unwrap(), state);
                        state.pop();
                    }
                } else {
                    let leaf_ptr = ptr;
                    for i in 0..init_len {
                        if i == 0 {
                            state.push(Kind::LeafFst);
                        } else if i == init_len - 1 {
                            state.push(Kind::LeafLst);
                        } else {
                            state.push(Kind::LeafMid);
                        }
                        state.display((*ptr).buf[i].assume_init_ref());
                        state.pop();
                    }
                }
            }
        }

        let mut state = State { path: vec![] };
        dfs(self, &mut state);
    }
}

impl<'a, T, BorrowType> NodeRef<marker::Mut<'a>, T, BorrowType> {
    fn reborrow_mut(&mut self) -> Self { self.cast() }
}

impl<'a, T> MutNodeRef<'a, T> {
    fn adjoin(self, sep: T, mut right: Self) -> OwnedNodeRef<T> {
        let mut left = self;

        let (node, height) = if left.height < right.height {
            while left.height < right.height {
                right = right.first_child().unwrap();
            }
            let parent = right.parent().unwrap();
            parent.insert(0, sep, [left, right])
        } else if left.height > right.height {
            while left.height > right.height {
                left = left.last_child().unwrap();
            }
            let parent = left.parent().unwrap();
            let init_len = unsafe { (*parent.node.as_ptr()).buflen as usize };
            parent.insert(init_len, sep, [left, right])
        } else {
            let height = left.height;
            let root = InternalNode::single_child(left.node, left.treelen());
            InternalNode::push(root, sep, right.node, right.treelen());
            (InternalNode::as_leaf_ptr(root), height + 1)
        };

        // XXX We have to take care of the invariant of `.buflen` of
        // `left` and `right`. The root node may ignore the lower bound
        // of it, but at least one of `left` and `right` are no longer
        // the root node.
        todo!();

        NodeRef { node, height, _marker: PhantomData }
    }
}

impl<'a, T> MutLeafNodeRef<'a, T> {
    #[must_use]
    fn push_front(self, elt: T) -> (NonNull<LeafNode<T>>, u8) {
        self.insert(0, elt)
    }
    #[must_use]
    fn push_back(self, elt: T) -> (NonNull<LeafNode<T>>, u8) {
        let i = unsafe { (*self.node.as_ptr()).buflen as usize };
        self.insert(i, elt)
    }

    #[must_use]
    fn insert(mut self, i: usize, elt: T) -> (NonNull<LeafNode<T>>, u8) {
        if let Some((parent, parent_idx)) = self.parent_with_index() {
            if self.is_full() {
                let ([mut left, mut right], pop) = self.resolve_overfull();
                if i < B {
                    left.insert_fit(i, elt);
                } else {
                    right.insert_fit(i - B, elt);
                }
                parent.insert(parent_idx as _, pop, [
                    left.reborrow_mut(),
                    right.reborrow_mut(),
                ])
            } else {
                self.insert_fit(i, elt);
                parent.repair_to_root() // NOTE: just `.treelen += 1` will do.
            }
        } else {
            if self.is_full() {
                let ([mut left, mut right], pop) = self.resolve_overfull();
                if i < B {
                    left.insert_fit(i, elt);
                } else {
                    right.insert_fit(i - B, elt);
                }
                let parent = InternalNode::single_child(left.node, B - 1);
                InternalNode::push(parent, pop, right.node, B);
                (InternalNode::as_leaf_ptr(parent), 1)
            } else {
                self.insert_fit(i, elt);
                (self.node, 0)
            }
        }
    }

    fn insert_fit(&mut self, i: usize, elt: T) {
        let ptr = self.node.as_ptr();
        unsafe {
            let init_len = (*ptr).buflen as usize;
            debug_assert!(i <= init_len);
            if i < init_len {
                shr(&mut (*ptr).buf[i..init_len], 1);
            }
            (*ptr).buf[i].write(elt);
            (*ptr).buflen += 1;
        }
    }

    #[must_use]
    fn resolve_overfull(
        self,
    ) -> ([NodeRef<marker::Mut<'a>, T, marker::Leaf>; 2], T) {
        // The field `.parent_idx` (of `self.node` and its siblings) should
        // be repaired by the caller, as we can't determine the parents of
        // these nodes at this point.
        debug_assert!(self.is_full() && self.is_leaf());
        let (children, pop) = LeafNode::split_half(self.node);
        let children = children.map(|node| NodeRef {
            node,
            height: 0,
            _marker: PhantomData,
        });
        (children, pop)
    }

    #[must_use]
    fn resolve_underfull(self) -> NodeRef<marker::Mut<'a>, T, marker::Leaf> {
        // Returns the new `NodeRef`, say `res`, to the node which
        // contains elements of `self`. The parent of `res` may be
        // underfull. The children of `res` are repaired, but `res`
        // should be repaired by the caller.
        debug_assert!(self.is_underfull() && self.is_leaf() && !self.is_root());
        if let Some(left_sibling) = self.left_shareable() {
            left_sibling.share(self);
            self
        } else if let Some(right_sibling) = self.right_shareable() {
            self.share(right_sibling);
            self
        } else if let Some(left_sibling) = self.left_mergeable() {
            left_sibling.merge(self)
        } else if let Some(right_sibling) = self.right_mergeable() {
            self.merge(right_sibling)
        } else {
            unreachable!()
        }
    }

    fn share(&mut self, other: &mut Self) {
        // From their parent's point of view, the sharing does not
        // affect the `.treelen` field.
        unsafe {
            let left = self.node.as_ptr();
            let right = other.node.as_ptr();
            let leftlen_old = (*left).buflen as usize;
            let rightlen_old = (*right).buflen as usize;
            let leftlen_new = (leftlen_old + rightlen_old) / 2;
            let rightlen_new = leftlen_old + rightlen_old - leftlen_new;
            let (parent, parent_idx) = self.parent_with_index().unwrap();
            let parent = parent.node.as_ptr();
            let parent_idx = parent_idx as usize;
            let parent_elt = (*parent).buf[parent_idx].assume_init_read();
            if leftlen_old < rightlen_old {
                (*left).buf[leftlen_old].write(parent_elt);
                let dst = ptr::addr_of_mut!((*left).buf[leftlen_old + 1]);
                let src = ptr::addr_of!((*right).buf[0]);
                let count = leftlen_new - leftlen_old - 1;
                ptr::copy_nonoverlapping(src, dst, count);
                let new_parent_elt = (*right).buf[count].assume_init_read();
                (*parent).buf[parent_idx].write(new_parent_elt);
                let dst = ptr::addr_of_mut!((*right).buf[0]);
                let src = ptr::addr_of!((*right).buf[count + 1]);
                let count = rightlen_new; // (!) check the equation
                ptr::copy(src, dst, count);
            } else {
                todo!();
            }
            (*left).buflen = leftlen_new as _;
            (*right).buflen = rightlen_new as _;
        }
    }
    #[must_use]
    fn merge(self, other: Self) -> Self {
        // If their parent become underfull, the cascading fix-up takes
        // place usually. However if it is the root and we take its only
        // element, we become the new root.
        todo!()
    }
}

impl<'a, T> MutInternalNodeRef<'a, T> {
    #[must_use]
    fn insert<NodeType>(
        mut self,
        i: usize,
        elt: T,
        children: [NodeRef<marker::Mut<'a>, T, NodeType>; 2],
    ) -> (NonNull<LeafNode<T>>, u8) {
        let height = self.height;
        if let Some((parent, parent_idx)) = self.parent_with_index() {
            if self.is_full() {
                let ([mut left, mut right], pop) = self.resolve_overfull();
                left.repair_children();
                right.repair_children();
                if i < B {
                    left.insert_fit(i, elt, children);
                } else {
                    right.insert_fit(i - B, elt, children);
                }
                parent.insert(parent_idx as _, pop, [
                    left.reborrow_mut(),
                    right.reborrow_mut(),
                ])
            } else {
                self.insert_fit(i, elt, children);
                parent.repair_to_root()
            }
        } else {
            if self.is_full() {
                let ([mut left, mut right], pop) = self.resolve_overfull();
                left.repair_children();
                right.repair_children();
                if i < B {
                    left.insert_fit(i, elt, children);
                } else {
                    right.insert_fit(i - B, elt, children)
                }
                let (leftlen, rightlen) = unsafe {
                    (
                        (*left.as_internal_ptr()).treelen,
                        (*right.as_internal_ptr()).treelen,
                    )
                };
                let parent = InternalNode::single_child(left.node, leftlen);
                InternalNode::push(parent, pop, right.node, rightlen);
                (InternalNode::as_leaf_ptr(parent), height + 1)
            } else {
                self.insert_fit(i, elt, children);
                (self.node, height)
            }
        }
    }

    fn insert_fit<NodeType>(
        &mut self,
        i: usize,
        elt: T,
        [left, right]: [NodeRef<marker::Mut<'a>, T, NodeType>; 2],
    ) {
        // We should call `.repair_children()`, the caller does not have
        // the responsibility for it.

        let ptr = self.node.as_ptr() as *mut InternalNode<T>;
        unsafe {
            let init_len = (*ptr).data.buflen as usize;
            debug_assert!(i <= init_len);
            if i < init_len {
                shr(&mut (*ptr).data.buf[i..init_len], 1);
                shr(&mut (*ptr).children[i..=init_len], 1);
            }
            (*ptr).data.buf[i].write(elt);
            (*ptr).children[i].write(left.node);
            (*ptr).children[i + 1].write(right.node);
            (*ptr).data.buflen += 1;
        }
        self.repair_children();
    }

    #[must_use]
    fn resolve_overfull(
        self,
    ) -> ([NodeRef<marker::Mut<'a>, T, marker::Internal>; 2], T) {
        debug_assert!(self.is_full() && self.is_internal());
        let height = self.height;
        let internal_ptr = NonNull::new(self.as_internal_ptr()).unwrap();
        let (children, pop) = InternalNode::split_half(internal_ptr);
        let children = children.map(|node| NodeRef {
            node: InternalNode::as_leaf_ptr(node),
            height,
            _marker: PhantomData,
        });
        (children, pop)
    }

    #[must_use]
    fn repair_to_root(self) -> (NonNull<LeafNode<T>>, u8) {
        let mut cur = self;
        loop {
            cur.repair_children();
            if let Some(parent) = cur.parent() {
                cur = parent;
            } else {
                return (cur.node, cur.height);
            }
        }
    }

    fn repair_children(&mut self) {
        // Repair `.parent` and `.parent_idx` of the children of this
        // node, as well as `.treelen` of this node.
        let ptr = self.as_internal_ptr();
        let parent = NonNull::new(ptr);
        unsafe {
            let init_len = (*ptr).data.buflen as usize;
            let mut treelen = init_len;
            for i in 0..=init_len {
                let child = self.select_child(i).unwrap();
                let ptr = child.node.as_ptr();
                (*ptr).parent = parent;
                (*child.node.as_ptr()).parent_idx.write(i as _);
                treelen += child.treelen();
            }
            (*ptr).treelen = treelen;
        }
    }
}

impl<BorrowType, T, NodeType> NodeRef<BorrowType, T, NodeType> {
    fn forget_type(self) -> NodeRef<BorrowType, T, marker::LeafOrInternal> {
        self.cast()
    }
    fn cast<NewBorrowType, NewNodeType>(
        &self,
    ) -> NodeRef<NewBorrowType, T, NewNodeType> {
        NodeRef {
            node: self.node,
            height: self.height,
            _marker: PhantomData,
        }
    }
    fn is_leaf(&self) -> bool { self.height == 0 }
    fn is_internal(&self) -> bool { self.height > 0 }
    fn is_root(&self) -> bool {
        unsafe { (*self.node.as_ptr()).parent.is_none() }
    }
    fn is_full(&self) -> bool {
        let ptr = self.node.as_ptr();
        unsafe { (*ptr).buflen as usize == CAPACITY }
    }
    fn is_underfull(&self) -> bool {
        let ptr = self.node.as_ptr();
        unsafe { ((*ptr).buflen as usize) < MIN_BUFLEN }
    }
}

impl<T> RootNode<T> {
    fn new(elt: T) -> NonNull<Self> {
        let root = Self {
            node_ref: OwnedNodeRef::<T>::new_leaf(elt).forget_type(),
        };
        NonNull::from(Box::leak(Box::new(root)))
    }

    fn borrow<'a>(&'a self) -> ImmutNodeRef<'a, T> { self.node_ref.cast() }
    fn borrow_mut<'a>(&'a mut self) -> MutNodeRef<'a, T> {
        self.node_ref.cast()
    }

    fn push_front(root: NonNull<RootNode<T>>, elt: T) {
        unsafe {
            let (new_root, new_height) =
                (*root.as_ptr()).node_ref.first_leaf_mut().push_front(elt);
            (*root.as_ptr()).node_ref.node = new_root;
            (*root.as_ptr()).node_ref.height = new_height;
        };
    }
    fn push_back(root: NonNull<RootNode<T>>, elt: T) {
        unsafe {
            let (new_root, new_height) =
                (*root.as_ptr()).node_ref.last_leaf_mut().push_back(elt);
            (*root.as_ptr()).node_ref.node = new_root;
            (*root.as_ptr()).node_ref.height = new_height;
        };
    }
    #[allow(unused)]
    fn adjoin(left: NonNull<RootNode<T>>, sep: T, right: NonNull<RootNode<T>>) {
        unsafe {
            let left_root = (*left.as_ptr()).borrow_mut();
            let right_root = (*right.as_ptr()).borrow_mut();
            let new_root = left_root.adjoin(sep, right_root);
            (*left.as_ptr()).node_ref = new_root;
            drop(Box::from_raw(right.as_ptr()));
        }
    }

    fn drop_subtree(root: NonNull<RootNode<T>>) {
        let ptr = root.as_ptr();
        unsafe { (*ptr).node_ref.drop_subtree() };
    }

    #[cfg(test)]
    fn visualize(root: NonNull<RootNode<T>>)
    where
        T: std::fmt::Debug,
    {
        unsafe { (*root.as_ptr()).borrow().visualize() }
    }

    #[cfg(test)]
    fn assert_invariant(root: NonNull<RootNode<T>>) {
        unsafe { (*root.as_ptr()).borrow().assert_invariant() }
    }
}

impl<T> BTreeSeq<T> {
    pub fn new() -> Self { Self { root: None } }
    pub fn singleton(elt: T) -> Self { Self { root: Some(RootNode::new(elt)) } }

    pub fn len(&self) -> usize {
        self.root
            .map(|root| unsafe { (*root.as_ptr()).node_ref.treelen() })
            .unwrap_or(0)
    }
    pub fn is_empty(&self) -> bool { self.root.is_none() }

    pub fn push_front(&mut self, elt: T) {
        if let Some(root) = self.root {
            RootNode::push_front(root, elt);
        } else {
            self.root = Some(RootNode::new(elt));
        }
    }
    pub fn push_back(&mut self, elt: T) {
        if let Some(root) = self.root {
            RootNode::push_back(root, elt);
        } else {
            self.root = Some(RootNode::new(elt));
        }
    }
    pub fn pop_front(&mut self) -> Option<T> { todo!() }
    pub fn pop_back(&mut self) -> Option<T> { todo!() }

    pub fn insert(&mut self, _i: usize, _elt: T) { todo!() }
    pub fn remove(&mut self, _i: usize) -> Option<T> { todo!() }

    pub fn append(&mut self, _other: BTreeSeq<T>) { todo!() }
    pub fn split_off(&mut self, _at: usize) -> BTreeSeq<T> { todo!() }
    pub fn adjoin(&mut self, sep: T, mut other: BTreeSeq<T>) {
        match (self.root, other.root.take()) {
            (Some(left), Some(right)) => RootNode::adjoin(left, sep, right),
            (Some(left), None) => RootNode::push_back(left, sep),
            (None, Some(right)) => {
                RootNode::push_front(right, sep);
                self.root = Some(right);
            }
            (None, None) => self.root = Some(RootNode::new(sep)),
        }
    }

    pub fn get(&self, _i: usize) -> Option<&T> { todo!() }
    pub fn get_mut(&mut self, _i: usize) -> Option<&mut T> { todo!() }

    pub fn into_iter(self) -> IntoIter<T> { todo!() }
    pub fn iter(&self) -> Iter<'_, T> { todo!() }
    pub fn iter_mut(&mut self) -> IterMut<'_, T> { todo!() }

    pub fn range(&self, _r: impl RangeBounds<usize>) -> Range<'_, T> { todo!() }
    pub fn range_mut(
        &mut self,
        _r: impl RangeBounds<usize>,
    ) -> RangeMut<'_, T> {
        todo!()
    }

    pub fn bisect(&self, _pred: impl FnMut(&T) -> bool) -> usize { todo!() }
    pub fn rotate(&mut self, new_first: usize) {
        let mut tmp = self.split_off(new_first);
        tmp.append(std::mem::take(self));
        *self = tmp;
    }

    #[cfg(test)]
    pub fn visualize(&self)
    where
        T: std::fmt::Debug,
    {
        if let Some(root) = self.root {
            RootNode::visualize(root);
        } else {
            eprintln!(".");
        }
    }

    #[cfg(test)]
    pub fn assert_invariant(&self) {
        if let Some(root) = self.root {
            RootNode::assert_invariant(root);
        }
    }
}

impl<T> Drop for BTreeSeq<T> {
    fn drop(&mut self) {
        if let Some(root) = self.root {
            unsafe {
                RootNode::drop_subtree(root);
                drop(Box::from_raw(root.as_ptr()));
            }
        }
    }
}

impl<T> Default for BTreeSeq<T> {
    fn default() -> Self { Self::new() }
}

impl<T> FromIterator<T> for BTreeSeq<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut res = BTreeSeq::new();
        iter.into_iter().for_each(|elt| res.push_back(elt));
        res
    }
}

impl<T> Extend<T> for BTreeSeq<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        iter.into_iter().for_each(|elt| self.push_back(elt));
    }
}

impl<T> IntoIterator for BTreeSeq<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> { Self::into_iter(self) }
}

impl<'a, T> IntoIterator for &'a BTreeSeq<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Iter<'a, T> { self.iter() }
}

impl<'a, T> IntoIterator for &'a mut BTreeSeq<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> IterMut<'a, T> { self.iter_mut() }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> { todo!() }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> { todo!() }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<&'a mut T> { todo!() }
}

impl<'a, T> Iterator for Range<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> { todo!() }
}

impl<'a, T> Iterator for RangeMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<&'a mut T> { todo!() }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> { todo!() }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> { todo!() }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<&'a mut T> { todo!() }
}

impl<'a, T> DoubleEndedIterator for Range<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> { todo!() }
}

impl<'a, T> DoubleEndedIterator for RangeMut<'a, T> {
    fn next_back(&mut self) -> Option<&'a mut T> { todo!() }
}

impl<T> RootNode<T> {
    fn join(
        _left: Option<NonNull<Self>>,
        _mid: T,
        _right: Option<NonNull<Self>>,
    ) -> NonNull<Self> {
        todo!()
    }

    fn split(
        _root: NonNull<Self>,
        _at: NonNull<LeafNode<T>>,
    ) -> (Option<NonNull<Self>>, Option<NonNull<Self>>) {
        todo!()
    }

    fn select(_root: NonNull<Self>, _i: usize) -> Option<Todo> { todo!() }
}

#[cfg(test)]
mod tests {
    use std::ops::RangeFrom;

    use super::*;

    #[test]
    fn test_empty() {
        let a = BTreeSeq::<()>::new();
        assert_eq!(a.len(), 0);
        assert!(a.is_empty());
    }

    #[test]
    fn test_singleton() {
        let a = BTreeSeq::singleton(());
        assert_eq!(a.len(), 1);
        assert!(!a.is_empty());
    }

    #[test]
    fn test_vz() {
        let node0 = |it: &mut RangeFrom<i32>| {
            let leaf = LeafNode::singleton(it.next().unwrap());
            LeafNode::push(leaf, it.next().unwrap());
            LeafNode::push(leaf, it.next().unwrap());
            leaf
        };
        // `child_treelen` are not important here, for now.
        let node1 = |it: &mut RangeFrom<i32>| {
            let intn = InternalNode::single_child(node0(it), 0);
            let v0 = it.next().unwrap();
            InternalNode::push(intn, v0, node0(it), 0);
            let v1 = it.next().unwrap();
            InternalNode::push(intn, v1, node0(it), 0);
            let v2 = it.next().unwrap();
            InternalNode::push(intn, v2, node0(it), 0);
            InternalNode::as_leaf_ptr(intn)
        };
        let node2 = |it: &mut RangeFrom<i32>| {
            let intn = InternalNode::single_child(node1(it), 0);
            let v0 = it.next().unwrap();
            InternalNode::push(intn, v0, node1(it), 0);
            let v1 = it.next().unwrap();
            InternalNode::push(intn, v1, node1(it), 0);
            let v2 = it.next().unwrap();
            InternalNode::push(intn, v2, node1(it), 0);
            InternalNode::as_leaf_ptr(intn)
        };

        let seq = |node, height| {
            let root = RootNode {
                node_ref: OwnedNodeRef { node, height, _marker: PhantomData },
            };
            let root = NonNull::from(Box::leak(Box::new(root)));
            BTreeSeq { root: Some(root) }
        };

        eprintln!();
        seq(node2(&mut (0..)), 2).visualize();
        seq(node1(&mut (0..)), 1).visualize();
        seq(node0(&mut (0..)), 0).visualize();
        BTreeSeq::<()>::new().visualize();
    }

    #[test]
    fn test_shr() {
        struct Foo([MaybeUninit<String>; 6]);
        unsafe {
            let mut foo = MaybeUninit::<Foo>::uninit().assume_init();
            foo.0[0].write("fourth".to_owned());
            foo.0[1].write("fifth".to_owned());
            shr(&mut foo.0[..2], 3);
            foo.0[0].write("first".to_owned());
            foo.0[1].write("second".to_owned());
            foo.0[2].write("third".to_owned());

            assert_eq!(foo.0[0].assume_init_ref(), "first");
            assert_eq!(foo.0[1].assume_init_ref(), "second");
            assert_eq!(foo.0[2].assume_init_ref(), "third");
            assert_eq!(foo.0[3].assume_init_ref(), "fourth");
            assert_eq!(foo.0[4].assume_init_ref(), "fifth");

            foo.0[0].assume_init_drop();
            foo.0[1].assume_init_drop();
            foo.0[2].assume_init_drop();
            foo.0[3].assume_init_drop();
            foo.0[4].assume_init_drop();
        }
    }

    #[test]
    fn test_push_back() {
        let mut a = BTreeSeq::new();
        for i in 0..200 {
            a.push_back(i);
            assert_eq!(a.len(), i + 1);
        }
        eprintln!();
        a.visualize();
        a.assert_invariant();
    }

    #[test]
    fn test_push_front() {
        let mut a = BTreeSeq::new();
        let n = 200;
        for i in (0..n).rev() {
            a.push_front(i);
            assert_eq!(a.len(), n - i);
        }
        eprintln!();
        a.visualize();
        a.assert_invariant();
    }

    #[test]
    fn test_adjoin() {
        // bad
        let mut left = BTreeSeq::new();
        let leftlen = 300;
        for i in 0..leftlen {
            left.push_back(i);
        }
        let mut right = BTreeSeq::new();
        let rightlen = 50;
        for i in 0..rightlen {
            right.push_back(leftlen + 1 + i);
        }
        left.adjoin(leftlen, right);
        left.visualize();
    }
}
