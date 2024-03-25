use std::ptr::NonNull;

type RootLink<T> = NonNull<RootNode<T>>;
type Link<T> = NonNull<Node<T>>;

pub struct FibonacciHeap<T> {
    len: usize,
    max: Option<RootLink<T>>,
    ends: Option<(RootLink<T>, RootLink<T>)>,
}

struct RootNode<T> {
    root: Link<T>,
    next_root: Option<RootLink<T>>,
}

struct Node<T> {
    val: T,
    parent: Option<Link<T>>,
    // (neighbor.0 == this) implies |sibling| == 1
    // (neighbor.0 == neighbor.1) implies |sibling| <= 2
    neighbor: (Link<T>, Link<T>),
    any_child: Option<Link<T>>,
    order: usize,
    cut: bool,
    root: Option<RootLink<T>>,
}

pub struct Handle<T> {
    node: Link<T>,
}

struct Bucket<T> {
    bucket: Vec<Option<RootLink<T>>>,
}

impl<T: Ord> FibonacciHeap<T> {
    pub fn new() -> Self { Self { len: 0, max: None, ends: None } }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }

    pub fn push(&mut self, elt: T) -> Handle<T> {
        self.len += 1;
        let new = Node::new(elt);
        self.push_root(RootNode::new(new));
        Handle::new(new)
    }

    pub fn pop(&mut self) -> Option<T> {
        let root = self.max.take()?;
        self.len -= 1;
        while let Some(child) =
            unsafe { Node::pop_child((*root.as_ptr()).root) }
        {
            // we clean the parent-pointer of child later.
            self.push_root(RootNode::new(child));
        }
        // if `root` has no child, `self.max.is_none() && self.ends.is_some()`
        // may hold at this point.
        self.coalesce();
        Some(RootNode::take(root))
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

    pub fn urge(&mut self, handle: Handle<T>, new: T) -> bool {
        let node = handle.node;
        unsafe {
            if (*node.as_ptr()).val >= new {
                return false;
            }

            (*node.as_ptr()).val = new;
            if !Node::is_heapified(node) {
                let mut node = Some(node);
                while let Some(cur) = node {
                    node = Node::orphan(cur);
                    self.push_root(RootNode::new(cur));
                }
            }

            if let (Some(old), Some(new)) = (self.max, (*node.as_ptr()).root) {
                RootNode::challenge(old, new);
            }
        }
        true
    }

    fn push_root(&mut self, new: RootLink<T>) {
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

impl<T> RootNode<T> {
    pub fn new(node: Link<T>) -> RootLink<T> {
        let root = Self { root: node, next_root: None };
        let root = NonNull::from(Box::leak(Box::new(root)));
        unsafe { (*node.as_ptr()).root = Some(root) };
        root
    }

    pub fn append(fst: RootLink<T>, snd: RootLink<T>) {
        unsafe {
            debug_assert!((*fst.as_ptr()).next_root.is_none());
            (*fst.as_ptr()).next_root = Some(snd);
        }
    }

    pub fn challenge(old: RootLink<T>, new: RootLink<T>)
    where
        T: Ord,
    {
        if old == new {
            return;
        }
        if Self::val(old) < Self::val(new) {
            let old_ptr = old.as_ptr();
            let new_ptr = new.as_ptr();
            unsafe {
                debug_assert!((*(*old_ptr).root.as_ptr()).parent.is_none());
                debug_assert!((*(*new_ptr).root.as_ptr()).parent.is_none());
                std::mem::swap(
                    &mut (*(*old_ptr).root.as_ptr()).root,
                    &mut (*(*new_ptr).root.as_ptr()).root,
                );
                std::mem::swap(&mut (*old_ptr).root, &mut (*new_ptr).root);
            }
        }
    }

    fn val<'a>(this: RootLink<T>) -> &'a T {
        unsafe { &(*(*this.as_ptr()).root.as_ptr()).val }
    }
    fn order(this: RootLink<T>) -> usize {
        unsafe { (*(*this.as_ptr()).root.as_ptr()).order }
    }
    fn is_isolated_root(this: RootLink<T>) -> bool {
        let ptr = this.as_ptr();
        unsafe {
            (*ptr).next_root.is_none()
                && (*(*ptr).root.as_ptr()).parent.is_none()
                && (*ptr).root == (*(*ptr).root.as_ptr()).neighbor.0
        }
    }

    fn isolate(this: RootLink<T>) {
        let ptr = this.as_ptr();
        unsafe {
            (*ptr).next_root.take();
            (*(*ptr).root.as_ptr()).parent.take();
            Node::init_siblings((*ptr).root);
        }
    }

    fn fuse(par: RootLink<T>, child: RootLink<T>) -> RootLink<T>
    where
        T: Ord,
    {
        Self::precheck_fuse(par, child);
        let (greater, less) = if Self::val(par) > Self::val(child) {
            (par, child)
        } else {
            (child, par)
        };
        unsafe {
            Node::push_child((*greater.as_ptr()).root, (*less.as_ptr()).root);
            drop(Box::from_raw(less.as_ptr()));
            greater
        }
    }

    fn precheck_fuse(par: RootLink<T>, child: RootLink<T>) {
        debug_assert_eq!(Self::order(par), Self::order(child));
        debug_assert!(Self::is_isolated_root(par));
        debug_assert!(Self::is_isolated_root(child));
    }

    fn take(this: RootLink<T>) -> T {
        let ptr = this.as_ptr();
        let node = unsafe { Box::from_raw((*ptr).root.as_ptr()) };
        let res = node.val;
        unsafe { drop(Box::from_raw(ptr)) };
        res
    }
}

impl<T> RootNode<T> {
    fn drop(this: RootLink<T>) {
        unsafe {
            Node::drop((*this.as_ptr()).root);
            drop(Box::from_raw(this.as_ptr()));
        }
    }
}

impl<T> Node<T> {
    pub fn new(elt: T) -> Link<T> {
        let node = Self {
            val: elt,
            parent: None,
            neighbor: (NonNull::dangling(), NonNull::dangling()),
            any_child: None,
            order: 0,
            cut: false,
            root: None,
        };
        let ptr = NonNull::from(Box::leak(Box::new(node)));
        Node::init_siblings(ptr);
        ptr
    }

    pub fn push_child(this: Link<T>, child: Link<T>) {
        debug_assert!(Node::is_root(this));
        let par = this.as_ptr();
        unsafe {
            (*par).order += 1;
            if let Some(old_child) = (*par).any_child {
                Node::push_sibling(old_child, child);
            } else {
                Node::init_siblings(child);
                (*par).any_child = Some(child);
            }
            (*child.as_ptr()).parent = Some(this);
            (*child.as_ptr()).root.take();
        }
    }

    pub fn pop_child(this: Link<T>) -> Option<Link<T>> {
        let par = this.as_ptr();
        unsafe {
            let child = (*par).any_child.take()?;
            (*par).order -= 1;
            if (*par).parent.is_some() {
                // root nodes does not use this flag.
                (*par).cut = true;
            }
            (*child.as_ptr()).parent.take();
            let (prev, next) = (*child.as_ptr()).neighbor;
            if child == prev {
                // that is the last child; nothing is to be done.
            } else {
                // `next` may be equal to `prev`, but no special care is needed.
                (*par).any_child = Some(next);
                (*prev.as_ptr()).neighbor.1 = next;
                (*next.as_ptr()).neighbor.0 = prev;
            }
            Node::init_siblings(child);
            Some(child)
        }
    }

    pub fn init_siblings(this: Link<T>) {
        unsafe { (*this.as_ptr()).neighbor = (this, this) };
    }
    pub fn push_sibling(old: Link<T>, new: Link<T>) {
        unsafe {
            let neighbor = (*old.as_ptr()).neighbor;
            if old == neighbor.0 {
                // siblings == {old}
                (*new.as_ptr()).neighbor = (old, old);
                (*old.as_ptr()).neighbor = (new, new);
            } else {
                let next = neighbor.1;
                (*old.as_ptr()).neighbor.1 = new;
                (*next.as_ptr()).neighbor.0 = new;
                (*new.as_ptr()).neighbor = (old, next);
            }
        }
    }

    fn is_root(this: Link<T>) -> bool {
        unsafe {
            debug_assert_eq!(
                (*this.as_ptr()).root.is_some(),
                (*this.as_ptr()).parent.is_none(),
            );
            (*this.as_ptr()).parent.is_none()
        }
    }

    pub fn orphan(this: Link<T>) -> Option<Link<T>> {
        let ptr = this.as_ptr();
        unsafe {
            (*ptr).cut = false;
            let par = (*ptr).parent.take()?;

            let (prev, next) = (*ptr).neighbor;
            if this == prev {
                // the parent has no child now.
                (*par.as_ptr()).any_child.take();
            } else {
                (*prev.as_ptr()).neighbor.1 = next;
                (*next.as_ptr()).neighbor.0 = prev;
                if this == (*par.as_ptr()).any_child.unwrap() {
                    // `next` is now the representative child.
                    (*par.as_ptr()).any_child = Some(next);
                }
            }

            (*par.as_ptr()).order -= 1;
            if Node::is_root(par) {
                None
            } else if (*par.as_ptr()).cut {
                // cascading orphan is occurred.
                Some(par)
            } else {
                (*par.as_ptr()).cut = true;
                None
            }
        }
    }

    fn is_heapified(child: Link<T>) -> bool
    where
        T: Ord,
    {
        unsafe {
            match (*child.as_ptr()).parent {
                Some(par) => (*par.as_ptr()).val >= (*child.as_ptr()).val,
                _ => true,
            }
        }
    }
}

impl<T> Node<T> {
    fn drop(node: Link<T>) {
        while let Some(child) = Node::pop_child(node) {
            Self::drop(child);
        }
        unsafe { drop(Box::from_raw(node.as_ptr())) };
    }
}

impl<T> Handle<T> {
    fn new(node: Link<T>) -> Self { Self { node } }
}

impl<T> Copy for Handle<T> {}
impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self { *self }
}

impl<T: Ord> Bucket<T> {
    pub fn new() -> Self { Self { bucket: vec![] } }
    pub fn push(&mut self, root: RootLink<T>) {
        let order = RootNode::order(root);
        if order >= self.bucket.len() {
            self.bucket.resize(order + 1, None)
        }
        if let Some(old) = self.bucket[order].take() {
            self.push(RootNode::fuse(root, old));
        } else {
            self.bucket[order] = Some(root);
        }
    }
    pub fn take(self) -> impl Iterator<Item = RootLink<T>> {
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
            assert_eq!(q.len(), i);
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
    }

    #[test]
    fn heap_sort() {
        let n = 1 << 8;
        let mut q = FibonacciHeap::new();
        for i in (0..n).map(|i| (i >> 1) ^ i) {
            q.push(i);
        }
        let actual = (0..n).map(|_| q.pop().unwrap());
        assert!(actual.eq((0..n).rev()));
    }

    #[test]
    fn urge() {
        let mut q = FibonacciHeap::new();
        q.push(1);
        let x2 = q.push(2);
        q.push(3);
        q.urge(x2, 20);
        let actual: Vec<_> = (0..q.len()).map(|_| q.pop().unwrap()).collect();
        assert_eq!(actual, [20, 3, 1]);

        let x2 = q.push(2);
        let x1 = q.push(1);
        let _x6 = q.push(6);
        let x3 = q.push(3);
        let x5 = q.push(5);
        let _x4 = q.push(4);
        let _x7 = q.push(7);
        assert_eq!(q.pop(), Some(7));

        q.urge(x5, 500);
        assert_eq!(q.pop(), Some(500));

        q.urge(x1, 4);
        assert_eq!(q.pop(), Some(6));

        q.urge(x2, 1);
        assert_eq!(q.pop(), Some(4));

        q.urge(x3, 5);
        assert_eq!(q.pop(), Some(5));

        q.urge(x2, 10);
        assert_eq!(q.pop(), Some(10));

        assert_eq!(q.pop(), Some(4));
        assert!(q.is_empty());
    }
}

// TODO: `heap.iter()` and `node.iter()` should be implemented.
