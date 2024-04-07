use std::{
    marker::PhantomData,
    mem::MaybeUninit,
    ops::RangeBounds,
    ptr::{self, NonNull},
};

const B: usize = 6;
const CAPACITY: usize = 2 * B - 1;

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
        let parent = self.parent()?;
        let idx = unsafe { (*self.node.as_ptr()).parent_idx.assume_init() };
        Some((parent, idx))
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
                        (LeafFst | LeafMid | LeafLst, _) => unreachable!(),
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
                        if i == 0 {
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
    fn adjoin(self, _sep: T, _other: Self) { todo!() }

    #[must_use]
    fn insert(self, _i: usize, _elt: T) -> (NonNull<LeafNode<T>>, u8) {
        todo!()
    }

    #[must_use]
    fn split_half<NodeType>(
        self,
    ) -> ([NodeRef<marker::Mut<'a>, T, NodeType>; 2], T) {
        // The field `.parent_idx` (of `self.node` and its siblings) should
        // be repaired by the caller, as we can't determine the parents of
        // these nodes at this point.
        debug_assert!(self.is_full() || self.is_leaf());
        let (children, pop) = LeafNode::split_half(self.node);
        let children = children.map(|node| NodeRef {
            node,
            height: 0,
            _marker: PhantomData,
        });
        (children, pop)
    }
}

impl<'a, T> MutInternalNodeRef<'a, T> {
    #[must_use]
    fn insert<BorrowType>(
        self,
        _i: usize,
        _elt: T,
        [_left, _right]: [NodeRef<marker::Mut<'a>, T, BorrowType>; 2],
    ) -> (NonNull<LeafNode<T>>, u8) {
        todo!()
    }

    fn repair_children(&mut self) {
        // Repair `.parent` and `.parent_idx` of the children of this
        // node, as well as `.treelen` of this node.
        todo!()
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
    fn get_internal_ptr(&self) -> Option<*mut InternalNode<T>> {
        (self.is_internal()).then(|| self.node.as_ptr() as *mut InternalNode<T>)
    }
    fn is_leaf(&self) -> bool { self.height == 0 }
    fn is_internal(&self) -> bool { self.height > 0 }
    fn is_full(&self) -> bool {
        let ptr = self.node.as_ptr();
        unsafe { (*ptr).buflen as usize == CAPACITY }
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
        // unsafe {
        //     let left = (*left.as_ptr()).borrow_mut();
        //     let right = (*right.as_ptr()).borrow_mut();
        //     left.adjoin(sep, right);
        // }
        todo!()
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
    pub fn adjoin(&mut self, sep: T, other: BTreeSeq<T>) {
        match (self.root, other.root) {
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
        for i in 0..11 {
            a.push_back(i);
            assert_eq!(a.len(), i + 1);
        }
        a.visualize();
    }
}
