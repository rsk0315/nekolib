use std::ptr::NonNull;

pub struct FibonacciHeap<T> {
    len: usize,
    max: Option<NonNull<RootNode<T>>>,
    ends: Option<(NonNull<RootNode<T>>, NonNull<RootNode<T>>)>,
}

struct RootNode<T> {
    handle: Handle<T>,
    next_root: Option<NonNull<RootNode<T>>>,
}

struct Node<T> {
    val: T,
    parent: Option<Handle<T>>,
    // (neighbor.0 == this) implies |sibling| == 1
    // (neighbor.0 == neighbor.1) implies |sibling| <= 2
    neighbor: (Handle<T>, Handle<T>),
    any_child: Option<Handle<T>>,
    order: usize,
    cut: bool,
}

pub struct Handle<T> {
    node: NonNull<Node<T>>,
}

struct Bucket<T> {
    bucket: Vec<Option<NonNull<RootNode<T>>>>,
}

impl<T: Ord + std::fmt::Debug> FibonacciHeap<T> {
    pub fn new() -> Self { Self { len: 0, max: None, ends: None } }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }

    pub fn push(&mut self, elt: T) -> Handle<T>
    where
        T: std::fmt::Debug,
    {
        self.len += 1;
        let new = Node::new(elt);
        self.push_root(RootNode::new(new));
        Handle::new(new)
    }

    pub fn pop(&mut self) -> Option<T> {
        eprintln!("-- pop() starts --");
        let root = self.max.take()?;
        eprintln!("root order: {}", RootNode::order(root));
        self.len -= 1;
        while let Some(child) = unsafe { (*root.as_ptr()).handle.pop_child() } {
            let order = unsafe { (*child.node.as_ptr()).order };
            // eprintln!("new root of order {order}");
            self.push_root(RootNode::new(child.node));
        }
        // if `root` has no child, `self.max.is_none() && self.ends.is_some()`
        // may hold at this point.
        self.coalesce();
        let res = RootNode::take(root);
        eprintln!("-- pop() returns {res:?} --");
        Some(res)
        // Some(RootNode::take(root))
    }

    pub fn meld(&mut self, mut other: Self) {
        self.len += other.len;
        if let Some((other_first, other_last)) = other.ends.take() {
            if let Some((first, last)) = self.ends {
                RootNode::append(last, other_first);
                self.ends = Some((first, other_last));
            } else {
                self.ends = other.ends;
            }
        }
        if let Some(other_max) = other.max.take() {
            self.push_root(other_max);
        }
    }

    fn push_root(&mut self, new: NonNull<RootNode<T>>) {
        if let Some(old) = self.max {
            RootNode::challenge(old, new);
            if let Some((first, last)) = self.ends {
                RootNode::append(last, new);
                self.ends = Some((first, new));
            } else {
                self.ends = Some((new, new));
            }
        } else {
            self.max = Some(new);
        }
    }

    fn coalesce(&mut self) {
        let mut bucket = Bucket::new();
        eprintln!("coalesce()");

        if let Some(max) = self.max.take() {
            RootNode::isolate(max);
            bucket.push(max);
        }
        if let Some((first, _)) = self.ends.take() {
            let mut root = Some(first);
            while let Some(cur) = root {
                unsafe {
                    let ptr = cur.as_ptr();
                    root = (*ptr).next_root;
                }
                RootNode::isolate(cur);
                bucket.push(cur);
            }
        }

        for root in bucket.take() {
            eprintln!("root: {root:?}");
            self.push_root(root);
        }
    }
}

impl<T> Drop for FibonacciHeap<T> {
    fn drop(&mut self) {
        if let Some(max) = self.max.take() {
            RootNode::drop(max);
        }
        if let Some((first, _)) = self.ends.take() {
            let mut cur = Some(first);
            while let Some(root) = cur {
                cur = unsafe { (*root.as_ptr()).next_root };
                RootNode::drop(root);
            }
        }
    }
}

impl<T: Ord + std::fmt::Debug> RootNode<T> {
    pub fn new(node: NonNull<Node<T>>) -> NonNull<Self> {
        let root = Self { handle: Handle::new(node), next_root: None };
        let res = NonNull::from(Box::leak(Box::new(root)));
        eprintln!("new root: addr: {res:?}, value: {:?}", Self::val(res));
        res
    }

    pub fn append(fst: NonNull<Self>, snd: NonNull<Self>) {
        unsafe {
            debug_assert!((*fst.as_ptr()).next_root.is_none());
            (*fst.as_ptr()).next_root = Some(snd);
        }
    }

    pub fn challenge(old: NonNull<Self>, new: NonNull<Self>) {
        if Self::val(old) < Self::val(new) {
            unsafe {
                std::mem::swap(
                    &mut (*old.as_ptr()).handle,
                    &mut (*new.as_ptr()).handle,
                )
            };
        }
    }

