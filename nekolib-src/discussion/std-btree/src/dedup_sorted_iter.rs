//! [`DedupSortedIter`] の定義。

use std::iter::Peekable;

pub struct DedupSortedIter<K, V, I>
where
    I: Iterator<Item = (K, V)>,
{
    iter: Peekable<I>,
}

impl<K, V, I> DedupSortedIter<K, V, I>
where
    I: Iterator<Item = (K, V)>,
{
    pub fn new(iter: I) -> Self { todo!() }
}

impl<K, V, I> Iterator for DedupSortedIter<K, V, I>
where
    K: Eq,
    I: Iterator<Item = (K, V)>,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> { todo!() }
}
