use std::ops::{Range, RangeInclusive};

fn compress_vec_bool<const W: usize>(buf: &[bool]) -> Vec<u64> {
    let n = buf.len();
    let len = (n + W - 1) / W;
    let mut res = vec![0; len];
    for i in 0..n {
        if buf[i] {
            res[i / W] |= 1 << (i % W);
        }
    }
    res
}

const W: usize = u64::BITS as usize;

const LG_N: usize = 24;

const LEAF_LEN: usize = LG_N / 2;
const POW2_SMALL: usize = 1 << LEAF_LEN;
const RANK_LOOKUP: [[u16; LEAF_LEN]; POW2_SMALL] =
    rank_lookup::<LEAF_LEN, POW2_SMALL>();

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

pub struct RankIndex<const LARGE: usize, const SMALL: usize> {
    buf: Vec<u64>,

    /// $`\log(n)^2`$-bit blocks.
    large: Vec<u16>,

    /// $`O(\tfrac12\log(n))`$-bit blocks.
    small: Vec<u16>,
}

impl<const LARGE: usize, const SMALL: usize> RankIndex<LARGE, SMALL> {
    // `a` should be the value returned by `compress_vec_bool::<SMALL>(_)`
    pub fn new(a: Vec<u64>) -> Self {
        let count: Vec<_> =
            a.iter().map(|&ai| RANK_LOOKUP[ai as usize][SMALL]).collect();

        let small: Vec<_> = count
            .chunks(LARGE / SMALL)
            .flat_map(|c| {
                c.iter()
                    .scan(0, |acc, x| Some(std::mem::replace(acc, *acc + x)))
            })
            .collect();

        let large: Vec<_> = count
            .chunks(LARGE / SMALL)
            .map(|c| c.iter().sum::<u16>())
            .scan(0, |acc, x| Some(std::mem::replace(acc, *acc + x)))
            .collect();

        Self { buf: a, large, small }
    }

