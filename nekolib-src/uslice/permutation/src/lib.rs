pub trait Permutation {
    fn inv(&self) -> Vec<usize>;
}

impl Permutation for [usize] {
    fn inv(&self) -> Vec<usize> {
        let n = self.len();
        let mut res = vec![0; n];
        for i in 0..n {
            res[self[i]] = i;
        }
        res
    }
}

#[test]
fn sanity_check() {
    let a = [1, 5, 2, 3, 6, 0, 4];
    assert_eq!(a.inv(), [5, 0, 2, 3, 6, 1, 4]);
}
