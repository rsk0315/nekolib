pub trait MajorityVote {
    type Item;
    fn majority_vote(&self) -> Option<(&Self::Item, usize)>;
}

impl<T: Eq> MajorityVote for [T] {
    type Item = T;
    fn majority_vote(&self) -> Option<(&T, usize)> {
        let mut maj = self.get(0)?;
        let mut vote = 1;
        let n = self.len();
        for x in &self[1..] {
            if maj == x {
                vote += 1;
            } else if vote == 0 {
                maj = x;
                vote = 1;
            } else {
                vote -= 1;
            }
        }

        let mut vote = 0;
        let mut occ = 0;
        for (i, x) in self.iter().enumerate().rev() {
            if maj == x {
                vote += 1;
                occ = i;
            }
        }
        (vote > n - vote).then(|| (&self[occ], vote))
    }
}

#[test]
fn sanity_check() {
    assert_eq!([1].majority_vote(), Some((&1, 1)));
    assert_eq!([1, 2, 1, 2, 1].majority_vote(), Some((&1, 3)));
    assert_eq!([1, 2, 1, 2, 3].majority_vote(), None);
    assert_eq!([1, 2, 1, 2].majority_vote(), None);

    let empty: [i32; 0] = [];
    assert_eq!(empty.majority_vote(), None);
}
