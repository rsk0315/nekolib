use std::marker::PhantomData;

use super::BTreeMap;
use crate::{
    borrow::DormantMutRef,
    node::{marker, Handle, NodeRef},
};

pub enum Entry<'a, K: 'a, V: 'a> {
    Vacant(VacantEntry<'a, K, V>),
    Occupied(OccupiedEntry<'a, K, V>),
}

pub struct VacantEntry<'a, K, V> {
    pub(super) key: K,
    pub(super) handle: Option<
        Handle<NodeRef<marker::Mut<'a>, K, V, marker::Leaf>, marker::Edge>,
    >,
    pub(super) dormant_map: DormantMutRef<'a, BTreeMap<K, V>>,
    pub(super) _marker: PhantomData<&'a mut (K, V)>,
}

pub struct OccupiedEntry<'a, K, V> {
    pub(super) handle: Handle<
        NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal>,
        marker::KV,
    >,
    pub(super) dormant_map: DormantMutRef<'a, BTreeMap<K, V>>,
    pub(super) _marker: PhantomData<&'a mut (K, V)>,
}
