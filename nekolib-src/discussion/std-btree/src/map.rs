use std::{
    borrow::Borrow,
    fmt,
    hash::Hash,
    iter::FusedIterator,
    marker::PhantomData,
    ops::{Index, RangeBounds},
    ptr,
};

use crate::{
    navigate::{LazyLeafRange, LeafRange},
    node::{self, marker, Handle, NodeRef, Root},
    set_val::SetValZST,
};

mod entry;

pub use entry::{Entry, OccupiedEntry, VacantEntry};

pub(super) const MIN_LEN: usize = node::MIN_LEN_AFTER_SPLIT;

pub struct BTreeMap<K, V> {
    root: Option<Root<K, V>>,
    length: usize,
    _marker: PhantomData<Box<(K, V)>>,
}

impl<K, V> Drop for BTreeMap<K, V> {
    fn drop(&mut self) { todo!() }
}

// BAD?
impl<K, V> core::panic::UnwindSafe for BTreeMap<K, V>
where
    K: core::panic::RefUnwindSafe,
    V: core::panic::RefUnwindSafe,
{
}

impl<K: Clone, V: Clone> Clone for BTreeMap<K, V> {
    fn clone(&self) -> BTreeMap<K, V> { todo!() }
}

impl<K, Q: ?Sized> super::Recover<Q> for BTreeMap<K, SetValZST>
where
    K: Borrow<Q> + Ord,
    Q: Ord,
{
    type Key = K;

    fn get(&self, key: &Q) -> Option<&Self::Key> { todo!() }
    fn take(&mut self, key: &Q) -> Option<Self::Key> { todo!() }
    fn replace(&mut self, key: Self::Key) -> Option<Self::Key> { todo!() }
}

pub struct Iter<'a, K: 'a, V: 'a> {
    range: LazyLeafRange<marker::Immut<'a>, K, V>,
    length: usize,
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for Iter<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

impl<'a, K: 'a, V: 'a> Default for Iter<'a, K, V> {
    fn default() -> Self { todo!() }
}

pub struct IterMut<'a, K: 'a, V: 'a> {
    range: LazyLeafRange<marker::ValMut<'a>, K, V>,
    length: usize,
    _marker: PhantomData<&'a mut (K, V)>,
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IterMut<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

impl<'a, K: 'a, V: 'a> Default for IterMut<'a, K, V> {
    fn default() -> Self { todo!() }
}

pub struct IntoIter<K, V> {
    range: LazyLeafRange<marker::Dying, K, V>,
    length: usize,
}

impl<K, V> IntoIter<K, V> {
    pub(super) fn iter(&self) -> Iter<'_, K, V> { todo!() }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IntoIter<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

impl<K, V> Default for IntoIter<K, V> {
    fn default() -> Self { todo!() }
}

pub struct Keys<'a, K, V> {
    inner: Iter<'a, K, V>,
}

impl<K: fmt::Debug, V> fmt::Debug for Keys<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

pub struct Values<'a, K, V> {
    inner: Iter<'a, K, V>,
}

impl<K, V: fmt::Debug> fmt::Debug for Values<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

pub struct ValuesMut<'a, K, V> {
    inner: IterMut<'a, K, V>,
}

impl<K, V: fmt::Debug> fmt::Debug for ValuesMut<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

pub struct IntoKeys<K, V> {
    inner: IntoIter<K, V>,
}

impl<K: fmt::Debug, V> fmt::Debug for IntoKeys<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

pub struct IntoValues<K, V> {
    inner: IntoIter<K, V>,
}

impl<K, V: fmt::Debug> fmt::Debug for IntoValues<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

pub struct Range<'a, K: 'a, V: 'a> {
    inner: LeafRange<marker::Immut<'a>, K, V>,
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for Range<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

pub struct RangeMut<'a, K: 'a, V: 'a> {
    inner: LeafRange<marker::ValMut<'a>, K, V>,
    _marker: PhantomData<&'a mut (K, V)>,
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for RangeMut<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

impl<K, V> BTreeMap<K, V> {
    pub const fn new() -> BTreeMap<K, V> { todo!() }

