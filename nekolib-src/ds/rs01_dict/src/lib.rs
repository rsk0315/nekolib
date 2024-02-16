use std::ops::{Range, RangeInclusive};

const RANK_BLOCK: usize = 64;
const RANK_POPCNT: usize = 16;
const RANK_BIT_PATTERNS: usize = 1 << RANK_POPCNT;

const SELECT_LG2_POPCNT: usize = 3;
const SELECT_POPCNT: usize = !(!0 << SELECT_LG2_POPCNT);
const SELECT_LEAF_LEN: usize = 12;
const SELECT_POW2_LEAF_LEN: usize = 1 << SELECT_LEAF_LEN;
const SELECT_SPARSE_LEN: usize = 27648;
const SELECT_BRANCH: usize = 4;
const SELECT_BIT_PATTERNS: usize = 1 << (SELECT_BRANCH * SELECT_LG2_POPCNT);

pub type Rs01Dict = Rs01DictParam<
    RANK_BIT_PATTERNS,
    RANK_BLOCK,
    RANK_POPCNT,
    SELECT_BIT_PATTERNS,
    SELECT_POPCNT,
    SELECT_LG2_POPCNT,
    SELECT_LEAF_LEN,
    SELECT_POW2_LEAF_LEN,
    SELECT_SPARSE_LEN,
    SELECT_BRANCH,
>;

pub struct Rs01DictParam<
    const RANK_BIT_PATTERNS: usize,
    const RANK_LARGE: usize,
    const RANK_POPCNT: usize,
    const SELECT_BIT_PATTERNS: usize,
    const SELECT_POPCNT: usize,
    const SELECT_LG2_POPCNT: usize,
    const SELECT_LEAF_LEN: usize,
    const SELECT_POW2_LEAF_LEN: usize,
    const SELECT_SPARSE_LEN: usize,
    const SELECT_BRANCH: usize,
> {
    buf: SimpleBitVec,
    rank_index: RankIndex<RANK_BIT_PATTERNS, RANK_LARGE, RANK_POPCNT>,
    select1_index: SelectIndex<
        SELECT_BIT_PATTERNS,
        SELECT_POPCNT,
        SELECT_LG2_POPCNT,
        SELECT_LEAF_LEN,
        SELECT_POW2_LEAF_LEN,
        SELECT_SPARSE_LEN,
        SELECT_BRANCH,
    >,
    select0_index: SelectIndex<
        SELECT_BIT_PATTERNS,
        SELECT_POPCNT,
        SELECT_LG2_POPCNT,
        SELECT_LEAF_LEN,
        SELECT_POW2_LEAF_LEN,
        SELECT_SPARSE_LEN,
        SELECT_BRANCH,
    >,
}

impl<
    const RANK_BIT_PATTERNS: usize,
    const RANK_LARGE: usize,
    const RANK_POPCNT: usize,
    const SELECT_BIT_PATTERNS: usize,
    const SELECT_POPCNT: usize,
    const SELECT_LG2_POPCNT: usize,
    const SELECT_LEAF_LEN: usize,
    const SELECT_POW2_LEAF_LEN: usize,
    const SELECT_SPARSE_LEN: usize,
    const SELECT_BRANCH: usize,
>
    Rs01DictParam<
        RANK_BIT_PATTERNS,
        RANK_LARGE,
        RANK_POPCNT,
        SELECT_BIT_PATTERNS,
        SELECT_POPCNT,
        SELECT_LG2_POPCNT,
        SELECT_LEAF_LEN,
        SELECT_POW2_LEAF_LEN,
        SELECT_SPARSE_LEN,
        SELECT_BRANCH,
    >
{
    pub fn new(a: &[bool]) -> Self {
        let buf = SimpleBitVec::from(a);
        let rank_index =
            RankIndex::<RANK_BIT_PATTERNS, RANK_LARGE, RANK_POPCNT>::new(&buf);
        let select1_index = SelectIndex::<
            SELECT_BIT_PATTERNS,
            SELECT_POPCNT,
            SELECT_LG2_POPCNT,
            SELECT_LEAF_LEN,
            SELECT_POW2_LEAF_LEN,
            SELECT_SPARSE_LEN,
            SELECT_BRANCH,
        >::new::<true>(a, &buf);
        let select0_index = SelectIndex::<
            SELECT_BIT_PATTERNS,
            SELECT_POPCNT,
            SELECT_LG2_POPCNT,
            SELECT_LEAF_LEN,
            SELECT_POW2_LEAF_LEN,
            SELECT_SPARSE_LEN,
            SELECT_BRANCH,
        >::new::<false>(a, &buf);
        Self { buf, rank_index, select1_index, select0_index }
    }

    pub fn rank1(&self, i: usize) -> usize {
        self.rank_index.rank(i, &self.buf)
    }
    pub fn rank0(&self, i: usize) -> usize { i + 1 - self.rank1(i) }

    pub fn select1(&self, i: usize) -> usize {
        self.select1_index.select::<true>(i, &self.buf)
    }
    pub fn select0(&self, i: usize) -> usize {
        self.select0_index.select::<false>(i, &self.buf)
    }
}

