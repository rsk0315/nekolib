use super::{
    marker,
    slice::{move_to_slice, slice_insert},
    Handle, InternalNode, LeafNode, NodeRef, Root, B, CAPACITY,
};

pub(crate) struct SplitResult<'a, T, R, NodeType> {
    pub left: NodeRef<marker::Mut<'a>, T, R, NodeType>,
    pub value: T,
    pub right: NodeRef<marker::Owned, T, R, NodeType>,
}

impl<'a, T, R> SplitResult<'a, T, R, marker::Leaf> {
    pub fn forget_node_type(
        self,
    ) -> SplitResult<'a, T, R, marker::LeafOrInternal> {
        SplitResult {
            left: self.left.forget_type(),
            value: self.value,
            right: self.right.forget_type(),
        }
    }
}
impl<'a, T, R> SplitResult<'a, T, R, marker::Internal> {
    pub fn forget_node_type(
        self,
    ) -> SplitResult<'a, T, R, marker::LeafOrInternal> {
        SplitResult {
            left: self.left.forget_type(),
            value: self.value,
            right: self.right.forget_type(),
        }
    }
}

enum LeftOrRight<T> {
    Left(T),
    Right(T),
}

const KV_IDX_CENTER: usize = B - 1;
const EDGE_IDX_LEFT_OF_CENTER: usize = B - 1;
const EDGE_IDX_RIGHT_OF_CENTER: usize = B - 1;

fn splitpoint(edge_idx: usize) -> (usize, LeftOrRight<usize>) {
    debug_assert!(edge_idx <= CAPACITY);
    match edge_idx {
        EDGE_IDX_LEFT_OF_CENTER => {
            (KV_IDX_CENTER - 1, LeftOrRight::Left(edge_idx))
        }
        EDGE_IDX_RIGHT_OF_CENTER => (KV_IDX_CENTER, LeftOrRight::Right(0)),
        0..=EDGE_IDX_LEFT_OF_CENTER => {
            (KV_IDX_CENTER - 1, LeftOrRight::Left(edge_idx))
        }
        _ => (
            KV_IDX_CENTER + 1,
            LeftOrRight::Right(edge_idx - (KV_IDX_CENTER + 1 + 1)),
        ),
    }
}

impl<'a, T: 'a, R: 'a>
    Handle<NodeRef<marker::Mut<'a>, T, R, marker::Leaf>, marker::Edge>
{
    pub fn insert_recursing(
        self,
        value: T,
        split_root: impl FnOnce(SplitResult<'a, T, R, marker::LeafOrInternal>),
    ) -> Handle<NodeRef<marker::Mut<'a>, T, R, marker::Leaf>, marker::Value>
    {
        let (mut split, handle) = match self.insert(value) {
            (None, handle) => return unsafe { handle.awaken() },
            (Some(split), handle) => (split.forget_node_type(), handle),
        };
        loop {
            split = match split.left.ascend() {
                Ok(parent) => match parent.insert(split.value, split.right) {
                    None => return unsafe { handle.awaken() },
                    Some(split) => split.forget_node_type(),
                },
                Err(root) => {
                    split_root(SplitResult { left: root, ..split });
                    return unsafe { handle.awaken() };
                }
            };
        }
    }

    fn insert(
        self,
        value: T,
    ) -> (
        Option<SplitResult<'a, T, R, marker::Leaf>>,
        Handle<NodeRef<marker::DormantMut, T, R, marker::Leaf>, marker::Value>,
    ) {
        if self.node.len() < CAPACITY {
            let handle = unsafe { self.insert_fit(value) };
            (None, handle.dormant())
        } else {
            let (middle_kv_idx, insertion) = splitpoint(self.idx);
            let middle = unsafe { Handle::new_value(self.node, middle_kv_idx) };
            let mut result = middle.split();
            let insertion_edge = match insertion {
                LeftOrRight::Left(insert_idx) => unsafe {
                    Handle::new_edge(result.left.reborrow_mut(), insert_idx)
                },
                LeftOrRight::Right(insert_idx) => unsafe {
                    Handle::new_edge(result.right.borrow_mut(), insert_idx)
                },
            };
            let handle = unsafe { insertion_edge.insert_fit(value).dormant() };
            (Some(result), handle)
        }
    }

    unsafe fn insert_fit(
        mut self,
        value: T,
    ) -> Handle<NodeRef<marker::Mut<'a>, T, R, marker::Leaf>, marker::Value>
    {
        debug_assert!(self.node.len() < CAPACITY);
        let new_len = self.node.len() + 1;

        unsafe {
            slice_insert(self.node.val_area_mut(..new_len), self.idx, value);
            *self.node.len_mut() = new_len as _;
            Handle::new_value(self.node, self.idx)
        }
    }
}

impl<'a, T: 'a, R: 'a>
    Handle<NodeRef<marker::Mut<'a>, T, R, marker::Leaf>, marker::Value>
{
    pub fn split(mut self) -> SplitResult<'a, T, R, marker::Leaf> {
        let mut new_node = LeafNode::new();
        let value = self.split_leaf_data(&mut new_node);
        let right = NodeRef::from_new_leaf(new_node);
        SplitResult { left: self.node, value, right }
    }
}

