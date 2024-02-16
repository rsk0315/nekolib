// N: O(n)
// Nl: O(n log(n))
// Nll: O(n log(log(n)))
// L: O(log(n))
// Ll: O(log(log(n)))
// C: O(1), constant
//
// e.g. FooNllC: <O(n log(log(n))), O(1)> implementation for Foo.
// In this context, it means <space complexity (in bits), query-time complexity>.

use std::ops::Range;

pub struct RankIndexNlC(Vec<usize>);

impl RankIndexNlC {
    pub fn new(a: &[bool]) -> Self {
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
    pub fn rank<const X: bool>(&self, i: usize) -> usize {
        if X { self.0[i] } else { i + 1 - self.0[i] }
    }
}

pub struct SelectIndexNlC(Vec<usize>);

impl SelectIndexNlC {
    pub fn new<const X: bool>(a: &[bool]) -> Self {
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

pub struct Rs01DictNlC {
    rank_index: RankIndexNlC,
    select1_index: SelectIndexNlC,
    select0_index: SelectIndexNlC,
}

impl Rs01DictNlC {
    pub fn new(a: &[bool]) -> Self {
        Self {
            rank_index: RankIndexNlC::new(a),
            select1_index: SelectIndexNlC::new::<true>(a),
            select0_index: SelectIndexNlC::new::<false>(a),
        }
    }
    fn rank<const X: bool>(&self, i: usize) -> usize {
        self.rank_index.rank::<X>(i)
    }
    fn select<const X: bool>(&self, i: usize) -> usize {
        if X {
            self.select1_index.select(i)
        } else {
            self.select0_index.select(i)
        }
    }

    pub fn rank1(&self, i: usize) -> usize { self.rank::<true>(i) }
    pub fn rank0(&self, i: usize) -> usize { self.rank::<false>(i) }
    pub fn select1(&self, i: usize) -> usize { self.select::<true>(i) }
    pub fn select0(&self, i: usize) -> usize { self.select::<false>(i) }
}

struct RankIndexNC {
    block: Vec<usize>,
    buf: Vec<u64>,
}

const W: usize = u64::BITS as usize;

impl RankIndexNC {
    pub fn new(a: &[bool]) -> Self {
        let len = a.len();
        let n = (len + W - 1) / W;
        let mut buf = vec![0_u64; n + 1];
        for i in 0..len {
            if a[i] {
                buf[i / W] |= 1 << (i % W);
            }
        }
        let block: Vec<_> = buf
            .iter()
            .map(|x| x.count_ones() as usize)
            .scan(0, |acc, x| Some(std::mem::replace(acc, *acc + x)))
            .collect();
        Self { block, buf }
    }
    pub fn rank<const X: bool>(&self, i: usize) -> usize {
        let i = i + 1;
        let large = i / W;
        let small = i % W;
        let mini = self.buf[large] & !(!0 << small);
        let count1 = self.block[large] + mini.count_ones() as usize;
        if X { count1 } else { i - count1 }
    }
}

struct SelectIndexNLl<const POPCNT: usize, const SPARSE_LEN: usize> {
    inner: Vec<SelectIndexNLlInner>,
}

impl<const POPCNT: usize, const SPARSE_LEN: usize>
    SelectIndexNLl<POPCNT, SPARSE_LEN>
{
    pub fn new<const X: bool>(a: &[bool]) -> Self {
        let n = a.len();
        let mut cur = vec![];
        let mut res = vec![];
        let mut start = 0;
        for i in 0..n {
            if a[i] == X {
                cur.push(i);
            }
            if cur.len() >= POPCNT || i == n - 1 {
                let tmp = std::mem::take(&mut cur);
                if tmp.len() >= SPARSE_LEN {
                    res.push(SelectIndexNLlInner::Sparse(tmp));
                } else {
                    res.push(SelectIndexNLlInner::Dense(start..i + 1));
                }
                start = i + 1;
            }
        }
        Self { inner: res }
    }

    pub fn select<const X: bool>(&self, i: usize, r: &RankIndexNC) -> usize {
        self.inner[i / POPCNT].select::<X>(i, i % POPCNT, r)
    }
}

enum SelectIndexNLlInner {
    Sparse(Vec<usize>),
    Dense(Range<usize>),
}

impl SelectIndexNLlInner {
    fn select<const X: bool>(
        &self,
        i: usize,
        i_rem: usize,
        r: &RankIndexNC,
    ) -> usize {
        match self {
            Self::Sparse(pos) => pos[i_rem],
            Self::Dense(Range { start, end }) => {
                let mut lo = *start;
                let mut hi = *end;
                if r.rank::<X>(lo) > i {
                    return lo;
                }
                while hi - lo > 1 {
                    let mid = lo + (hi - lo) / 2;
                    *(if r.rank::<X>(mid) <= i { &mut lo } else { &mut hi }) =
                        mid;
                }
                hi
            }
        }
    }
}

const POPCNT: usize = 64; // log(n)
const SPARSE_LEN: usize = 4096; // log(n)^2

pub type Rs01DictNLl = Rs01DictNLlParam<POPCNT, SPARSE_LEN>;

pub struct Rs01DictNLlParam<const POPCNT: usize, const SPARSE_LEN: usize> {
    rank_index: RankIndexNC,
    select1_index: SelectIndexNLl<POPCNT, SPARSE_LEN>,
    select0_index: SelectIndexNLl<POPCNT, SPARSE_LEN>,
}

impl<const POPCNT: usize, const SPARSE_LEN: usize>
    Rs01DictNLlParam<POPCNT, SPARSE_LEN>
{
    pub fn new(a: &[bool]) -> Self {
        Self {
            rank_index: RankIndexNC::new(a),
            select1_index: SelectIndexNLl::new::<true>(a),
            select0_index: SelectIndexNLl::new::<false>(a),
        }
    }
    fn rank<const X: bool>(&self, i: usize) -> usize {
        self.rank_index.rank::<X>(i)
    }
    fn select<const X: bool>(&self, i: usize) -> usize {
        if X {
            self.select1_index.select::<X>(i, &self.rank_index)
        } else {
            self.select0_index.select::<false>(i, &self.rank_index)
        }
    }

    pub fn rank1(&self, i: usize) -> usize { self.rank::<true>(i) }
    pub fn rank0(&self, i: usize) -> usize { self.rank::<false>(i) }
    pub fn select1(&self, i: usize) -> usize { self.select::<true>(i) }
    pub fn select0(&self, i: usize) -> usize { self.select::<false>(i) }
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

#[cfg(test)]
macro_rules! bitvec {
    ($lit:literal) => {
        $lit.iter()
            .filter(|&&b| matches!(b, b'0' | b'1'))
            .map(|&b| b != b'0')
            .collect::<Vec<_>>()
    };
}

#[test]
fn sanity_check_rank() {
    let a = bitvec!(b"000 010 110 000; 111 001 000 011; 000 000 010 010");
    let rs = Rs01DictNLlParam::<100, 100>::new(&a);
    let expected1 = [
        0, 0, 0, 0, 1, 1, 2, 3, 3, 3, 3, 3, 4, 5, 6, 6, 6, 7, 7, 7, 7, 7, 8, 9,
        9, 9, 9, 9, 9, 9, 9, 10, 10, 10, 11, 11,
    ];
    let actual1: Vec<_> = (0..a.len()).map(|i| rs.rank1(i)).collect();
    assert_eq!(actual1, expected1);

    let expected0 = [
        1, 2, 3, 4, 4, 5, 5, 5, 6, 7, 8, 9, 9, 9, 9, 10, 11, 11, 12, 13, 14,
        15, 15, 15, 16, 17, 18, 19, 20, 21, 22, 22, 23, 24, 24, 25,
    ];
    let actual0: Vec<_> = (0..a.len()).map(|i| rs.rank0(i)).collect();
    assert_eq!(actual0, expected0);
}

#[test]
fn sanity_check_select_dense() {
    let a = bitvec!(b"000 010 110; 000 111 001; 000 011 000");
    let ones = a.iter().filter(|&&x| x).count();
    let zeros = a.len() - ones;
    let rs = Rs01DictNLlParam::<100, 100>::new(&a);
    let expected1: Vec<_> = (0..a.len()).filter(|&i| a[i]).collect();
    let expected0: Vec<_> = (0..a.len()).filter(|&i| !a[i]).collect();
    let actual1: Vec<_> = (0..ones).map(|i| rs.select1(i)).collect();
    let actual0: Vec<_> = (0..zeros).map(|i| rs.select0(i)).collect();
    assert_eq!(actual1, expected1);
    assert_eq!(actual0, expected0);
}

#[test]
fn sanity_check_select_sparse() {
    let a = bitvec!(b"001 010 000; 000 000 110");
    let ones = a.iter().filter(|&&x| x).count();
    let zeros = a.len() - ones;
    let rs = Rs01DictNLlParam::<2, 0>::new(&a);
    let expected1: Vec<_> = (0..a.len()).filter(|&i| a[i]).collect();
    let expected0: Vec<_> = (0..a.len()).filter(|&i| !a[i]).collect();
    let actual1: Vec<_> = (0..ones).map(|i| rs.select1(i)).collect();
    let actual0: Vec<_> = (0..zeros).map(|i| rs.select0(i)).collect();
    assert_eq!(actual1, expected1);
    assert_eq!(actual0, expected0);
}
