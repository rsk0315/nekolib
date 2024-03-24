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

impl<T: Ord> FibonacciHeap<T> {
    pub fn new() -> Self { Self { len: 0, max: None, ends: None } }

    pub fn push(&mut self, elt: T) -> Handle<T> {
        self.len += 1;
        let new = Node::new(elt);
        self.push_root(RootNode::new(new));
        Handle::new(new)
    }

    pub fn pop(&mut self) -> Option<T> { todo!() }

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
        unsafe {
            (*ptr.as_ptr()).neighbor =
                (Handle { node: ptr }, Handle { node: ptr })
        };
    }
    fn insert_sibling(self, sibling: Self) {
        todo!();
    }
}

impl<T> Copy for Handle<T> {}
impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self { *self }
}
