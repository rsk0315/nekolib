//! [`MergeIterInner`] の定義。

use std::{cmp::Ordering, fmt, iter::FusedIterator};

pub struct MergeIterInner<I: Iterator> {
    a: I,
    b: I,
    peeked: Option<Peeked<I>>,
}

#[derive(Clone, Debug)]
enum Peeked<I: Iterator> {
    A(I::Item),
    B(I::Item),
}

impl<I: Iterator> Clone for MergeIterInner<I>
where
    I: Clone,
    I::Item: Clone,
{
    fn clone(&self) -> Self { todo!() }
}

impl<I: Iterator> fmt::Debug for MergeIterInner<I>
where
    I: fmt::Debug,
    I::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
}

impl<I: Iterator> MergeIterInner<I> {
    pub fn new(a: I, b: I) -> Self { todo!() }

    pub fn nexts<Cmp: Fn(&I::Item, &I::Item) -> Ordering>(
        &mut self,
        cmp: Cmp,
    ) -> (Option<I::Item>, Option<I::Item>)
    where
        I: FusedIterator,
    {
        todo!()
    }

    pub fn lens(&self) -> (usize, usize)
    where
        I: ExactSizeIterator,
    {
        todo!()
    }
}