trait RankLookup<
    const BIT_PATTERNS: usize,
    const LARGE: usize,
    const POPCNT: usize,
>
{
    const WORD: [[u16; POPCNT]; BIT_PATTERNS];
}

impl<const BIT_PATTERNS: usize, const LARGE: usize, const POPCNT: usize>
    RankLookup<BIT_PATTERNS, LARGE, POPCNT>
    for RankIndex<BIT_PATTERNS, LARGE, POPCNT>
{
    const WORD: [[u16; POPCNT]; BIT_PATTERNS] =
        rank_lookup::<POPCNT, BIT_PATTERNS>();
}

const W: usize = u64::BITS as usize;

const fn rank_lookup<
    const MAX_LEN: usize,      // log(n)/2
    const BIT_PATTERNS: usize, // sqrt(n)
>() -> [[u16; MAX_LEN]; BIT_PATTERNS] {
    let mut table = [[0; MAX_LEN]; BIT_PATTERNS];
    let mut i = 0;
    while i < BIT_PATTERNS {
        table[i][0] = (i & 1) as u16;
        let mut j = 1;
        while j < MAX_LEN {
            table[i][j] = table[i][j - 1] + (i >> j & 1) as u16;
            j += 1;
        }
        i += 1;
    }
    table
}

const fn select_lookup_tree<
    const BIT_PATTERNS: usize, // 2^(branch * large)
    const BRANCH: usize,       // sqrt(log(n))
    const POPCNT: usize,       // log(n)^2
    const LG2_POPCNT: usize,   // O(log(log(n)))