    pub fn clear(&mut self) { todo!() }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        todo!()
    }

    pub fn get_key_value<Q: ?Sized>(&self, k: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        todo!()
    }

    pub fn first_key_value(&self) -> Option<(&K, &V)>
    where
        K: Ord,
    {
        todo!()
    }

    pub fn first_entry(&mut self) -> Option<OccupiedEntry<'_, K, V>>
    where
        K: Ord,
    {
        todo!()
    }

    pub fn pop_first(&mut self) -> Option<(K, V)>
    where
        K: Ord,
    {
        todo!()
    }

    pub fn last_key_value(&self) -> Option<(&K, &V)>
    where
        K: Ord,
    {
        todo!()
    }

    pub fn last_entry(&mut self) -> Option<OccupiedEntry<'_, K, V>>
    where
        K: Ord,
    {
        todo!()
    }

    pub fn pop_last(&mut self) -> Option<(K, V)>
    where
        K: Ord,
    {
        todo!()
    }

    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        todo!()
    }

    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        todo!()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        todo!()
    }

    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        todo!()
    }

    pub fn remove_entry<Q: ?Sized>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        todo!()
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        K: Ord,
        F: FnMut(&K, &mut V) -> bool,
    {
        todo!()
    }

    pub fn append(&mut self, other: &mut Self)
    where
        K: Ord,
    {
        todo!()
    }

    pub fn range<T: ?Sized, R>(&self, range: R) -> Range<'_, K, V>
    where
        T: Ord,
        K: Borrow<T> + Ord,
        R: RangeBounds<T>,
    {
        todo!()
    }

    pub fn range_mut<T: ?Sized, R>(&self, range: R) -> RangeMut<'_, K, V>
    where
        T: Ord,
        K: Borrow<T> + Ord,
        R: RangeBounds<T>,
    {
        todo!()
    }

    pub fn entry(&mut self, key: K) -> Entry<'_, K, V>
    where
        K: Ord,
    {
        todo!()
    }

    pub fn split_off<Q: ?Sized + Ord>(&mut self, key: &Q) -> Self
    where
        K: Borrow<Q> + Ord,
    {
        todo!()
    }

    pub fn into_keys(self) -> IntoKeys<K, V> { todo!() }

    pub fn into_values(self) -> IntoValues<K, V> { todo!() }

    pub(crate) fn bulk_build_from_sorted_iter<I>(iter: I) -> Self
    where
        K: Ord,
        I: IntoIterator<Item = (K, V)>,
    {
        todo!()
    }
}

impl<'a, K, V> IntoIterator for &'a BTreeMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter { todo!() }
}

impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> { todo!() }
    fn size_hint(&self) -> (usize, Option<usize>) { todo!() }
    fn last(mut self) -> Option<Self::Item> { todo!() }
    fn min(mut self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        todo!()
    }
    fn max(mut self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        todo!()
    }
}

impl<K, V> FusedIterator for Iter<'_, K, V> {}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> { todo!() }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {
    fn len(&self) -> usize { todo!() }
}

impl<K, V> Clone for Iter<'_, K, V> {
    fn clone(&self) -> Self { todo!() }
}

impl<'a, K, V> IntoIterator for &'a mut BTreeMap<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter { todo!() }
}

impl<'a, K: 'a, V: 'a> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> { todo!() }
    fn size_hint(&self) -> (usize, Option<usize>) { todo!() }
    fn last(mut self) -> Option<Self::Item> { todo!() }
    fn min(mut self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        todo!()
    }
    fn max(mut self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        todo!()
    }
}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for IterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> { todo!() }
}

impl<K, V> ExactSizeIterator for IterMut<'_, K, V> {
    fn len(&self) -> usize { todo!() }
}

impl<K, V> FusedIterator for IterMut<'_, K, V> {}

impl<'a, K, V> IterMut<'a, K, V> {
    pub(super) fn iter(&self) -> Iter<'_, K, V> { todo!() }
}

impl<K, V> IntoIterator for BTreeMap<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter { todo!() }
}

impl<K, V> Drop for IntoIter<K, V> {
    fn drop(&mut self) { todo!() }
}

impl<K, V> IntoIter<K, V> {
    fn dying_next(
        &mut self,
    ) -> Option<
        Handle<
            NodeRef<marker::Dying, K, V, marker::LeafOrInternal>,
            marker::KV,
        >,
    > {
        todo!()
    }

    fn dying_next_back(
        &mut self,
    ) -> Option<
        Handle<
            NodeRef<marker::Dying, K, V, marker::LeafOrInternal>,
            marker::KV,
        >,
    > {
        todo!()
    }
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> { todo!() }

    fn size_hint(&self) -> (usize, Option<usize>) { todo!() }
}

impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> { todo!() }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {
    fn len(&self) -> usize { todo!() }
}

impl<K, V> FusedIterator for IntoIter<K, V> {}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> { todo!() }
    fn size_hint(&self) -> (usize, Option<usize>) { todo!() }
    fn last(mut self) -> Option<Self::Item> { todo!() }
    fn min(mut self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        todo!()
    }
    fn max(mut self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        todo!()
    }
}

impl<'a, K, V> DoubleEndedIterator for Keys<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> { todo!() }
}

impl<K, V> ExactSizeIterator for Keys<'_, K, V> {
    fn len(&self) -> usize { todo!() }
}

impl<K, V> FusedIterator for Keys<'_, K, V> {}

impl<K, V> Clone for Keys<'_, K, V> {
    fn clone(&self) -> Self { todo!() }
}

impl<K, V> Default for Keys<'_, K, V> {
    fn default() -> Self { todo!() }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> { todo!() }
    fn size_hint(&self) -> (usize, Option<usize>) { todo!() }
    fn last(mut self) -> Option<&'a V> { todo!() }
}

impl<'a, K, V> DoubleEndedIterator for Values<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> { todo!() }
}

impl<K, V> ExactSizeIterator for Values<'_, K, V> {
    fn len(&self) -> usize { todo!() }
}

