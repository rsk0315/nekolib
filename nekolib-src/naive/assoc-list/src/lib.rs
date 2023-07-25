#![allow(dead_code)]

use std::borrow::Borrow;
use std::marker::PhantomData;

use borrow::DormantMutRef;

pub struct AssocList<K, V>(Vec<(K, V)>);
pub enum Entry<'a, K: 'a, V: 'a> {
    Vacant(VacantEntry<'a, K, V>),
    Occupied(OccupiedEntry<'a, K, V>),
}
pub struct VacantEntry<'a, K, V> {
    key: K,
    dormant_map: DormantMutRef<'a, AssocList<K, V>>,
    _marker: PhantomData<&'a mut (K, V)>,
}
pub struct OccupiedEntry<'a, K, V> {
    item: &'a mut (K, V),
    dormant_map: DormantMutRef<'a, AssocList<K, V>>,
}

impl<K: Eq, V> AssocList<K, V> {
    pub fn new() -> Self { Self(vec![]) }

    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    pub fn len(&self) -> usize { self.0.len() }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if let Some((_, v)) = self.0.iter_mut().find(|(k, _)| k == &key) {
            Some(std::mem::replace(v, value))
        } else {
            self.0.push((key, value));
            None
        }
    }

    pub fn remove<Q>(&mut self, key: Q) -> Option<V>
    where
        Q: Borrow<K>,
        K: PartialEq<Q>,
    {
        (0..self.0.len())
            .find(|&i| self.0[i].0 == key)
            .map(|i| self.0.remove(i).1)
    }

    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        let (map, dormant_map) = DormantMutRef::new(self);
        if let Some(item) = map.0.iter_mut().find(|(k, _)| k == &key) {
            Entry::occupied(item, dormant_map)
        } else {
            Entry::vacant(key, dormant_map)
        }
    }
}

impl<K: Eq, V> Default for AssocList<K, V> {
    fn default() -> Self { Self::new() }
}

impl<'a, K: 'a + Eq, V: 'a> Entry<'a, K, V> {
    pub(crate) fn vacant(
        key: K,
        dormant_map: DormantMutRef<'a, AssocList<K, V>>,
    ) -> Self {
        Self::Vacant(VacantEntry { key, dormant_map, _marker: PhantomData })
    }
    pub(crate) fn occupied(
        item: &'a mut (K, V),
        dormant_map: DormantMutRef<'a, AssocList<K, V>>,
    ) -> Self {
        Self::Occupied(OccupiedEntry { item, dormant_map })
    }

    pub fn and_modify<F: FnOnce(&mut V)>(self, f: F) -> Entry<'a, K, V> {
        match self {
            Self::Occupied(mut entry) => {
                f(entry.get_mut());
                Self::Occupied(entry)
            }
            Self::Vacant(entry) => Self::Vacant(entry),
        }
    }

    pub fn key(&self) -> &K {
        match *self {
            Self::Occupied(ref entry) => entry.key(),
            Self::Vacant(ref entry) => entry.key(),
        }
    }
    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => entry.insert(Default::default()),
        }
    }
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => entry.insert(default),
        }
    }
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => entry.insert(default()),
        }
    }
    pub fn or_insert_with_key<F: FnOnce(&K) -> V>(
        self,
        default: F,
    ) -> &'a mut V {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => {
                let value = default(entry.key());
                entry.insert(value)
            }
        }
    }
}

impl<'a, K: 'a + Eq, V: 'a> VacantEntry<'a, K, V> {
    pub fn into_key(self) -> K { self.key }
    pub fn key(&self) -> &K { &self.key }

    pub fn insert(self, _value: V) -> &'a mut V { todo!() }
}

impl<'a, K: 'a + Eq, V: 'a> OccupiedEntry<'a, K, V> {
    pub fn insert(&mut self, value: V) -> V {
        std::mem::replace(self.get_mut(), value)
    }

    pub fn get(&self) -> &V { todo!() }
    pub fn get_mut(&mut self) -> &mut V { todo!() }
    pub fn into_mut(self) -> &'a mut V { todo!() }
    pub fn key(&self) -> &K { todo!() }
    pub fn remove(self) -> V { todo!() }
    pub fn remove_entry(self) -> (K, V) { todo!() }
}