    fn val<'a>(this: NonNull<Self>) -> &'a T {
        unsafe { &(*(*this.as_ptr()).handle.node.as_ptr()).val }
    }
    fn order(this: NonNull<Self>) -> usize {
        unsafe { (*(*this.as_ptr()).handle.node.as_ptr()).order }
    }
    fn is_isolated_root(this: NonNull<Self>) -> bool {
        let ptr = this.as_ptr();

        unsafe {
            (*ptr).next_root.is_none()
                && (*(*ptr).handle.node.as_ptr()).parent.is_none()
                && (*ptr).handle.eq((*(*ptr).handle.node.as_ptr()).neighbor.0)
        }
    }

    fn isolate(this: NonNull<Self>) {
        let ptr = this.as_ptr();
        unsafe {
            (*ptr).next_root.take();
            (*ptr).handle.init_siblings();
        }
    }

    fn fuse(par: NonNull<Self>, child: NonNull<Self>) -> NonNull<Self> {
        Self::precheck_fuse(par, child);
        let (greater, less) = if Self::val(par) > Self::val(child) {
            (par, child)
        } else {
            (child, par)
        };
        unsafe {
            let g = Self::val(greater);
            let l = Self::val(less);
            eprintln!(
                "node {greater:?} ({g:?}) is now parent of node {less:?} ({l:?})"
            );
            (*greater.as_ptr()).handle.push_child((*less.as_ptr()).handle);
            drop(Box::from_raw(less.as_ptr()));
            greater
        }
    }

    fn precheck_fuse(par: NonNull<Self>, child: NonNull<Self>) {
        debug_assert_eq!(Self::order(par), Self::order(child));
        debug_assert!(Self::is_isolated_root(par));
        debug_assert!(Self::is_isolated_root(child));
    }

    fn take(this: NonNull<Self>) -> T {
        let ptr = this.as_ptr();
        let node = unsafe { Box::from_raw((*ptr).handle.node.as_ptr()) };
        let res = node.val;
        unsafe { drop(Box::from_raw(ptr)) };
        res
    }
}

impl<T> RootNode<T> {
    fn drop(this: NonNull<Self>) {
        unsafe {
            let handle = (*this.as_ptr()).handle;
            Handle::drop(handle);
            eprintln!("drop root: addr {this:?}");
            drop(Box::from_raw(this.as_ptr()));
        }
    }
}

impl<T: Ord + std::fmt::Debug> Node<T> {
    pub fn new(elt: T) -> NonNull<Self> {
        let node = Self {
            val: elt,
            parent: None,
            neighbor: (Handle::dangling(), Handle::dangling()),
            any_child: None,
            order: 0,
            cut: false,
        };
        let ptr = NonNull::from(Box::leak(Box::new(node)));
        let this = Handle { node: ptr };
        this.init_siblings();
        ptr
    }
}

impl<T> Handle<T> {
    pub fn urge(self, new: T) -> bool { todo!() }

    fn new(node: NonNull<Node<T>>) -> Self { Self { node } }
    fn dangling() -> Self { Self { node: NonNull::dangling() } }

    fn eq(self, other: Self) -> bool { self.node == other.node }

    fn push_child(self, child: Self) {
        let par = self.node.as_ptr();
        unsafe {
            (*par).order += 1;
            if let Some(old_child) = (*par).any_child {
                old_child.push_sibling(child);
            } else {
                child.init_siblings();
                (*child.node.as_ptr()).parent = Some(self);
                (*par).any_child = Some(child);
            }
        }
    }
    fn pop_child(self) -> Option<Self> {
        let par = self.node.as_ptr();
        unsafe {
            let child = (*par).any_child.take()?;
            (*par).order -= 1;
            (*par).cut = true;
            (*child.node.as_ptr()).parent.take();
            let (prev, next) = (*child.node.as_ptr()).neighbor;
            if child.eq(prev) {
                // that is the last child; nothing is to be done.
            } else {
                // `next` may be equal to `prev`.
                (*par).any_child = Some(next);
                (*prev.node.as_ptr()).neighbor.1 = next;
                (*next.node.as_ptr()).neighbor.0 = prev;
            }
            child.init_siblings();
            Some(child)
        }
    }

    fn init_siblings(self) {
        let ptr = self.node;
        let handle = Handle { node: ptr };
        unsafe { (*ptr.as_ptr()).neighbor = (handle, handle) };
    }
    fn push_sibling(self, sibling: Self) {
        unsafe {
            let neighbor = (*self.node.as_ptr()).neighbor;
            if self.eq(neighbor.0) {
                // siblings = {self}
                (*sibling.node.as_ptr()).neighbor = (self, self);
                (*self.node.as_ptr()).neighbor = (sibling, sibling);
            } else {
                let next = neighbor.1;
                (*self.node.as_ptr()).neighbor.1 = sibling;
                (*next.node.as_ptr()).neighbor.0 = sibling;
                (*sibling.node.as_ptr()).neighbor = (self, next);
            }
        }
    }