impl<K, V> FusedIterator for Values<'_, K, V> {}

impl<K, V> Clone for Values<'_, K, V> {
    fn clone(&self) -> Self { todo!() }
}

impl<K, V> Default for Values<'_, K, V> {
    fn default() -> Self { todo!() }
}

impl<'a, K, V> Iterator for Range<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> { todo!() }
    fn last(mut self) -> Option<Self::Item> { todo!() }
    fn min(mut self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        todo!()
    }
    fn max(mut self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        todo!()
    }
}

impl<K, V> Default for Range<'_, K, V> {
    fn default() -> Self { todo!() }
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> { todo!() }
    fn size_hint(&self) -> (usize, Option<usize>) { todo!() }
    fn last(mut self) -> Option<Self::Item> { todo!() }
}

impl<'a, K, V> DoubleEndedIterator for ValuesMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> { todo!() }
}

impl<K, V> ExactSizeIterator for ValuesMut<'_, K, V> {
    fn len(&self) -> usize { todo!() }
}

impl<K, V> FusedIterator for ValuesMut<'_, K, V> {}

impl<K, V> Iterator for IntoKeys<K, V> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> { todo!() }
    fn size_hint(&self) -> (usize, Option<usize>) { todo!() }
    fn last(mut self) -> Option<Self::Item> { todo!() }
    fn min(mut self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        todo!()
    }
    fn max(mut self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        todo!()
    }
}

impl<K, V> DoubleEndedIterator for IntoKeys<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> { todo!() }
}

impl<K, V> ExactSizeIterator for IntoKeys<K, V> {
    fn len(&self) -> usize { todo!() }
}

impl<K, V> FusedIterator for IntoKeys<K, V> {}

impl<K, V> Default for IntoKeys<K, V> {
    fn default() -> Self { todo!() }
}

impl<K, V> Iterator for IntoValues<K, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> { todo!() }
    fn size_hint(&self) -> (usize, Option<usize>) { todo!() }
    fn last(mut self) -> Option<Self::Item> { todo!() }
}

impl<K, V> DoubleEndedIterator for IntoValues<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> { todo!() }
}

impl<K, V> ExactSizeIterator for IntoValues<K, V> {
    fn len(&self) -> usize { todo!() }
}

impl<K, V> FusedIterator for IntoValues<K, V> {}

impl<K, V> Default for IntoValues<K, V> {
    fn default() -> Self { todo!() }
}

impl<'a, K, V> DoubleEndedIterator for Range<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> { todo!() }
}

impl<K, V> Clone for Range<'_, K, V> {
    fn clone(&self) -> Self { todo!() }
}

impl<'a, K, V> Iterator for RangeMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> { todo!() }
    fn last(mut self) -> Option<Self::Item> { todo!() }
    fn min(mut self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        todo!()
    }
    fn max(mut self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        todo!()
    }
}

impl<'a, K, V> DoubleEndedIterator for RangeMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> { todo!() }
}

impl<K, V> FusedIterator for RangeMut<'_, K, V> {}

impl<K: Ord, V> FromIterator<(K, V)> for BTreeMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self { todo!() }
}

impl<K: Ord, V> Extend<(K, V)> for BTreeMap<K, V> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) { todo!() }
}

impl<'a, K: Ord + Copy, V: Copy> Extend<(&'a K, &'a V)> for BTreeMap<K, V> {
    fn extend<T: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: T) {
        todo!()
    }
}

impl<K: Hash, V: Hash> Hash for BTreeMap<K, V> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { todo!() }
}

impl<K, V> Default for BTreeMap<K, V> {
    fn default() -> Self { todo!() }
}

impl<K: PartialEq, V: PartialEq> PartialEq for BTreeMap<K, V> {
    fn eq(&self, other: &Self) -> bool { todo!() }
}

impl<K: Eq, V: Eq> Eq for BTreeMap<K, V> {}

impl<K: PartialOrd, V: PartialOrd> PartialOrd for BTreeMap<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        todo!()
    }
}

impl<K: Ord, V: Ord> Ord for BTreeMap<K, V> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { todo!() }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for BTreeMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

impl<K, Q: ?Sized, V> Index<&Q> for BTreeMap<K, V>
where
    K: Borrow<Q> + Ord,
    Q: Ord,
{
    type Output = V;

    fn index(&self, index: &Q) -> &Self::Output { todo!() }
}

impl<K: Ord, V, const N: usize> From<[(K, V); N]> for BTreeMap<K, V> {
    fn from(value: [(K, V); N]) -> Self { todo!() }
}

impl<K, V> BTreeMap<K, V> {
    pub fn iter(&self) -> Iter<'_, K, V> { todo!() }
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> { todo!() }
    pub fn keys(&self) -> Keys<'_, K, V> { todo!() }
    pub fn values(&self) -> Values<'_, K, V> { todo!() }
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> { todo!() }

    pub const fn len(&self) -> usize { todo!() }
    pub const fn is_empty(&self) -> bool { todo!() }
}
