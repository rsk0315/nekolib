//! [`BTreeSet`] の定義。

use std::{
    borrow::Borrow,
    fmt,
    hash::Hash,
    iter::{FusedIterator, Peekable},
    ops::{BitAnd, BitOr, BitXor, RangeBounds, Sub},
};

use crate::{
    map::{BTreeMap, Keys},
    merge_iter::MergeIterInner,
    set_val::SetValZST,
};

pub struct BTreeSet<T> {
    map: BTreeMap<T, SetValZST>,
}

impl<T: Hash> Hash for BTreeSet<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { todo!() }
}

impl<T: PartialEq> PartialEq for BTreeSet<T> {
    fn eq(&self, other: &Self) -> bool { todo!() }
}

impl<T: Eq> Eq for BTreeSet<T> {}

impl<T: PartialOrd> PartialOrd for BTreeSet<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        todo!()
    }
}

impl<T: Ord> Ord for BTreeSet<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { todo!() }
}

impl<T: Clone> Clone for BTreeSet<T> {
    fn clone(&self) -> Self { todo!() }
    fn clone_from(&mut self, other: &Self) { todo!() }
}

pub struct Iter<'a, T: 'a> {
    iter: Keys<'a, T, SetValZST>,
}

impl<T: fmt::Debug> fmt::Debug for Iter<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

pub struct IntoIter<T> {
    iter: super::map::IntoIter<T, SetValZST>,
}

pub struct Range<'a, T: 'a> {
    iter: super::map::Range<'a, T, SetValZST>,
}

pub struct Difference<'a, T: 'a> {
    inner: DifferenceInner<'a, T>,
}