>() -> [[(u16, u16); POPCNT]; BIT_PATTERNS] {
    let mut table = [[(0, 0); POPCNT]; BIT_PATTERNS];
    let mut i = 0;
    while i < BIT_PATTERNS {
        let mut j = 0;
        let mut index = 0;
        while j < BRANCH {
            // [0011, 0100, 0010] (0b_0010_0100_0011)
            // [0, 0, 0, 1, 1, 1, 1, 2, 2, 3, ...]
            let count = i >> (j * LG2_POPCNT) & !(!0 << LG2_POPCNT);
            let mut k = 0;
            while k < count && index < POPCNT {
                table[i][index] = (j as u16, (index - k) as u16);
                index += 1;
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }
    table
}

const fn select_lookup_word<
    const BIT_PATTERNS: usize, // 2^leaflen
    const LEAF_LEN: usize,     // log(n)/2
>() -> [[u16; LEAF_LEN]; BIT_PATTERNS] {
    let mut table = [[0; LEAF_LEN]; BIT_PATTERNS];
    let mut i = 0;
    while i < BIT_PATTERNS {
        let mut j = 0;
        let mut count = 0;
        while j < LEAF_LEN {
            if i >> j & 1 != 0 {
                table[i][count] = j as u16;
                count += 1;
            }
            j += 1;
        }
        i += 1;
    }
    table
}

pub struct RankIndex<
    const BIT_PATTERNS: usize,
    const LARGE: usize,
    const POPCNT: usize,
> {
    /// $`\log(n)^2`$-bit blocks.
    large: Vec<u64>,

    /// $`O(\tfrac12\log(n))`$-bit blocks.
    small: Vec<u16>,
}

impl<const BIT_PATTERNS: usize, const LARGE: usize, const POPCNT: usize>
    RankIndex<BIT_PATTERNS, LARGE, POPCNT>
{
    fn new(a: &SimpleBitVec) -> Self {
        let count: Vec<_> = a
            .chunks::<true>(POPCNT)
            .map(|ai| Self::WORD[ai as usize][POPCNT - 1])
            .collect();

        let small: Vec<_> = count
            .chunks(LARGE / POPCNT)
            .flat_map(|c| {
                c.iter()
                    .scan(0, |acc, x| Some(std::mem::replace(acc, *acc + x)))
            })
            .collect();

        let large: Vec<_> = count
            .chunks(LARGE / POPCNT)
            .map(|c| c.iter().map(|&x| x as u64).sum::<u64>())
            .scan(0, |acc, x| Some(std::mem::replace(acc, *acc + x)))
            .collect();

        Self { large, small }
    }

    fn rank(&self, n: usize, b: &SimpleBitVec) -> usize {
        let large = self.large[n / LARGE];
        let small = self.small[n / POPCNT];
        let rem = Self::WORD[b
            .get::<true>(n / POPCNT * POPCNT..(n / POPCNT + 1) * POPCNT)
            as usize][n % POPCNT];
        large as usize + small as usize + rem as usize
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SimpleBitVec {
    buf: Vec<u64>,
    len: usize,
}

impl From<(Vec<u64>, usize)> for SimpleBitVec {
    fn from((buf, len): (Vec<u64>, usize)) -> Self { Self { buf, len } }
}

impl From<&[bool]> for SimpleBitVec {
    fn from(a: &[bool]) -> Self {
        let len = a.len();
        let n = (len + W - 1) / W;
        let mut buf = vec![0; n];
        for i in 0..len {
            if a[i] {
                buf[i / W] |= 1 << (i % W);
            }
        }
        Self { buf, len }
    }
}

impl SimpleBitVec {
    fn new() -> Self { Self { buf: vec![], len: 0 } }

    fn len(&self) -> usize { self.len }

    fn get<const X: bool>(&self, Range { start, end }: Range<usize>) -> u64 {
        assert!(end - start <= 64);
        assert!(end <= self.len);

        let mask = !(!0 << (end - start));
        let res = if start == end {
            0
        } else if start % W == 0 {
            self.buf[start / W] & mask
        } else if end <= (start / W + 1) * W {
            self.buf[start / W] >> (start % W) & mask
        } else {
            (self.buf[start / W] >> (start % W)
                | self.buf[end / W] << (W - start % W))
                & mask
        };
        if X { res } else { !res & mask }
    }

    fn push(&mut self, w: u64, len: usize) {
        assert!(len == 0 || w & (!0 << len) == 0);

        if len == 0 {
            // nothing to do
        } else if self.len % W == 0 {
            // including the case `self.buf.is_empty()`
            self.buf.push(w);
        } else {
            self.buf[self.len / W] |= w << (self.len % W);
            if self.len % W + len > W {
                self.buf.push(w >> (W - self.len % W));
            }
        }
        self.len += len;
    }

    fn chunks<const X: bool>(
        &self,
        size: usize,
    ) -> impl Iterator<Item = u64> + '_ {
        (0..self.len)
            .step_by(size)
            .map(move |i| self.get::<X>(i..self.len.min(i + size)))
    }
}

enum SelectIndexInner<
    const BIT_PATTERNS: usize,
    const POPCNT: usize,
    const LG2_POPCNT: usize,
    const LEAF_LEN: usize,
    const POW2_LEAF_LEN: usize,
    const SPARSE_LEN: usize,
    const BRANCH: usize,
> {
    /// at least $`\log(n)^4`$-bit blocks.
    Sparse(Vec<usize>),

    /// less than $`\log(n)^4`$-bit blocks.
    Dense(Vec<SimpleBitVec>, usize),
}

pub struct SelectIndex<
    const BIT_PATTERNS: usize,
    const POPCNT: usize,
    const LG2_POPCNT: usize,
    const LEAF_LEN: usize,
    const POW2_LEAF_LEN: usize,
    const SPARSE_LEN: usize,
    const BRANCH: usize,
> {
    inner: Vec<
        SelectIndexInner<
            BIT_PATTERNS,
            POPCNT,
            LG2_POPCNT,
            LEAF_LEN,
            POW2_LEAF_LEN,
            SPARSE_LEN,
            BRANCH,
        >,
    >,
}

trait SelectLookup<
    const BIT_PATTERNS: usize,
    const POPCNT: usize,
    const POW2_LEAF_LEN: usize,
    const LEAF_LEN: usize,
>
{
    const TREE: [[(u16, u16); POPCNT]; BIT_PATTERNS];
    const WORD: [[u16; LEAF_LEN]; POW2_LEAF_LEN];
}

impl<
    const BIT_PATTERNS: usize,
    const POPCNT: usize,
    const LG2_POPCNT: usize,
    const LEAF_LEN: usize,
    const POW2_LEAF_LEN: usize,
    const SPARSE_LEN: usize,
    const BRANCH: usize,
> SelectLookup<BIT_PATTERNS, POPCNT, POW2_LEAF_LEN, LEAF_LEN>
    for SelectIndexInner<
        BIT_PATTERNS,
        POPCNT,
        LG2_POPCNT,
        LEAF_LEN,
        POW2_LEAF_LEN,
        SPARSE_LEN,
        BRANCH,
    >
{
    const TREE: [[(u16, u16); POPCNT]; BIT_PATTERNS] =
        select_lookup_tree::<BIT_PATTERNS, BRANCH, POPCNT, LG2_POPCNT>();
    const WORD: [[u16; LEAF_LEN]; POW2_LEAF_LEN] =
        select_lookup_word::<POW2_LEAF_LEN, LEAF_LEN>();
}

impl<
    const BIT_PATTERNS: usize,
    const POPCNT: usize,
    const LG2_POPCNT: usize,
    const LEAF_LEN: usize,
    const POW2_LEAF_LEN: usize,
    const SPARSE_LEN: usize,
    const BRANCH: usize,
>
    SelectIndexInner<
        BIT_PATTERNS,
        POPCNT,
        LG2_POPCNT,
        LEAF_LEN,
        POW2_LEAF_LEN,
        SPARSE_LEN,
        BRANCH,
    >
{
    fn new<const X: bool>(
        a: Vec<usize>,
        range: RangeInclusive<usize>,
        b: &SimpleBitVec,
    ) -> Self {
        let start = *range.start();
        let end = *range.end() + 1;
        if end - start >= SPARSE_LEN {
            Self::Sparse(a)
        } else {
            Self::new_dense::<X>(b, start..end)
        }
    }
    fn new_dense<const X: bool>(
        b: &SimpleBitVec,
        Range { start, end }: Range<usize>,
    ) -> Self {
        let rl = &RankIndex::<POW2_LEAF_LEN, 0, LEAF_LEN>::WORD;
        let len = end - start;

        let leaf = {
            let mut leaf = SimpleBitVec::new();
            for i in 0..(len + LEAF_LEN - 1) / LEAF_LEN {
                let w = b.get::<X>(
                    start + i * LEAF_LEN..start + len.min((i + 1) * LEAF_LEN),
                );
                leaf.push(rl[w as usize][LEAF_LEN - 1] as u64, LG2_POPCNT);
            }
            leaf
        };

        let mut tree = vec![];
        let mut last = leaf;
        while last.len() > LG2_POPCNT {
            let mut cur = SimpleBitVec::new();
            let tmp = last;
            {
                let mut it = tmp.chunks::<true>(LG2_POPCNT);
                while let Some(mut sum) = it.next() {
                    sum += (1..BRANCH).filter_map(|_| it.next()).sum::<u64>();
                    cur.push(sum, LG2_POPCNT);
                }
            }
            tree.push(tmp);
            last = cur;
        }
        tree.reverse();
        Self::Dense(tree, start)
    }

    fn select<const X: bool>(&self, i: usize, b: &SimpleBitVec) -> usize {
        match self {
            Self::Sparse(index) => index[i],
            Self::Dense(tree, start) => {
                let mut i = i;
                let mut cur = 0;
                let mut off = 0;
                let len = LG2_POPCNT * BRANCH;
                for level in tree {
                    let w = level.get::<true>(cur..level.len().min(cur + len))
                        as usize;
                    let (br, count) = Self::TREE[w][i];
                    cur = (cur + LG2_POPCNT * br as usize) * BRANCH;
                    off = off * BRANCH + br as usize;
                    i -= count as usize;
                }

                let bstart = start + cur / (BRANCH * LG2_POPCNT) * LEAF_LEN;
                let bend = b.len().min(bstart + LEAF_LEN);
                let leaf = b.get::<X>(bstart..bend);
                start + off * LEAF_LEN + Self::WORD[leaf as usize][i] as usize
            }
        }
    }
}

impl<
    const BIT_PATTERNS: usize,
    const POPCNT: usize,
    const LG2_POPCNT: usize,
    const LEAF_LEN: usize,
    const POW2_LEAF_LEN: usize,
    const SPARSE_LEN: usize,
    const BRANCH: usize,
>
    SelectIndex<
        BIT_PATTERNS,
        POPCNT,
        LG2_POPCNT,
        LEAF_LEN,
        POW2_LEAF_LEN,
        SPARSE_LEN,
        BRANCH,
    >
{
    fn new<const X: bool>(a: &[bool], b: &SimpleBitVec) -> Self {
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
                res.push(SelectIndexInner::new::<X>(tmp, start..=i, b));
                start = i + 1;
            }
        }
        Self { inner: res }
    }

    fn select<const X: bool>(&self, i: usize, b: &SimpleBitVec) -> usize {
        self.inner[i / POPCNT].select::<X>(i % POPCNT, b)
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
fn test_rank_lookup() {
    let table = rank_lookup::<3, 8>();

    assert_eq!(&table[0b000][0..3], [0, 0, 0]);
    assert_eq!(&table[0b100][0..3], [0, 0, 1]);
    assert_eq!(&table[0b010][0..3], [0, 1, 1]);
    assert_eq!(&table[0b110][0..3], [0, 1, 2]);
    assert_eq!(&table[0b001][0..3], [1, 1, 1]);
    assert_eq!(&table[0b101][0..3], [1, 1, 2]);
    assert_eq!(&table[0b011][0..3], [1, 2, 2]);
    assert_eq!(&table[0b111][0..3], [1, 2, 3]);
}

#[test]
fn test_select_lookup() {
    let table = select_lookup_tree::<4096, 3, 16, 4>();
    let tmp: [_; 9] = table[0b_0010_0100_0011][0..9].try_into().unwrap();
    assert_eq!(tmp.map(|x| x.0), [0, 0, 0, 1, 1, 1, 1, 2, 2]);
    assert_eq!(tmp.map(|x| x.1), [0, 0, 0, 3, 3, 3, 3, 7, 7]);
}

#[test]
fn sanity_check_rank() {
    let a = bitvec!(b"000 010 110 000; 111 001 000 011; 000 000 010 010");
    let rs = Rs01DictParam::<4096, 12, 3, 4096, 12, 4, 3, 8, 100, 3>::new(&a);
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
    let rs = Rs01DictParam::<4096, 12, 3, 4096, 12, 4, 3, 8, 100, 3>::new(&a);
    let expected = [4, 6, 7, 12, 13, 14, 17, 22, 23];
    let actual: Vec<_> = (0..ones).map(|i| rs.select1(i)).collect();
    assert_eq!(actual, expected);
}

#[test]
fn sanity_check_select_sparse() {
    let a = bitvec!(b"001 010 000; 000 000 110");
    let ones = a.iter().filter(|&&x| x).count();
    let rs = Rs01DictParam::<4096, 12, 3, 2, 1, 1, 1, 2, 0, 1>::new(&a);
    let expected = [2, 4, 15, 16];
    let actual: Vec<_> = (0..ones).map(|i| rs.select1(i)).collect();
    assert_eq!(actual, expected);
}

#[test]
fn bench() {
    type Rs = Rs01DictParam<4096, 12, 3, 4096, 12, 4, 3, 8, 100, 3>;

    eprintln!("{:?}", &RankIndex::<8, 0, 3>::WORD[0]);

    let w = 0x_3046_2FB7_58C1_EDA9_u64;
    let a: Vec<_> = (0..64).map(|i| w >> i & 1 != 0).collect();

    let rs = Rs::new(&a);

    eprintln!("{w:064b}");

    for i in 0..32 {
        eprintln!("select1({i}) -> {}", rs.select1(i));
    }
    for i in 0..32 {
        eprintln!("select0({i}) -> {}", rs.select0(i));
    }
}