    fn drop(self) {
        while let Some(child) = self.pop_child() {
            Self::drop(child);
        }
        unsafe { drop(Box::from_raw(self.node.as_ptr())) };
    }
}

impl<T> Copy for Handle<T> {}
impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self { *self }
}

impl<T: Ord + std::fmt::Debug> Bucket<T> {
    pub fn new() -> Self { Self { bucket: vec![] } }
    pub fn push(&mut self, root: NonNull<RootNode<T>>) {
        let order = RootNode::order(root);
        if order >= self.bucket.len() {
            self.bucket.resize(order + 1, None)
        }
        if let Some(old) = self.bucket[order].take() {
            self.push(RootNode::fuse(root, old));
        } else {
            eprintln!("push: order: {order}, val: {:?}", RootNode::val(root));
            self.bucket[order] = Some(root);
        }
    }
    pub fn take(self) -> impl Iterator<Item = NonNull<RootNode<T>>> {
        let a: Vec<_> = self
            .bucket
            .iter()
            .enumerate()
            .map(|(i, o)| o.is_some().then(|| 1 << i))
            .collect();
        eprintln!("{a:?}");
        self.bucket.into_iter().filter_map(std::convert::identity)
    }
}

#[test]
fn nonnull_eq() {
    use std::ptr::NonNull;

    #[allow(unused)]
    struct A(i32);

    let a1 = NonNull::from(Box::leak(Box::new(A(0))));
    let a2 = a1;
    let b = NonNull::from(Box::leak(Box::new(A(0))));
    assert_ne!(a1, b);
    assert_eq!(a1, a2);

    unsafe { *a1.as_ptr() = A(1) };
    assert_eq!(a1, a2);

    unsafe { *a2.as_ptr() = A(2) };
    assert_eq!(a1, a2);

    unsafe {
        drop(Box::from_raw(a1.as_ptr()));
        drop(Box::from_raw(b.as_ptr()));
    }
}

#[test]
fn memleak() {
    use std::ptr::NonNull;

    #[allow(unused)]
    struct A(i32);

    let a = NonNull::from(Box::leak(Box::new(A(0))));
    let a_box = unsafe { Box::from_raw(a.as_ptr()) };
    assert_eq!(a_box.0, 0);
}

#[test]
fn nested_leak() {
    use std::ptr::NonNull;

    struct A(i32);
    #[allow(unused)]
    struct B(NonNull<A>);
    let a = NonNull::from(Box::leak(Box::new(A(100))));
    let b = NonNull::from(Box::leak(Box::new(B(a))));
    let a0 = unsafe {
        drop(Box::from_raw(b.as_ptr()));
        Box::from_raw(a.as_ptr()).0
    };
    assert_eq!(a0, 100);
}

#[test]
fn mutual_ref() {
    use std::ptr::NonNull;

    struct A(NonNull<A>);
    let a = NonNull::from(Box::leak(Box::new(A(NonNull::dangling()))));
    let b = NonNull::from(Box::leak(Box::new(A(NonNull::dangling()))));
    unsafe {
        (*a.as_ptr()).0 = b;
        (*b.as_ptr()).0 = a;
        (*a.as_ptr()).0 = b;
    };

    unsafe {
        drop(Box::from_raw(a.as_ptr()));
        drop(Box::from_raw(b.as_ptr()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single() {
        let mut q = FibonacciHeap::new();
        q.push(1);
        q.push(2);
        q.push(3);
        q.pop();
    }

    #[test]
    fn push_meld_pop() {
        let mut q = FibonacciHeap::new();
        assert!(q.is_empty());
        q.push(1);
        q.push(3);
        q.push(4);
        assert_eq!(q.len(), 3);

        let mut r = FibonacciHeap::new();
        r.push(0);
        r.push(2);
        q.meld(r);
        assert_eq!(q.len(), 5);

        let mut actual = vec![];
        for i in (0..5).rev() {
            actual.extend(q.pop());
            // assert_eq!(q.len(), i);
        }
        assert_eq!(actual, [4, 3, 2, 1, 0]);
    }

    #[test]
    fn many_pushes() {
        let mut q = FibonacciHeap::new();
        let mut r = FibonacciHeap::new();
        for i in 0..100 {
            q.push(i);
            r.push(i);
            assert_eq!(q.len(), i + 1);
        }
        let mut res = vec![];
        for i in (0..100).rev() {
            res.extend(q.pop());
            assert_eq!(q.len(), i);
        }
        eprintln!("{res:?}");
    }
}