enum DifferenceInner<'a, T: 'a> {
    Stitch { self_iter: Iter<'a, T>, other_iter: Peekable<Iter<'a, T>> },
    Search { self_iter: Iter<'a, T>, other_set: &'a BTreeSet<T> },
    Iterate(Iter<'a, T>),
}

impl<T: fmt::Debug> fmt::Debug for DifferenceInner<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

impl<T: fmt::Debug> fmt::Debug for Difference<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

pub struct SymmetricDifference<'a, T: 'a>(MergeIterInner<Iter<'a, T>>);

impl<T: fmt::Debug> fmt::Debug for SymmetricDifference<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

pub struct Intersection<'a, T: 'a> {
    inner: IntersectionInner<'a, T>,
}

enum IntersectionInner<'a, T: 'a> {
    Stitch { a: Iter<'a, T>, b: Iter<'a, T> },
    Search { small_iter: Iter<'a, T>, laerge_set: &'a BTreeSet<T> },
    Answer(Option<&'a T>),
}

impl<T: fmt::Debug> fmt::Debug for IntersectionInner<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

impl<T: fmt::Debug> fmt::Debug for Intersection<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

pub struct Union<'a, T: 'a>(MergeIterInner<Iter<'a, T>>);

impl<T: fmt::Debug> fmt::Debug for Union<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

const ITER_PERFORMANCE_TIPPING_SIZE_DIFF: usize = 16;

impl<T> BTreeSet<T> {
    pub fn new() -> BTreeSet<T> { todo!() }
}

impl<T> BTreeSet<T> {
    pub fn range<K: ?Sized, R>(&self, range: R) -> Range<'_, T>
    where
        K: Ord,
        T: Borrow<K> + Ord,
        R: RangeBounds<K>,
    {
        todo!()
    }

    pub fn difference<'a>(&'a self, other: &'a BTreeSet<T>) -> Difference<'a, T>
    where
        T: Ord,
    {
        todo!()
    }

    pub fn symmetric_difference<'a>(
        &'a self,
        other: &'a BTreeSet<T>,
    ) -> SymmetricDifference<'a, T>
    where
        T: Ord,
    {
        todo!()
    }

    pub fn intersection<'a>(
        &'a self,
        other: &'a BTreeSet<T>,
    ) -> Intersection<'a, T>
    where
        T: Ord,
    {
        todo!()
    }

    pub fn union<'a>(&'a self, other: &'a BTreeSet<T>) -> Union<'a, T>
    where
        T: Ord,
    {
        todo!()
    }

    pub fn clear(&mut self) { todo!() }

    pub fn contains<Q: ?Sized>(&self, value: &Q) -> bool
    where
        T: Borrow<Q> + Ord,
        Q: Ord,
    {
        todo!()
    }

    pub fn get<Q: ?Sized>(&self, value: &Q) -> Option<&T>
    where
        T: Borrow<Q> + Ord,
        Q: Ord,
    {
        todo!()
    }

    pub fn is_disjoint(&self, other: &BTreeSet<T>) -> bool
    where
        T: Ord,
    {
        todo!()
    }

    pub fn is_subset(&self, other: &BTreeSet<T>) -> bool
    where
        T: Ord,
    {
        todo!()
    }

    pub fn is_superset(&self, other: &BTreeSet<T>) -> bool
    where
        T: Ord,
    {
        todo!()
    }

    pub fn first(&self) -> Option<&T>
    where
        T: Ord,
    {
        todo!()
    }

    pub fn last(&self) -> Option<&T>
    where
        T: Ord,
    {
        todo!()
    }

    pub fn pop_first(&mut self) -> Option<T>
    where
        T: Ord,
    {
        todo!()
    }

    pub fn pop_last(&mut self) -> Option<T>
    where
        T: Ord,
    {
        todo!()
    }

    pub fn insert(&mut self, value: T) -> bool
    where
        T: Ord,
    {
        todo!()
    }

    pub fn replace(&mut self, value: T) -> Option<T>
    where
        T: Ord,
    {
        todo!()
    }

    pub fn remove<Q: ?Sized>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q> + Ord,
        Q: Ord,
    {
        todo!()
    }

    pub fn take<Q: ?Sized>(&mut self, value: &Q) -> Option<T>
    where
        T: Borrow<Q> + Ord,
        Q: Ord,
    {
        todo!()
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        T: Ord,
        F: FnMut(&T) -> bool,
    {
        todo!()
    }

    pub fn append(&mut self, other: &mut Self)
    where
        T: Ord,
    {
        todo!()
    }

    pub fn split_off<Q: ?Sized + Ord>(&mut self, value: &Q) -> Self
    where
        T: Borrow<Q> + Ord,
    {
        todo!()
    }

    pub fn iter(&self) -> Iter<'_, T> { todo!() }

    pub const fn len(&self) -> usize { todo!() }

    pub const fn is_empty(&self) -> bool { todo!() }
}

impl<T: Ord> FromIterator<T> for BTreeSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self { todo!() }
}

impl<T: Ord> BTreeSet<T> {
    fn from_sorted_iter<I: Iterator<Item = T>>(iter: I) -> BTreeSet<T> {
        todo!()
    }
}

impl<T: Ord, const N: usize> From<[T; N]> for BTreeSet<T> {
    fn from(value: [T; N]) -> Self { todo!() }
}

impl<T> IntoIterator for BTreeSet<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter { todo!() }
}

impl<'a, T> IntoIterator for &'a BTreeSet<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter { todo!() }
}

impl<T: Ord> Extend<T> for BTreeSet<T> {
    fn extend<Iter: IntoIterator<Item = T>>(&mut self, iter: Iter) { todo!() }
}

impl<'a, T: 'a + Ord + Copy> Extend<&'a T> for BTreeSet<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) { todo!() }
}

impl<T> Default for BTreeSet<T> {
    fn default() -> Self { todo!() }
}

impl<T: Ord + Clone> Sub<&BTreeSet<T>> for &BTreeSet<T> {
    type Output = BTreeSet<T>;

    fn sub(self, rhs: &BTreeSet<T>) -> Self::Output { todo!() }
}

