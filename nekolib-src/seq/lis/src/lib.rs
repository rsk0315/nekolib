use std::cmp::Reverse;

pub trait LisMapProj<T: Ord + Clone> {
    type Mapped: Ord + Clone;
    fn map(&self, index: usize, elt: &T) -> Self::Mapped;
    fn proj(&self, elt: &Self::Mapped) -> usize;
}

pub struct Smallest;
pub struct Largest;
pub struct Leftmost;
pub struct Rightmost;

impl<T: Ord + Clone> LisMapProj<T> for Smallest {
    type Mapped = (Reverse<T>, usize);
    fn map(&self, index: usize, elt: &T) -> (Reverse<T>, usize) {
        (Reverse(elt.clone()), index)
    }
    fn proj(&self, &(_, index): &(Reverse<T>, usize)) -> usize { index }
}
impl<T: Ord + Clone> LisMapProj<T> for Largest {
    type Mapped = (T, usize);
    fn map(&self, index: usize, elt: &T) -> (T, usize) { (elt.clone(), index) }
    fn proj(&self, &(_, index): &(T, usize)) -> usize { index }
}
impl<T: Ord + Clone> LisMapProj<T> for Leftmost {
    type Mapped = Reverse<usize>;
    fn map(&self, index: usize, _: &T) -> Reverse<usize> { Reverse(index) }
    fn proj(&self, &Reverse(index): &Reverse<usize>) -> usize { index }
}
impl<T: Ord + Clone> LisMapProj<T> for Rightmost {
    type Mapped = usize;
    fn map(&self, index: usize, _: &T) -> usize { index }
    fn proj(&self, &index: &usize) -> usize { index }
}

pub trait Lis {
    type Item: Ord + Clone;
    fn lis(
        &self,
        strict: bool,
        mp: impl LisMapProj<Self::Item>,
    ) -> Vec<Self::Item>;
}

impl<T: Ord + Clone> Lis for [T] {
    type Item = T;
    fn lis(&self, strict: bool, mp: impl LisMapProj<T>) -> Vec<Self::Item>
where {
        if self.is_empty() {
            return vec![];
        }

        let n = self.len();
        let ord = {
            let mut ord: Vec<_> = (0..n).collect();
            if strict {
                ord.reverse();
            }
            ord.sort_by_key(|&i| &self[i]);
            ord
        };
        let mut dp = FenwickTree::from(vec![None; n]);
        let mut prev = vec![None; n];
        let mut len = vec![0; n];
        for &i in &ord {
            let max = dp.max(..i).unwrap_or(None);
            let max_len = max.as_ref().map(|x: &(usize, _)| x.0).unwrap_or(0);
            dp.update(i, Some((max_len + 1, mp.map(i, &self[i]))));
            len[i] = max_len + 1;
            prev[i] = max.map(|(_, x)| mp.proj(&x));
        }
        let mut last =
            dp.max(..n).unwrap_or(None).map(|(_, x): (usize, _)| mp.proj(&x));
        let mut res = vec![];
        while let Some(i) = last {
            res.push(self[i].clone());
            last = prev[i];
        }
        res.reverse();
        res
    }
}

use std::ops::RangeTo;

struct FenwickTree<T: Ord + Clone>(Vec<T>);

impl<T: Ord + Clone> From<Vec<T>> for FenwickTree<T> {
    fn from(buf: Vec<T>) -> Self { Self(buf) }
}

impl<T: Ord + Clone> FenwickTree<T> {
    fn max(&self, RangeTo { end: i }: RangeTo<usize>) -> Option<T> {
        std::iter::successors(Some(i), |&i| Some(i - (i & i.wrapping_neg())))
            .take_while(|&i| i > 0)
            .map(|i| &self.0[i - 1])
            .max()
            .cloned()
    }
    fn update(&mut self, i: usize, x: T) {
        let n = self.0.len();
        std::iter::successors(Some(i + 1), |&i| {
            Some(i + (i & i.wrapping_neg()))
        })
        .take_while(|&i| i <= n)
        .for_each(|i| {
            if self.0[i - 1] < x {
                self.0[i - 1] = x.clone();
            }
        });
    }
}

