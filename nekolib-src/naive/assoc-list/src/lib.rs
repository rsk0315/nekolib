#![allow(dead_code)]

use std::borrow::Borrow;

pub struct AssocList<K, V>(Vec<(K, V)>);
pub enum Entry<'a, K, V> {
    Vacant(VacantEntry<'a, K, V>),
    Occupied(OccupiedEntry<'a, K, V>),
}
pub struct VacantEntry<'a, K, V> {
    key: K,
    map: &'a mut AssocList<K, V>,
}
pub struct OccupiedEntry<'a, K, V> {
    key: K,
    map: &'a mut AssocList<K, V>,
}

impl<K: Eq, V> AssocList<K, V> {
    pub fn new() -> Self { Self(vec![]) }

    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    pub fn len(&self) -> usize { self.0.len() }

    pub fn get<Q>(&self, key: Q) -> Option<&V>
    where
        Q: Borrow<K>,
        K: PartialEq<Q>,
    {
        self.0.iter().find(|(k, _)| k == &key).map(|(_, v)| v)
    }

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

    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        if self.0.iter().any(|(k, _)| k == &key) {
            Entry::occupied(key, self)
        } else {
            Entry::vacant(key, self)
        }
    }
}

impl<K: Eq, V> Default for AssocList<K, V> {
    fn default() -> Self { Self::new() }
}

impl<'a, K: Eq, V> Entry<'a, K, V> {
    pub(crate) fn vacant(key: K, map: &'a mut AssocList<K, V>) -> Self {
        Self::Vacant(VacantEntry { key, map })
    }
    pub(crate) fn occupied(key: K, map: &'a mut AssocList<K, V>) -> Self {
        Self::Occupied(OccupiedEntry { key, map })
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

impl<'a, K: Eq, V> VacantEntry<'a, K, V> {
    pub fn into_key(self) -> K { self.key }
    pub fn key(&self) -> &K { &self.key }
    pub fn insert(self, value: V) -> &'a mut V {
        let Self { key, map } = self;
        map.0.push((key, value));
        &mut map.0.last_mut().unwrap().1
    }
}

impl<'a, K: Eq, V> OccupiedEntry<'a, K, V> {
    pub fn insert(&mut self, value: V) -> V {
        std::mem::replace(self.get_mut(), value)
    }
    pub fn get(&self) -> &V {
        let Self { key, map } = self;
        map.0.iter().find(|(k, _)| k == key).map(|(_, v)| v).unwrap()
    }
    pub fn get_mut(&mut self) -> &mut V {
        let Self { key, map } = self;
        map.0.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v).unwrap()
    }
    pub fn into_mut(self) -> &'a mut V {
        let Self { key, map } = self;
        map.0.iter_mut().find(|(k, _)| k == &key).map(|(_, v)| v).unwrap()
    }
    pub fn key(&self) -> &K { &self.key }
    pub fn remove(self) -> V { self.remove_entry().1 }
    pub fn remove_entry(self) -> (K, V) {
        let Self { key, map } = self;
        let i = (0..map.len()).find(|&i| key == map.0[i].0).unwrap();
        map.0.remove(i)
    }
}

#[test]
fn sanity_check() {
    let mut alist = AssocList::new();

    assert_eq!(alist.entry(0).key(), &0);

    alist.entry(0).or_insert("zero");
    assert_eq!(alist.get(0).unwrap(), &"zero");
    assert_eq!(alist.len(), 1);

    alist.entry(0).or_insert_with(|| "xxx");
    assert_eq!(alist.get(0).unwrap(), &"zero");
    assert_eq!(alist.len(), 1);

    alist.entry(2).or_insert_with_key(|_| "two");
    assert!(alist.get(1).is_none());
    assert_eq!(alist.get(2).unwrap(), &"two");
    assert_eq!(alist.len(), 2);

    alist.entry(2).and_modify(|v| *v = "second");
    assert_eq!(alist.len(), 2);

    if let Entry::Occupied(o) = alist.entry(2) {
        assert_eq!(o.get(), &"second");
        assert_eq!(o.remove(), "second");
        assert_eq!(alist.len(), 1);
    }

    alist.entry(1).or_default();
    assert_eq!(alist.len(), 2);
    assert!(alist.get(1).unwrap().is_empty());
    if let Entry::Occupied(mut o) = alist.entry(1) {
        o.insert("first");
        assert_eq!(o.get(), &"first");
    }
}