    pub fn rank(&self, n: usize) -> usize {
        let large = self.large[n / LARGE];
        let small = self.small[n / SMALL];
        let rem = RANK_LOOKUP[self.buf[n / SMALL] as usize][n % SMALL];
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

impl SimpleBitVec {
    fn new() -> Self { Self { buf: vec![], len: 0 } }

    fn len(&self) -> usize { self.len }
    fn is_empty(&self) -> bool { self.len == 0 }

    fn get(&self, Range { start, end }: Range<usize>) -> u64 {
        assert!(end - start <= 64);
        assert!(end <= self.len);

        let mask = !(!0 << (end - start));
        if start == end {
            0
        } else if start % W == 0 {
            self.buf[start / W] & mask
        } else if end <= (start / W + 1) * W {
            self.buf[start / W] >> (start % W) & mask
        } else {
            (self.buf[start / W] >> (start % W)
                | self.buf[end / W] << (W - start % W))
                & mask
        }
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

    fn push_vec(&mut self, mut other: Self) {
        if other.is_empty() {
            // nothing to do
        } else if self.len % W == 0 {
            self.buf.append(&mut other.buf);
            self.len += other.len;
        } else {
            // `self.len` is updated in `self.push(..)`
            for &w in &other.buf[..other.len / W] {
                self.push(w, W);
            }
            self.push(other.buf[other.len / W], other.len % W);
        }
    }

    fn chunks(&self, size: usize) -> impl Iterator<Item = u64> + '_ {
        (0..self.len)
            .step_by(size)
            .map(move |i| self.get(i..self.len.min(i + size)))
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
    Dense(Vec<SimpleBitVec>, SimpleBitVec),
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
    ds: Vec<(
        usize,
        SelectIndexInner<
            BIT_PATTERNS,
            POPCNT,
            LG2_POPCNT,
            LEAF_LEN,
            POW2_LEAF_LEN,
            SPARSE_LEN,
            BRANCH,
        >,
    )>,
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
    fn new(a: Vec<usize>, range: RangeInclusive<usize>) -> Self {
        let start = *range.start();
        let end = *range.end() + 1;
        if end - start >= SPARSE_LEN {
            Self::Sparse(a)
        } else {
            let len = end - start;
            let mut tmp = vec![0_u64; (len + W - 1) / W];
            for ai in a.iter().map(|&ai| ai - start) {
                tmp[ai / W] |= 1 << (ai % W);
            }
            Self::new_dense(tmp, len)
        }
    }
    fn new_dense(a: Vec<u64>, len: usize) -> Self {
        let a = SimpleBitVec::from((a, len));
        let leaf = {
            let mut leaf = SimpleBitVec::new();
            for i in 0..(len + LEAF_LEN - 1) / LEAF_LEN {
                let w = a.get(i * LEAF_LEN..len.min((i + 1) * LEAF_LEN));
                leaf.push(
                    RANK_LOOKUP[w as usize][LEAF_LEN - 1] as u64,
                    LG2_POPCNT,
                );
            }
            leaf
        };

        let mut tree = vec![];
        let mut last = leaf;
        while last.len() > LG2_POPCNT {
            let mut cur = SimpleBitVec::new();
            let tmp = last;
            {
                let mut it = tmp.chunks(LG2_POPCNT);
                while let Some(mut sum) = it.next() {
                    sum += (1..BRANCH).filter_map(|_| it.next()).sum::<u64>();
                    cur.push(sum, LG2_POPCNT);
                }
            }
            tree.push(tmp);
            last = cur;
        }
        tree.reverse();
        Self::Dense(tree, a)
    }

    fn select(&self, i: usize) -> usize {
        match self {
            Self::Sparse(index) => index[i],
            Self::Dense(tree, buf) => {
                let mut i = i;
                let mut cur = 0;
                let mut off = 0;
                let len = LG2_POPCNT * BRANCH;
                for level in tree {
                    let w = level.get(cur..level.len().min(cur + len)) as usize;
                    let (br, count) = Self::TREE[w][i];
                    cur = (cur + LG2_POPCNT * br as usize) * BRANCH;
                    off = off * BRANCH + br as usize;
                    i -= count as usize;
                }

                let start = cur / (BRANCH * LG2_POPCNT) * LEAF_LEN;
                let end = buf.len().min(start + LEAF_LEN);
                let leaf = buf.get(start..end);

                off * LEAF_LEN + Self::WORD[leaf as usize][i] as usize
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
    pub fn new<const X: bool>(a: &[bool]) -> Self {
        let n = a.len();
        let mut cur = vec![];
        let mut res = vec![];
        let mut start = 0;
        for i in 0..n {
            if a[i] == X {
                cur.push(i);
            }
            if cur.len() == POPCNT || i == n - 1 {
                let tmp = std::mem::take(&mut cur);
                res.push((start, SelectIndexInner::new(tmp, start..=i)));
                start = i + 1;
            }
        }
        Self { ds: res }
    }

    pub fn select(&self, i: usize) -> usize {
        let ds = &self.ds[i / POPCNT];
        ds.0 + ds.1.select(i % POPCNT)
    }
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
    let b = compress_vec_bool::<3>(&a);
    let rp = RankIndex::<12, 3>::new(b.clone());
    let expected = [
        0, 0, 0, 0, 1, 1, 2, 3, 3, 3, 3, 3, 4, 5, 6, 6, 6, 7, 7, 7, 7, 7, 8, 9,
        9, 9, 9, 9, 9, 9, 9, 10, 10, 10, 11, 11,
    ];
    let actual: Vec<_> = (0..a.len()).map(|i| rp.rank(i)).collect();
    assert_eq!(actual, expected);
}

#[test]
fn sanity_check_select() {
    let a = bitvec!(b"000 010 110; 000 111 001; 000 011 000");
    let ones = a.iter().filter(|&&x| x).count();
    let sp = SelectIndex::<4096, 12, 4, 3, 8, 100, 3>::new::<true>(&a);
    let expected = [4, 6, 7, 12, 13, 14, 17, 22, 23];
    let actual: Vec<_> = (0..ones).map(|i| sp.select(i)).collect();
    assert_eq!(actual, expected);
}