#[test]
fn sanity_check() {
    assert_eq!([1, 1, 1].lis(true, Smallest), [1]);
    assert_eq!([1, 1, 1].lis(false, Smallest), [1, 1, 1]);

    assert_eq!([1, 2, 3, 4].lis(true, Smallest), [1, 2, 3, 4]);
    assert_eq!([1, 2, 4, 3].lis(true, Smallest), [1, 2, 3]);
    assert_eq!([1, 3, 2, 4].lis(true, Smallest), [1, 2, 4]);
    assert_eq!([1, 3, 4, 2].lis(true, Smallest), [1, 3, 4]);
    assert_eq!([1, 4, 2, 3].lis(true, Smallest), [1, 2, 3]);
    assert_eq!([1, 4, 3, 2].lis(true, Smallest), [1, 2]);

    assert_eq!([2, 1, 3, 4].lis(true, Smallest), [1, 3, 4]);
    assert_eq!([2, 1, 4, 3].lis(true, Smallest), [1, 3]);
    assert_eq!([2, 3, 1, 4].lis(true, Smallest), [2, 3, 4]);
    assert_eq!([2, 3, 4, 1].lis(true, Smallest), [2, 3, 4]);
    assert_eq!([2, 4, 1, 3].lis(true, Smallest), [1, 3]);
    assert_eq!([2, 4, 3, 1].lis(true, Smallest), [2, 3]);

    assert_eq!([3, 1, 2, 4].lis(true, Smallest), [1, 2, 4]);
    assert_eq!([3, 1, 4, 2].lis(true, Smallest), [1, 2]);
    assert_eq!([3, 2, 1, 4].lis(true, Smallest), [1, 4]);
    assert_eq!([3, 2, 4, 1].lis(true, Smallest), [2, 4]);
    assert_eq!([3, 4, 1, 2].lis(true, Smallest), [1, 2]);
    assert_eq!([3, 4, 2, 1].lis(true, Smallest), [3, 4]);

    assert_eq!([4, 1, 2, 3].lis(true, Smallest), [1, 2, 3]);
    assert_eq!([4, 1, 3, 2].lis(true, Smallest), [1, 2]);
    assert_eq!([4, 2, 1, 3].lis(true, Smallest), [1, 3]);
    assert_eq!([4, 2, 3, 1].lis(true, Smallest), [2, 3]);
    assert_eq!([4, 3, 1, 2].lis(true, Smallest), [1, 2]);
    assert_eq!([4, 3, 2, 1].lis(true, Smallest), [1]);

    assert_eq!([1, 2, 3, 4].lis(true, Largest), [1, 2, 3, 4]);
    assert_eq!([1, 2, 4, 3].lis(true, Largest), [1, 2, 4]);
    assert_eq!([1, 3, 2, 4].lis(true, Largest), [1, 3, 4]);
    assert_eq!([1, 3, 4, 2].lis(true, Largest), [1, 3, 4]);
    assert_eq!([1, 4, 2, 3].lis(true, Largest), [1, 2, 3]);
    assert_eq!([1, 4, 3, 2].lis(true, Largest), [1, 4]);

    assert_eq!([2, 1, 3, 4].lis(true, Largest), [2, 3, 4]);
    assert_eq!([2, 1, 4, 3].lis(true, Largest), [2, 4]);
    assert_eq!([2, 3, 1, 4].lis(true, Largest), [2, 3, 4]);
    assert_eq!([2, 3, 4, 1].lis(true, Largest), [2, 3, 4]);
    assert_eq!([2, 4, 1, 3].lis(true, Largest), [2, 4]);
    assert_eq!([2, 4, 3, 1].lis(true, Largest), [2, 4]);

    assert_eq!([3, 1, 2, 4].lis(true, Largest), [1, 2, 4]);
    assert_eq!([3, 1, 4, 2].lis(true, Largest), [3, 4]);
    assert_eq!([3, 2, 1, 4].lis(true, Largest), [3, 4]);
    assert_eq!([3, 2, 4, 1].lis(true, Largest), [3, 4]);
    assert_eq!([3, 4, 1, 2].lis(true, Largest), [3, 4]);
    assert_eq!([3, 4, 2, 1].lis(true, Largest), [3, 4]);

    assert_eq!([4, 1, 2, 3].lis(true, Largest), [1, 2, 3]);
    assert_eq!([4, 1, 3, 2].lis(true, Largest), [1, 3]);
    assert_eq!([4, 2, 1, 3].lis(true, Largest), [2, 3]);
    assert_eq!([4, 2, 3, 1].lis(true, Largest), [2, 3]);
    assert_eq!([4, 3, 1, 2].lis(true, Largest), [1, 2]);
    assert_eq!([4, 3, 2, 1].lis(true, Largest), [4]);

    assert_eq!([1, 2, 3, 4].lis(true, Leftmost), [1, 2, 3, 4]);
    assert_eq!([1, 2, 4, 3].lis(true, Leftmost), [1, 2, 4]);
    assert_eq!([1, 3, 2, 4].lis(true, Leftmost), [1, 3, 4]);
    assert_eq!([1, 3, 4, 2].lis(true, Leftmost), [1, 3, 4]);
    assert_eq!([1, 4, 2, 3].lis(true, Leftmost), [1, 2, 3]);
    assert_eq!([1, 4, 3, 2].lis(true, Leftmost), [1, 4]);

    assert_eq!([2, 1, 3, 4].lis(true, Leftmost), [2, 3, 4]);
    assert_eq!([2, 1, 4, 3].lis(true, Leftmost), [2, 4]);
    assert_eq!([2, 3, 1, 4].lis(true, Leftmost), [2, 3, 4]);
    assert_eq!([2, 3, 4, 1].lis(true, Leftmost), [2, 3, 4]);
    assert_eq!([2, 4, 1, 3].lis(true, Leftmost), [2, 4]);
    assert_eq!([2, 4, 3, 1].lis(true, Leftmost), [2, 4]);

    assert_eq!([3, 1, 2, 4].lis(true, Leftmost), [1, 2, 4]);
    assert_eq!([3, 1, 4, 2].lis(true, Leftmost), [3, 4]);
    assert_eq!([3, 2, 1, 4].lis(true, Leftmost), [3, 4]);
    assert_eq!([3, 2, 4, 1].lis(true, Leftmost), [3, 4]);
    assert_eq!([3, 4, 1, 2].lis(true, Leftmost), [3, 4]);
    assert_eq!([3, 4, 2, 1].lis(true, Leftmost), [3, 4]);

    assert_eq!([4, 1, 2, 3].lis(true, Leftmost), [1, 2, 3]);
    assert_eq!([4, 1, 3, 2].lis(true, Leftmost), [1, 3]);
    assert_eq!([4, 2, 1, 3].lis(true, Leftmost), [2, 3]);
    assert_eq!([4, 2, 3, 1].lis(true, Leftmost), [2, 3]);
    assert_eq!([4, 3, 1, 2].lis(true, Leftmost), [1, 2]);
    assert_eq!([4, 3, 2, 1].lis(true, Leftmost), [4]);

    assert_eq!([1, 2, 3, 4].lis(true, Rightmost), [1, 2, 3, 4]);
    assert_eq!([1, 2, 4, 3].lis(true, Rightmost), [1, 2, 3]);
    assert_eq!([1, 3, 2, 4].lis(true, Rightmost), [1, 2, 4]);
    assert_eq!([1, 3, 4, 2].lis(true, Rightmost), [1, 3, 4]);
    assert_eq!([1, 4, 2, 3].lis(true, Rightmost), [1, 2, 3]);
    assert_eq!([1, 4, 3, 2].lis(true, Rightmost), [1, 2]);

    assert_eq!([2, 1, 3, 4].lis(true, Rightmost), [1, 3, 4]);
    assert_eq!([2, 1, 4, 3].lis(true, Rightmost), [1, 3]);
    assert_eq!([2, 3, 1, 4].lis(true, Rightmost), [2, 3, 4]);
    assert_eq!([2, 3, 4, 1].lis(true, Rightmost), [2, 3, 4]);
    assert_eq!([2, 4, 1, 3].lis(true, Rightmost), [1, 3]);
    assert_eq!([2, 4, 3, 1].lis(true, Rightmost), [2, 3]);

    assert_eq!([3, 1, 2, 4].lis(true, Rightmost), [1, 2, 4]);
    assert_eq!([3, 1, 4, 2].lis(true, Rightmost), [1, 2]);
    assert_eq!([3, 2, 1, 4].lis(true, Rightmost), [1, 4]);
    assert_eq!([3, 2, 4, 1].lis(true, Rightmost), [2, 4]);
    assert_eq!([3, 4, 1, 2].lis(true, Rightmost), [1, 2]);
    assert_eq!([3, 4, 2, 1].lis(true, Rightmost), [3, 4]);

    assert_eq!([4, 1, 2, 3].lis(true, Rightmost), [1, 2, 3]);
    assert_eq!([4, 1, 3, 2].lis(true, Rightmost), [1, 2]);
    assert_eq!([4, 2, 1, 3].lis(true, Rightmost), [1, 3]);
    assert_eq!([4, 2, 3, 1].lis(true, Rightmost), [2, 3]);
    assert_eq!([4, 3, 1, 2].lis(true, Rightmost), [1, 2]);
    assert_eq!([4, 3, 2, 1].lis(true, Rightmost), [1]);
}

#[test]
fn check() {
    for ai in (0..7_u32.pow(7)).map(|x| {
        std::iter::successors(Some(x), |x| Some(x / 7))
            .map(|x| x % 7)
            .take(7)
            .collect::<Vec<_>>()
    }) {
        assert_eq!(ai.lis(true, Smallest), ai.lis(true, Rightmost));
        assert_eq!(ai.lis(true, Largest), ai.lis(true, Leftmost));
        assert_eq!(ai.lis(false, Smallest), ai.lis(false, Rightmost));
        assert_eq!(ai.lis(false, Largest), ai.lis(false, Leftmost));
    }
}
