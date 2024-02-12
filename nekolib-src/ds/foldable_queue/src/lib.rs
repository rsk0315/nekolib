use std::ops::RangeFull;

use monoid::Monoid;

#[derive(Clone, PartialEq, Eq)]
pub struct FoldableQueue<M: Monoid> {
    front: Vec<M::Set>,
    front_folded: Vec<M::Set>,
    back: Vec<M::Set>,
    back_folded: Vec<M::Set>,
    monoid: M,
}

impl<M: Monoid> FoldableQueue<M> {
    pub fn new() -> Self
    where
        M: Default,
    {
        let monoid = M::default();
        Self {
            front: vec![],
            front_folded: vec![monoid.id()],
            back: vec![],
            back_folded: vec![monoid.id()],
            monoid,
        }
    }
    pub fn push(&mut self, elt: M::Set) {
        let tmp = self.monoid.op(self.back_folded.last().unwrap(), &elt);
        self.back_folded.push(tmp);
        self.back.push(elt);
    }
    pub fn pop(&mut self) -> Option<M::Set> {
        self.rotate();
        let elt = self.front.pop()?;
        self.front_folded.pop().unwrap();
        Some(elt)
    }
    pub fn fold(&self, _: RangeFull) -> M::Set {
        self.monoid.op(
            self.front_folded.last().unwrap(),
            self.back_folded.last().unwrap(),
        )
    }

    fn rotate(&mut self) {
        if !self.front.is_empty() {
            return;
        }
        while let Some(elt) = self.back.pop() {
            self.back_folded.pop();
            self.front_folded
                .push(self.monoid.op(&elt, self.front_folded.last().unwrap()));
            self.front.push(elt);
        }
    }
}

impl<M: Monoid> std::fmt::Debug for FoldableQueue<M>
where
    M::Set: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.front.iter().rev().chain(self.back.iter()))
            .finish()
    }
}

#[cfg(test)]
mod naive {
    use concat_monoid::OpConcat;

    use crate::*;

    #[test]
    fn sanity_check() {
        let mut queue = FoldableQueue::<OpConcat<i32, Vec<_>>>::new();
        queue.push(vec![1]);
        assert_eq!(queue.fold(..), vec![1]);
        assert_eq!(queue.pop(), Some(vec![1]));

        queue.push(vec![1]);
        assert_eq!(queue.fold(..), vec![1]);
        queue.push(vec![2]);
        assert_eq!(queue.fold(..), vec![1, 2]);
        queue.push(vec![3]);
        assert_eq!(queue.fold(..), vec![1, 2, 3]);

        assert_eq!(queue.pop(), Some(vec![1]));
        assert_eq!(queue.fold(..), vec![2, 3]);

        queue.push(vec![4]);
        assert_eq!(queue.fold(..), vec![2, 3, 4]);
        assert_eq!(queue.pop(), Some(vec![2]));
        assert_eq!(queue.fold(..), vec![3, 4]);
        assert_eq!(queue.pop(), Some(vec![3]));
        assert_eq!(queue.fold(..), vec![4]);
        assert_eq!(queue.pop(), Some(vec![4]));
        assert_eq!(queue.fold(..), vec![]);
        assert_eq!(queue.pop(), None);
        assert_eq!(queue.fold(..), vec![]);
    }

    #[test]
    fn test_fmt() {
        let mut queue = FoldableQueue::<OpConcat<_, Vec<_>>>::new();
        assert_eq!(format!("{queue:?}"), "[]");
        queue.push(vec![1]);
        queue.push(vec![2]);
        queue.push(vec![3]);
        queue.push(vec![4]);
        assert_eq!(format!("{queue:?}"), "[[1], [2], [3], [4]]");
    }
}
