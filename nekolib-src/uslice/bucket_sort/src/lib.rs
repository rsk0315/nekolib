pub trait BucketSort {
    fn bucket_sort(&mut self);
}

impl BucketSort for [usize] {
    fn bucket_sort(&mut self) {
        if self.is_empty() {
            return;
        }

        let m = *self.iter().max().unwrap();
        let mut count = vec![0; m + 1];
        for &ai in &*self {
            count[ai] += 1;
        }
        let mut i = 0;
        for j in 0..=m {
            for _ in 0..count[j] {
                self[i] = j;
                i += 1;
            }
        }
    }
}

#[test]
fn sanity_check() {
    let mut empty = vec![];
    empty.bucket_sort();
    assert_eq!(empty, []);

    let mut a = vec![3, 1, 4, 1, 5, 9, 2];
    a.bucket_sort();
    assert_eq!(a, [1, 1, 2, 3, 4, 5, 9]);
}
