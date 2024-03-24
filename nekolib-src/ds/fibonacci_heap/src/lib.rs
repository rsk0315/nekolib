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

impl<T: Ord> FibonacciHeap<T> {
    pub fn new() -> Self { Self { len: 0, max: None, ends: None } }

    pub fn push(&mut self, elt: T) -> Handle<T> {
        self.len += 1;
        let new = Node::new(elt);
        self.push_root(RootNode::new(new));
        Handle::new(new)
    }

    pub fn pop(&mut self) -> Option<T> {
        let root = self.max?;
        if let Some(child) =
            unsafe { (*(*root.as_ptr()).handle.node.as_ptr()).any_child.take() }
        {
            loop {
                let next = unsafe { (*child.node.as_ptr()).neighbor.1 };
                child.init_siblings();
                self.push_root(RootNode::new(child.node));
                if next.eq(child) {
                    break;
                }
            }
        }
        // if root has no child, `self.max.is_none() && self.ends.is_some()`
        // may hold at this point.
        self.coalesce();
        Some(RootNode::take(root))
    }

    pub fn meld(&mut self, other: Self) {
        self.len += other.len;
        if let Some((other_first, other_last)) = other.ends {
            if let Some((first, last)) = self.ends {
                RootNode::append(last, other_first);
                self.ends = Some((first, other_last));
            } else {
                self.ends = other.ends;
            }
        }
        if let Some(other_max) = other.max {
            self.push_root(other_max);
        }
    }

    fn push_root(&mut self, new: NonNull<RootNode<T>>) {
        if let Some(max) = self.max {
            RootNode::challenge(max, new);
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

        if let Some(max) = self.max.take() {
            bucket.push(max);
        }
        if let Some((first, _)) = self.ends.take() {
            let mut root = Some(first);
            while let Some(cur) = root {
                unsafe {
                    let ptr = cur.as_ptr();
                    RootNode { handle: (*ptr).handle, next_root: None };
                    root = (*ptr).next_root;
                }
                bucket.push(cur);
            }
        }

        for root in bucket.take() {
            self.push_root(root);
        }
    }
}

impl<T: Ord> RootNode<T> {
    pub fn new(node: NonNull<Node<T>>) -> NonNull<Self> {
        let root = Self { handle: Handle::new(node), next_root: None };
        NonNull::from(Box::leak(Box::new(root)))
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
        let node = unsafe { (*this.as_ptr()).handle.node };
        unsafe { &(*node.as_ptr()).val }
    }
    fn order(this: NonNull<Self>) -> usize {
        let node = unsafe { (*this.as_ptr()).handle.node };
        unsafe { (*node.as_ptr()).order }
    }
    fn is_isolated_root(this: NonNull<Self>) -> bool {
        unsafe {
            (*this.as_ptr()).next_root.is_none()
                && (*(*this.as_ptr()).handle.node.as_ptr()).parent.is_none()
            // && neighbor.0 == this
        }
    }

    fn fuse(par: NonNull<Self>, child: NonNull<Self>) {
        Self::precheck_fuse(par, child);
        unsafe {
            let par = (*par.as_ptr()).handle;
            let child = (*child.as_ptr()).handle;
            par.insert_child(child);
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

impl<T: Ord> Node<T> {
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

impl<T: Ord> Handle<T> {
    pub fn urge(self, new: T) -> bool { todo!() }

    fn new(node: NonNull<Node<T>>) -> Self { Self { node } }
    fn dangling() -> Self { Self { node: NonNull::dangling() } }

    fn eq(self, other: Self) -> bool { self.node == other.node }

    fn insert_child(self, child: Self) {
        let par = self.node.as_ptr();
        unsafe {
            if let Some(old_child) = (*par).any_child {
                old_child.insert_sibling(child);
            } else {
                child.init_siblings();
                (*par).any_child = Some(child);
            }
        }
    }
    fn init_siblings(self) {
        let ptr = self.node;
        let handle = Handle { node: ptr };
        unsafe { (*ptr.as_ptr()).neighbor = (handle, handle) };
    }
    fn insert_sibling(self, sibling: Self) {
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
}

impl<T> Copy for Handle<T> {}
impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self { *self }
}

impl<T: Ord> Bucket<T> {
    pub fn new() -> Self { Self { bucket: vec![] } }
    pub fn push(&mut self, root: NonNull<RootNode<T>>) {
        let order = RootNode::order(root);
        if order >= self.bucket.len() {
            self.bucket.resize(order + 1, None)
        }
        if let Some(old) = self.bucket[order].take() {
            RootNode::fuse(root, old);
            self.push(root);
        } else {
            self.bucket[order] = Some(root);
        }
    }
    pub fn take(self) -> impl Iterator<Item = NonNull<RootNode<T>>> {
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
    let mut a = NonNull::from(Box::leak(Box::new(A(NonNull::dangling()))));
    let mut b = NonNull::from(Box::leak(Box::new(A(NonNull::dangling()))));
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
