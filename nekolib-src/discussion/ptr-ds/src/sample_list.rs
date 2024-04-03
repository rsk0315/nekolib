use std::ptr::NonNull;

struct ListNode {
    val: i32,
    next: Option<NonNull<ListNode>>,
}

pub struct List {
    first_last: Option<(NonNull<ListNode>, NonNull<ListNode>)>,
}

impl ListNode {
    fn new(val: i32) -> NonNull<Self> {
        NonNull::from(Box::leak(Box::new(Self { val, next: None })))
    }
}

impl List {
    pub fn new() -> Self { Self { first_last: None } }
    pub fn push(&mut self, elt: i32) {
        let node = ListNode::new(elt);
        if let Some((_, last)) = &mut self.first_last {
            unsafe { (*last.as_ptr()).next = Some(node) };
            *last = node;
        } else {
            self.first_last = Some((node, node));
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = i32> + '_ {
        std::iter::successors(
            self.first_last.map(|(first, _)| first),
            |node| unsafe { (*node.as_ptr()).next },
        )
        .map(|node| unsafe { (*node.as_ptr()).val })
    }
}

impl Drop for List {
    fn drop(&mut self) {
        if let Some((first, _)) = self.first_last {
            let mut next = Some(first);
            while let Some(node) = next {
                next = unsafe { (*node.as_ptr()).next };
                unsafe { drop(Box::from_raw(node.as_ptr())) };
            }
        }
    }
}

#[test]
fn sanity_check() {
    let mut list = List::new();
    assert!(list.iter().eq([]));

    list.push(1);
    list.push(2);
    assert!(list.iter().eq([1, 2]));
}
