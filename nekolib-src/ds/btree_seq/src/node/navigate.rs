use super::{marker, Handle, NodeRef};

pub struct LeafRange<BorrowType, T, R> {
    front:
        Option<Handle<NodeRef<BorrowType, T, R, marker::Leaf>, marker::Edge>>,
    back: Option<Handle<NodeRef<BorrowType, T, R, marker::Leaf>, marker::Edge>>,
}