impl<'a, T: 'a, R: 'a>
    Handle<NodeRef<marker::Mut<'a>, T, R, marker::Internal>, marker::Edge>
{
    fn insert(
        mut self,
        value: T,
        edge: Root<T, R>,
    ) -> Option<SplitResult<'a, T, R, marker::Internal>> {
        assert!(edge.height == self.node.height - 1);

        if self.node.len() < CAPACITY {
            unsafe { self.insert_fit(value, edge) };
            None
        } else {
            let (middle_kv_idx, insertion) = splitpoint(self.idx);
            let middle = unsafe { Handle::new_value(self.node, middle_kv_idx) };
            let mut result = middle.split();
            let mut insertion_edge = match insertion {
                LeftOrRight::Left(insert_idx) => unsafe {
                    Handle::new_edge(result.left.reborrow_mut(), insert_idx)
                },
                LeftOrRight::Right(insert_idx) => unsafe {
                    Handle::new_edge(result.right.borrow_mut(), insert_idx)
                },
            };
            unsafe { insertion_edge.insert_fit(value, edge) };
            Some(result)
        }
    }

    unsafe fn insert_fit(mut self, value: T, edge: Root<T, R>) {
        debug_assert!(self.node.len() < CAPACITY);
        debug_assert!(edge.height == self.node.height - 1);
        let new_len = self.node.len() + 1;
        unsafe {
            slice_insert(self.node.val_area_mut(..new_len), self.idx, value);
            slice_insert(
                self.node.edge_area_mut(..new_len + 1),
                self.idx + 1,
                edge.node,
            );
            *self.node.len_mut() = new_len as _;
            self.node.correct_childrens_parent_links(self.idx + 1..new_len + 1);
        }
    }
}

impl<'a, T: 'a, R: 'a>
    Handle<NodeRef<marker::Mut<'a>, T, R, marker::Internal>, marker::Value>
{
    pub fn split(mut self) -> SplitResult<'a, T, R, marker::Internal> {
        let old_len = self.node.len();
        unsafe {
            let mut new_node = InternalNode::new();
            let value = self.split_leaf_data(&mut new_node.data);
            let new_len = usize::from(new_node.data.len);
            move_to_slice(
                self.node.edge_area_mut(self.idx + 1..old_len + 1),
                &mut new_node.edges[..new_len + 1],
            );
            let height = self.node.height;
            let right = NodeRef::from_new_internal(new_node, height);
            SplitResult { left: self.node, value, right }
        }
    }
}

impl<'a, T: 'a, R: 'a, NodeType>
    Handle<NodeRef<marker::Mut<'a>, T, R, NodeType>, marker::Value>
{
    fn split_leaf_data(&mut self, new_node: &mut LeafNode<T, R>) -> T {
        debug_assert!(self.idx < self.node.len());
        let old_len = self.node.len();
        let new_len = old_len - self.idx - 1;
        new_node.len = new_len as _;
        unsafe {
            let value = self.node.val_area_mut(self.idx).assume_init_read();
            move_to_slice(
                self.node.val_area_mut(self.idx + 1..old_len),
                &mut new_node.vals[..new_len],
            );
            *self.node.len_mut() = self.idx as _;
            value
        }
    }
}
