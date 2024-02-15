pub struct RankIndex(Vec<usize>);

impl RankIndex {
    pub fn new(a: Vec<bool>) -> Self {
        let n = a.len();
        let mut res = vec![0; n];
        let mut count = 0;
        for i in 0..n {
            if a[i] {
                count += 1;
            }
            res[i] = count;
        }
        Self(res)
    }
    pub fn rank(&self, i: usize) -> usize { self.0[i] }
}

pub struct SelectIndex(Vec<usize>);

impl SelectIndex {
    pub fn new<const X: bool>(a: Vec<bool>) -> Self {
        let n = a.len();
        let mut res = vec![0; n];
        let mut count = 0;
        for i in 0..n {
            if a[i] == X {
                res[count] = i;
                count += 1;
            }
        }
        Self(res)
    }
    pub fn select(&self, i: usize) -> usize { self.0[i] }
}

pub fn select_word<const X: bool>(mut w: u64, mut i: u32) -> u32 {
    if !X {
        w = !w;
    }

    let mut res = 0;
    for lg2 in (0..6).rev() {
        let len = 1 << lg2;
        let mask = !(!0 << len);
        let count = (w & mask).count_ones();
        if count <= i {
            w >>= len;
            i -= count;
            res += len;
        }
    }
    res
}

#[test]
fn sanity_check() {
    // % echo 0b_$(shuf -e {0,1}{0,1}{0,1}{0,1} | paste -sd _ -)_u64

    let w = 0b_1100_1011_0010_1101_1001_1000_0001_0100_1111_1110_0000_0011_0111_0101_1010_0110_u64;
    let mut wi = w;
    for i in 0..32 {
        let expected = wi.trailing_zeros();
        let actual = select_word::<true>(w, i);
        assert_eq!(actual, expected);
        wi ^= wi & wi.wrapping_neg();
    }
}