impl<T: Ord + Clone> BitXor<&BTreeSet<T>> for &BTreeSet<T> {
    type Output = BTreeSet<T>;

    fn bitxor(self, rhs: &BTreeSet<T>) -> Self::Output { todo!() }
}

impl<T: Ord + Clone> BitAnd<&BTreeSet<T>> for &BTreeSet<T> {
    type Output = BTreeSet<T>;

    fn bitand(self, rhs: &BTreeSet<T>) -> Self::Output { todo!() }
}

impl<T: Ord + Clone> BitOr<&BTreeSet<T>> for &BTreeSet<T> {
    type Output = BTreeSet<T>;

    fn bitor(self, rhs: &BTreeSet<T>) -> Self::Output { todo!() }
}

impl<T: fmt::Debug> fmt::Debug for BTreeSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

impl<T> Clone for Iter<'_, T> {
    fn clone(&self) -> Self { todo!() }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> { todo!() }
    fn size_hint(&self) -> (usize, Option<usize>) { todo!() }
    fn last(mut self) -> Option<Self::Item> { todo!() }
    fn min(mut self) -> Option<&'a T>
    where
        Self::Item: Ord,
    {
        todo!()
    }
    fn max(mut self) -> Option<&'a T>
    where
        Self::Item: Ord,
    {
        todo!()
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> { todo!() }
}

impl<T> ExactSizeIterator for Iter<'_, T> {
    fn len(&self) -> usize { todo!() }
}

impl<T> FusedIterator for Iter<'_, T> {}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> { todo!() }
    fn size_hint(&self) -> (usize, Option<usize>) { todo!() }
}

impl<T> Default for Iter<'_, T> {
    fn default() -> Self { todo!() }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> { todo!() }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize { todo!() }
}

impl<T> FusedIterator for IntoIter<T> {}

impl<T> Default for IntoIter<T> {
    fn default() -> Self { todo!() }
}

impl<T> Clone for Range<'_, T> {
    fn clone(&self) -> Self { todo!() }
}

impl<'a, T> Iterator for Range<'a, T> {
    type Item = &'a T;

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

impl<'a, T> DoubleEndedIterator for Range<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> { todo!() }
}

impl<T> FusedIterator for Range<'_, T> {}

impl<T> Default for Range<'_, T> {
    fn default() -> Self { todo!() }
}

impl<T> Clone for Difference<'_, T> {
    fn clone(&self) -> Self { todo!() }
}

impl<'a, T: Ord> Iterator for Difference<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> { todo!() }
    fn size_hint(&self) -> (usize, Option<usize>) { todo!() }
    fn min(mut self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        todo!()
    }
}

impl<T: Ord> FusedIterator for Difference<'_, T> {}

impl<T> Clone for SymmetricDifference<'_, T> {
    fn clone(&self) -> Self { todo!() }
}

impl<'a, T: Ord> Iterator for SymmetricDifference<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> { todo!() }
    fn size_hint(&self) -> (usize, Option<usize>) { todo!() }
    fn min(mut self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        todo!()
    }
}

impl<T: Ord> FusedIterator for SymmetricDifference<'_, T> {}

impl<T> Clone for Intersection<'_, T> {
    fn clone(&self) -> Self { todo!() }
}

impl<'a, T: Ord> Iterator for Intersection<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> { todo!() }
    fn size_hint(&self) -> (usize, Option<usize>) { todo!() }
    fn min(mut self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        todo!()
    }
}

impl<T: Ord> FusedIterator for Intersection<'_, T> {}

impl<T> Clone for Union<'_, T> {
    fn clone(&self) -> Self { todo!() }
}

impl<'a, T: Ord> Iterator for Union<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> { todo!() }
    fn size_hint(&self) -> (usize, Option<usize>) { todo!() }
    fn min(mut self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        todo!()
    }
}

impl<T: Ord> FusedIterator for Union<'_, T> {}
