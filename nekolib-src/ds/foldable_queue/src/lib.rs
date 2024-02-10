use std::ops::RangeFull;

use monoid::Monoid;

#[derive(Debug)]
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

#[cfg(test)]
mod naive {
    use std::{iter::Sum, ops::Add};

    #[derive(Debug, Eq, PartialEq)]
    pub struct Seq(pub Vec<u32>);
    impl Add<&Seq> for &Seq {
        type Output = Seq;
        fn add(self, rhs: &Seq) -> Seq {
            Seq(self.0.iter().chain(&rhs.0).copied().collect())
        }
    }
    impl<'a> Sum<&'a Seq> for Seq {
        fn sum<I: IntoIterator<Item = &'a Seq>>(iter: I) -> Seq {
            Seq(iter
                .into_iter()
                .flat_map(|seq| seq.0.iter().copied())
                .collect())
        }
    }
    impl Seq {
        pub fn new(x: u32) -> Self { Self(vec![x]) }
    }
}

#[test]
fn sanity_check() {
    use op_add::OpAdd;

    use crate::naive::Seq;

    let mut queue = FoldableQueue::<OpAdd<Seq>>::new();
    queue.push(Seq::new(1));
    assert_eq!(queue.fold(..), Seq(vec![1]));
    assert_eq!(queue.pop(), Some(Seq::new(1)));

    assert_eq!(queue.fold(..), Seq(vec![]));
    queue.push(Seq::new(1));
    queue.push(Seq::new(2));
    assert_eq!(queue.fold(..), Seq(vec![1, 2]));
    queue.push(Seq::new(3));
    assert_eq!(queue.fold(..), Seq(vec![1, 2, 3]));

    assert_eq!(queue.pop(), Some(Seq::new(1)));
    assert_eq!(queue.fold(..), Seq(vec![2, 3]));
    queue.push(Seq::new(4));
    assert_eq!(queue.fold(..), Seq(vec![2, 3, 4]));
    assert_eq!(queue.pop(), Some(Seq::new(2)));
    assert_eq!(queue.fold(..), Seq(vec![3, 4]));
    assert_eq!(queue.pop(), Some(Seq::new(3)));
    assert_eq!(queue.fold(..), Seq(vec![4]));
    assert_eq!(queue.pop(), Some(Seq::new(4)));
    assert_eq!(queue.fold(..), Seq(vec![]));
    assert_eq!(queue.pop(), None);
    assert_eq!(queue.fold(..), Seq(vec![]));
    assert_eq!(queue.pop(), None);
    assert_eq!(queue.fold(..), Seq(vec![]));
}
