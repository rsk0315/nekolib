use std::ops::RangeFull;

use monoid::Monoid;

#[derive(Clone, Eq, PartialEq)]
pub struct FoldableDeque<M: Monoid> {
    front: Vec<M::Set>,
    front_folded: Vec<M::Set>,
    back: Vec<M::Set>,
    back_folded: Vec<M::Set>,
    monoid: M,
}

impl<M: Monoid> FoldableDeque<M>
where
    M::Set: Clone,
{
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
    pub fn push_back(&mut self, elt: M::Set) {
        let tmp = self.monoid.op(self.back_folded.last().unwrap(), &elt);
        self.back_folded.push(tmp);
        self.back.push(elt);
    }
    pub fn push_front(&mut self, elt: M::Set) {
        let tmp = self.monoid.op(&elt, self.front_folded.last().unwrap());
        self.front_folded.push(tmp);
        self.front.push(elt);
    }
    pub fn pop_back(&mut self) -> Option<M::Set> {
        self.rotate_back();
        let elt = self.back.pop()?;
        self.back_folded.pop().unwrap();
        Some(elt)
    }
    pub fn pop_front(&mut self) -> Option<M::Set> {
        self.rotate_front();
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

    fn rotate_front(&mut self) {
        if !self.front.is_empty() {
            return;
        }
        let mut front = std::mem::take(&mut self.back);
        let len = front.len();

        // *][01234*; front: [], back: [0, 1, 2, 3, 4]
        // *012][34*; front: [2, 1, 0], back: [3, 4]
        let back = front.split_off((len + 1) / 2);
        front.reverse();
        self.front = front;
        self.back = back;
        self.fixup();
    }
    fn rotate_back(&mut self) {
        if !self.back.is_empty() {
            return;
        }
        let mut back = std::mem::take(&mut self.front);
        let len = back.len();

        // *01234][*; front: [4, 3, 2, 1, 0], back: []
        // *01][234*; front: [1, 0], back: [2, 3, 4]
        let front = back.split_off((len + 1) / 2);
        back.reverse();
        self.front = front;
        self.back = back;
        self.fixup();
    }
    fn fixup(&mut self) {
        self.front_folded = vec![self.monoid.id()];
        self.front_folded.extend(self.front.iter().scan(
            self.monoid.id(),
            |acc, x| {
                *acc = self.monoid.op(x, acc);
                Some(acc.clone())
            },
        ));

        self.back_folded = vec![self.monoid.id()];
        self.back_folded.extend(self.back.iter().scan(
            self.monoid.id(),
            |acc, x| {
                *acc = self.monoid.op(acc, x);
                Some(acc.clone())
            },
        ));
    }
}

impl<M: Monoid> std::fmt::Debug for FoldableDeque<M>
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
mod tests {
    use concat_monoid::OpConcat;

    use crate::FoldableDeque;

    #[test]
    fn sanity_check() {
        let mut queue = FoldableDeque::<OpConcat<_, Vec<_>>>::new();
        queue.push_back(vec![1]);
        assert_eq!(queue.fold(..), vec![1]);
        assert_eq!(queue.pop_back(), Some(vec![1]));

        queue.push_back(vec![1]);
        queue.push_back(vec![2]);
        queue.push_back(vec![3]);
        queue.push_back(vec![4]);
        queue.push_back(vec![5]);
        assert_eq!(queue.fold(..), vec![1, 2, 3, 4, 5]);

        assert_eq!(queue.pop_front(), Some(vec![1]));
        assert_eq!(queue.fold(..), vec![2, 3, 4, 5]);
        assert_eq!(queue.pop_front(), Some(vec![2]));
        assert_eq!(queue.fold(..), vec![3, 4, 5]);
        assert_eq!(queue.pop_front(), Some(vec![3]));
        assert_eq!(queue.fold(..), vec![4, 5]);
        assert_eq!(queue.pop_front(), Some(vec![4]));
        assert_eq!(queue.fold(..), vec![5]);
        assert_eq!(queue.pop_front(), Some(vec![5]));
        assert_eq!(queue.fold(..), vec![]);
        assert_eq!(queue.pop_front(), None);
        assert_eq!(queue.fold(..), vec![]);

        queue.push_front(vec![5]);
        queue.push_front(vec![4]);
        queue.push_front(vec![3]);
        queue.push_front(vec![2]);
        queue.push_front(vec![1]);
        assert_eq!(queue.fold(..), vec![1, 2, 3, 4, 5]);

        assert_eq!(queue.pop_back(), Some(vec![5]));
        assert_eq!(queue.fold(..), vec![1, 2, 3, 4]);
        assert_eq!(queue.pop_back(), Some(vec![4]));
        assert_eq!(queue.fold(..), vec![1, 2, 3]);
        assert_eq!(queue.pop_back(), Some(vec![3]));
        assert_eq!(queue.fold(..), vec![1, 2]);
        assert_eq!(queue.pop_back(), Some(vec![2]));
        assert_eq!(queue.fold(..), vec![1]);
        assert_eq!(queue.pop_back(), Some(vec![1]));
        assert_eq!(queue.fold(..), vec![]);
        assert_eq!(queue.pop_back(), None);
        assert_eq!(queue.fold(..), vec![]);
    }

    #[test]
    fn test_fmt() {
        let mut queue = FoldableDeque::<OpConcat<_, Vec<_>>>::new();
        assert_eq!(format!("{queue:?}"), "[]");
        queue.push_front(vec![2]);
        queue.push_front(vec![1]);
        queue.push_back(vec![3]);
        queue.push_back(vec![4]);
        assert_eq!(format!("{queue:?}"), "[[1], [2], [3], [4]]");
    }
}
