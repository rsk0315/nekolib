use std::{
    iter::{Product, Sum},
    ops::{Add, AddAssign},
};

pub trait Inversion {
    fn inversion<I: Add<I> + for<'a> AddAssign<&'a I> + Sum<I> + Product<I>>(
        &self,
    ) -> I;
}

impl<T: Ord> Inversion for [T] {
    fn inversion<I: Add<I> + for<'a> AddAssign<&'a I> + Sum<I> + Product<I>>(
        &self,
    ) -> I {
        let n = self.len();
        let ord = {
            let mut ord: Vec<_> = (0..n).collect();
            ord.sort_unstable_by(|&il, &ir| {
                self[il].cmp(&self[ir]).then_with(|| il.cmp(&ir))
            });
            ord
        };

        let zero = || None.into_iter().sum::<I>();
        let one = || None.into_iter().product::<I>();
        let mut res = zero();
        let mut sum: Vec<_> = (0..=n).map(|_| zero()).collect();
        for i in ord.iter().map(|&i| i + 1) {
            {
                let mut i = i;
                while i <= n {
                    res += &sum[i];
                    i += i & i.wrapping_neg();
                }
            }
            {
                let mut i = i;
                while i > 0 {
                    sum[i] += &one();
                    i -= i & i.wrapping_neg();
                }
            }
        }

        res
    }
}

#[test]
fn sanity_check() {
    assert_eq!([1, 5, 4, 2, 3].inversion::<usize>(), 5);
    assert_eq!([1, 2, 3, 4, 5].inversion::<usize>(), 0);
    assert_eq!([5, 4, 3, 2, 1].inversion::<usize>(), 10);
    assert_eq!([1, 1, 1, 1, 1].inversion::<usize>(), 0);

    let empty: [(); 0] = [];
    assert_eq!(empty.inversion::<usize>(), 0);
}
