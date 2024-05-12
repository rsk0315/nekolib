//! [`MergeIter`] の定義。

use std::iter::FusedIterator;

use crate::{merge_iter::MergeIterInner, node::Root};

impl<K, V> Root<K, V> {
    pub fn append_from_sorted_iters<I>(
        &mut self,
        left: I,
        right: I,
        length: &mut usize,
    ) where
        K: Ord,
        I: Iterator<Item = (K, V)> + FusedIterator,
    {
        todo!()
    }

    pub fn bulk_push<I>(&mut self, iter: I, length: &mut usize)
    where
        I: Iterator<Item = (K, V)>,
    {
        todo!()
    }
}

struct MergeIter<K, V, I: Iterator<Item = (K, V)>>(MergeIterInner<I>);

impl<K: Ord, V, I> Iterator for MergeIter<K, V, I>
where
    I: Iterator<Item = (K, V)> + FusedIterator,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> { todo!() }
}
