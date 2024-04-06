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
    treelen: usize,
    buf: [MaybeUninit<T>; CAPACITY],
    parent: Option<NonNull<InternalNode<T>>>,
    parent_idx: MaybeUninit<u8>,
}

#[repr(C)]
struct InternalNode<T> {
    data: LeafNode<T>,
    children: [MaybeUninit<NonNull<LeafNode<T>>>; CAPACITY + 1],
}

struct NodeRef<BorrowType, T, NodeType> {
    node: NonNull<LeafNode<T>>,
    height: u8,
    _marker: PhantomData<(BorrowType, T, NodeType)>,
}

struct RootNode<T> {
    node_ref: OwnedNodeRef<T>,
    height: u8,
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
    pub enum Owned {}
    pub enum Dying {}

    pub enum Leaf {}
    pub enum Internal {}
    pub enum LeafOrInternal {}
}

impl<T> LeafNode<T> {
    fn new() -> NonNull<LeafNode<T>> {
        let mut node_uninit = MaybeUninit::<LeafNode<T>>::uninit();
        let ptr = node_uninit.as_mut_ptr();
        let node = unsafe {
            ptr::addr_of_mut!((*ptr).buflen).write(0);
            ptr::addr_of_mut!((*ptr).treelen).write(0);
            ptr::addr_of_mut!((*ptr).parent).write(None);
            node_uninit.assume_init()
        };
        NonNull::from(Box::leak(Box::new(node)))
    }
    fn singleton(elt: T) -> NonNull<LeafNode<T>> {
        let node = Self::new();
        unsafe {
            let ptr = node.as_ptr();
            (*ptr).buflen = 1;
            (*ptr).treelen = 1;
            (*ptr).buf[0].write(elt);
        }
        node
    }
}

type OwnedNodeRef<T> = NodeRef<marker::Owned, T, marker::LeafOrInternal>;
type DyingNodeRef<T> = NodeRef<marker::Dying, T, marker::LeafOrInternal>;

impl<T> OwnedNodeRef<T> {
    fn new_leaf(elt: T) -> Self {
        Self {
            node: LeafNode::<T>::singleton(elt),
            height: 0,
            _marker: PhantomData,
        }
    }

    fn drop_subtree(&mut self) {
        let mut dying: DyingNodeRef<_> = self.cast();
        dying.drop_subtree();
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
    fn is_internal(&self) -> bool { self.height > 0 }
}

impl<T> RootNode<T> {
    fn new(elt: T) -> NonNull<Self> {
        let root = Self {
            node_ref: OwnedNodeRef::<T>::new_leaf(elt).forget_type(),
            height: 0,
        };
        NonNull::from(Box::leak(Box::new(root)))
    }

    fn drop_subtree(root: NonNull<RootNode<T>>) {
        let ptr = root.as_ptr();
        unsafe { (*ptr).node_ref.drop_subtree() };
    }
}

impl<T> BTreeSeq<T> {
    pub fn new() -> Self { Self { root: None } }
    pub fn singleton(elt: T) -> Self { Self { root: Some(RootNode::new(elt)) } }

    pub fn len(&self) -> usize {
        self.root
            .map(|root| unsafe {
                (*(*root.as_ptr()).node_ref.node.as_ptr()).treelen
            })
            .unwrap_or(0)
    }
    pub fn is_empty(&self) -> bool { self.root.is_none() }

    pub fn push_back(&mut self, elt: T) { todo!() }
    pub fn push_front(&mut self, elt: T) { todo!() }
    pub fn pop_back(&mut self) -> Option<T> { todo!() }
    pub fn pop_front(&mut self) -> Option<T> { todo!() }

    pub fn insert(&mut self, i: usize, elt: T) { todo!() }
    pub fn remove(&mut self, i: usize) -> Option<T> { todo!() }

    pub fn append(&mut self, other: BTreeSeq<T>) { todo!() }
    pub fn split_off(&mut self, at: usize) -> BTreeSeq<T> { todo!() }
    pub fn join(&mut self, sep: T, other: BTreeSeq<T>) { todo!() }

    pub fn get(&self, i: usize) -> Option<&T> { todo!() }
    pub fn get_mut(&mut self, i: usize) -> Option<&mut T> { todo!() }

    pub fn into_iter(self) -> IntoIter<T> { todo!() }
    pub fn iter(&self) -> Iter<'_, T> { todo!() }
    pub fn iter_mut(&mut self) -> IterMut<'_, T> { todo!() }

    pub fn range(&self, r: impl RangeBounds<usize>) -> Range<'_, T> { todo!() }
    pub fn range_mut(&mut self, r: impl RangeBounds<usize>) -> RangeMut<'_, T> {
        todo!()
    }

    pub fn bisect(&self, pred: impl FnMut(&T) -> bool) -> usize { todo!() }
    pub fn rotate(&mut self, new_first: usize) {
        let mut tmp = self.split_off(new_first);
        tmp.append(std::mem::take(self));
        *self = tmp;
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
        left: Option<NonNull<Self>>,
        mid: T,
        right: Option<NonNull<Self>>,
    ) -> NonNull<Self> {
        todo!()
    }

    fn split(
        root: NonNull<Self>,
        at: NonNull<LeafNode<T>>,
    ) -> (Option<NonNull<Self>>, Option<NonNull<Self>>) {
        todo!()
    }

    fn select(root: NonNull<Self>, i: usize) -> Option<Todo> { todo!() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singleton() {
        let a = BTreeSeq::singleton(());
        assert_eq!(a.len(), 1);
    }
}
