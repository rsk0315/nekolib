use std::ops::{Range, RangeBounds, RangeInclusive};

const W: usize = u64::BITS as usize;

const RANK_LARGE_LEN: usize = 1024; // (1/4) log(n)^2
const RANK_SMALL_LEN: usize = 16; // (1/2) log(n)/2
const RANK_BIT_PATTERNS: usize = 1 << RANK_SMALL_LEN;

const SELECT_SMALL_LEN: usize = 15; // (1/2) log(n)/2
const SELECT_LARGE_SPARSE_LEN: usize = 12946;
const SELECT_LARGE_POPCNT: usize = 17;
const SELECT_LARGE_NODE_LEN: usize = 4;
const SELECT_LARGE_BRANCH: usize = 4;
const SELECT_WORD_BIT_PATTERNS: usize = 1 << SELECT_SMALL_LEN;
const SELECT_TREE_BIT_PATTERNS: usize =
    1 << (SELECT_LARGE_NODE_LEN * SELECT_LARGE_BRANCH);

const _ASSERTION: () = {
    let popcnt = SELECT_LARGE_POPCNT;
    let node_len = SELECT_LARGE_NODE_LEN;
    let branch = SELECT_LARGE_BRANCH;

    let node_popcnt = !(!0 << node_len);
    if node_popcnt * branch < popcnt {
        panic!();
    }
};

pub type Rs01Dict = Rs01DictGenerics<
    RANK_LARGE_LEN,
    RANK_SMALL_LEN,
    RANK_BIT_PATTERNS,
    SELECT_SMALL_LEN,
    SELECT_LARGE_SPARSE_LEN,
    SELECT_LARGE_POPCNT,
    SELECT_LARGE_NODE_LEN,
    SELECT_LARGE_BRANCH,
    SELECT_WORD_BIT_PATTERNS,
    SELECT_TREE_BIT_PATTERNS,
>;

pub struct Rs01DictGenerics<
    const RANK_LARGE_LEN: usize,
    const RANK_SMALL_LEN: usize,
    const RANK_BIT_PATTERNS: usize,
    const SELECT_SMALL_LEN: usize,
    const SELECT_LARGE_SPARSE_LEN: usize,
    const SELECT_LARGE_POPCNT: usize,
    const SELECT_LARGE_NODE_LEN: usize,
    const SELECT_LARGE_BRANCH: usize,
    const SELECT_WORD_BIT_PATTERNS: usize,
    const SELECT_TREE_BIT_PATTERNS: usize,
> {
    buf: SimpleBitVec,
    rank_index: RankIndex<RANK_LARGE_LEN, RANK_SMALL_LEN, RANK_BIT_PATTERNS>,
    select1_index: SelectIndex<
        SELECT_SMALL_LEN,
        SELECT_LARGE_SPARSE_LEN,
        SELECT_LARGE_POPCNT,
        SELECT_LARGE_NODE_LEN,
        SELECT_LARGE_BRANCH,
        SELECT_WORD_BIT_PATTERNS,
        SELECT_TREE_BIT_PATTERNS,
    >,
    select0_index: SelectIndex<
        SELECT_SMALL_LEN,
        SELECT_LARGE_SPARSE_LEN,
        SELECT_LARGE_POPCNT,
        SELECT_LARGE_NODE_LEN,
        SELECT_LARGE_BRANCH,
        SELECT_WORD_BIT_PATTERNS,
        SELECT_TREE_BIT_PATTERNS,
    >,
}

struct SimpleBitVec {
    buf: Vec<u64>,
    len: usize,
}

struct RankIndex<
    const LARGE_LEN: usize,
    const SMALL_LEN: usize,
    const BIT_PATTERNS: usize,
> {
    large: Vec<u32>,
    small: Vec<u16>,
}

struct SelectIndex<
    const SMALL_LEN: usize,
    const LARGE_SPARSE_LEN: usize,
    const LARGE_POPCNT: usize,
    const LARGE_NODE_LEN: usize,
    const LARGE_BRANCH: usize,
    const WORD_BIT_PATTERNS: usize,
    const TREE_BIT_PATTERNS: usize,
> {
    inner: Vec<
        SelectIndexInner<
            SMALL_LEN,
            LARGE_SPARSE_LEN,
            LARGE_POPCNT,
            LARGE_NODE_LEN,
            LARGE_BRANCH,
            WORD_BIT_PATTERNS,
            TREE_BIT_PATTERNS,
        >,
    >,
}

enum SelectIndexInner<
    const SMALL_LEN: usize,
    const LARGE_SPARSE_LEN: usize,
    const LARGE_POPCNT: usize,
    const LARGE_NODE_LEN: usize,
    const LARGE_BRANCH: usize,
    const WORD_BIT_PATTERNS: usize,
    const TREE_BIT_PATTERNS: usize,
> {
    Sparse(Vec<usize>),
    Dense(SimpleBitVec, usize),
}

trait RankLookup<const SMALL_LEN: usize, const BIT_PATTERNS: usize> {
    const WORD: [[u8; SMALL_LEN]; BIT_PATTERNS];
}

trait SelectLookup<
    const NODE_LEN: usize,
    const POPCNT: usize,
    const TREE_BIT_PATTERNS: usize,
    const SMALL_LEN: usize,
    const WORD_BIT_PATTERNS: usize,
>
{
    const TREE: [[(u8, u8); POPCNT]; TREE_BIT_PATTERNS];
    const WORD: [[u8; SMALL_LEN]; WORD_BIT_PATTERNS];
}

const fn rank_lookup<const SMALL_LEN: usize, const BIT_PATTERNS: usize>()
-> [[u8; SMALL_LEN]; BIT_PATTERNS] {
    let mut table = [[0; SMALL_LEN]; BIT_PATTERNS];
    let mut i = 0;
    while i < BIT_PATTERNS {
        table[i][0] = (i & 1) as _;
        let mut j = 1;
        while j < SMALL_LEN {
            table[i][j] = table[i][j - 1] + (i >> j & 1) as u8;
            j += 1;
        }
        i += 1;
    }
    table
}

#[warn(long_running_const_eval)]
const fn select_tree_lookup<
    const NODE_LEN: usize,
    const POPCNT: usize,
    const BRANCH: usize,
    const BIT_PATTERNS: usize,
>() -> [[(u8, u8); POPCNT]; BIT_PATTERNS] {
    let mut table = [[(0, 0); POPCNT]; BIT_PATTERNS];
    let mut i = 0;
    while i < BIT_PATTERNS {
        let mut j = 0;
        let mut index = 0;
        while j < BRANCH {
            // [011, 100, 010] (0b_010_100_011)
            // [0, 0, 0, 1, 1, 1, 1, 2, 2]
            let count = i >> (j * NODE_LEN) & !(!0 << NODE_LEN);
            let mut k = 0;
            while k < count && index < POPCNT {
                table[i][index] = (j as _, (index - k) as _);
                index += 1;
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }
    table
}

const fn select_word_lookup<
    const SMALL_LEN: usize,
    const BIT_PATTERNS: usize,
>() -> [[u8; SMALL_LEN]; BIT_PATTERNS] {
    let mut table = [[0; SMALL_LEN]; BIT_PATTERNS];
    let mut i = 0;
    while i < BIT_PATTERNS {
        let mut j = 0;
        let mut count = 0;
        while j < SMALL_LEN {
            if i >> j & 1 != 0 {
                table[i][count] = j as _;
                count += 1;
            }
            j += 1;
        }
        i += 1;
    }
    table
}

impl<
    const RANK_LARGE_LEN: usize,
    const RANK_SMALL_LEN: usize,
    const RANK_BIT_PATTERNS: usize,
    const SELECT_SMALL_LEN: usize,
    const SELECT_LARGE_SPARSE_LEN: usize,
    const SELECT_LARGE_POPCNT: usize,
    const SELECT_LARGE_NODE_LEN: usize,
    const SELECT_LARGE_BRANCH: usize,
    const SELECT_WORD_BIT_PATTERNS: usize,
    const SELECT_TREE_BIT_PATTERNS: usize,
>
    Rs01DictGenerics<
        RANK_LARGE_LEN,
        RANK_SMALL_LEN,
        RANK_BIT_PATTERNS,
        SELECT_SMALL_LEN,
        SELECT_LARGE_SPARSE_LEN,
        SELECT_LARGE_POPCNT,
        SELECT_LARGE_NODE_LEN,
        SELECT_LARGE_BRANCH,
        SELECT_WORD_BIT_PATTERNS,
        SELECT_TREE_BIT_PATTERNS,
    >
{
    pub fn new(a: &[bool]) -> Self {
        let buf = SimpleBitVec::from(a);
        let rank_index = RankIndex::<
            RANK_LARGE_LEN,
            RANK_SMALL_LEN,
            RANK_BIT_PATTERNS,
        >::new(&buf);
        let select1_index = SelectIndex::<
            SELECT_SMALL_LEN,
            SELECT_LARGE_SPARSE_LEN,
            SELECT_LARGE_POPCNT,
            SELECT_LARGE_NODE_LEN,
            SELECT_LARGE_BRANCH,
            SELECT_WORD_BIT_PATTERNS,
            SELECT_TREE_BIT_PATTERNS,
        >::new::<true>(&buf);
        let select0_index = SelectIndex::<
            SELECT_SMALL_LEN,
            SELECT_LARGE_SPARSE_LEN,
            SELECT_LARGE_POPCNT,
            SELECT_LARGE_NODE_LEN,
            SELECT_LARGE_BRANCH,
            SELECT_WORD_BIT_PATTERNS,
            SELECT_TREE_BIT_PATTERNS,
        >::new::<false>(&buf);
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

    fn count<const X: bool>(&self, range: impl RangeBounds<usize>) -> usize {
        todo!()
    }

    pub fn count1(&self, range: impl RangeBounds<usize>) -> usize {
        self.count::<true>(range)
    }
    pub fn count0(&self, range: impl RangeBounds<usize>) -> usize {
        self.count::<false>(range)
    }
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

    fn get_single(&self, i: usize) -> bool {
        debug_assert!(i < self.len);
        self.buf[i / W] >> (i % W) & 1 != 0
    }

    fn get<const X: bool>(&self, Range { start, end }: Range<usize>) -> u64 {
        debug_assert!(end - start <= 64);
        debug_assert!(end <= self.len);

        let mask = !(!0 << (end - start));
        let res = if start == end {
            0
        } else if start % W == 0 {
            self.buf[start / W] & mask
        } else if end <= (start / W + 1) * W {
            self.buf[start / W] >> (start % W)
        } else {
            self.buf[start / W] >> (start % W)
                | self.buf[end / W] << (W - start % W)
        };
        (if X { res } else { !res }) & mask
    }

    fn push(&mut self, w: u64, len: usize) {
        assert!(len == W || w & (!0 << len) == 0);

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

    fn push_vec(&mut self, other: Self) {
        for (k, &w) in other.buf.iter().enumerate() {
            let il = k * W;
            let ir = other.len.min(il + W);
            self.push(w, ir - il);
        }
    }

    fn pad_zero(&mut self, new_len: usize) {
        if new_len <= self.len {
            return;
        }
        let n = (new_len + W - 1) / W;
        self.buf.resize(n, 0);
        self.len = new_len;
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

impl<const LARGE_LEN: usize, const SMALL_LEN: usize, const BIT_PATTERNS: usize>
    RankLookup<SMALL_LEN, BIT_PATTERNS>
    for RankIndex<LARGE_LEN, SMALL_LEN, BIT_PATTERNS>
{
    const WORD: [[u8; SMALL_LEN]; BIT_PATTERNS] =
        rank_lookup::<SMALL_LEN, BIT_PATTERNS>();
}

impl<const LARGE_LEN: usize, const SMALL_LEN: usize, const BIT_PATTERNS: usize>
    RankIndex<LARGE_LEN, SMALL_LEN, BIT_PATTERNS>
{
    fn new(a: &SimpleBitVec) -> Self {
        let mut small = vec![];
        let mut large = vec![];
        let mut small_acc = 0;
        let mut large_acc = 0;
        let per = LARGE_LEN / SMALL_LEN;
        for (c, i) in a
            .chunks::<true>(SMALL_LEN)
            .map(|ai| Self::WORD[ai as usize][SMALL_LEN - 1] as u16)
            .zip((0..per).cycle())
        {
            small.push(small_acc);
            small_acc = if i < per - 1 { small_acc + c } else { 0 };

            if i == 0 {
                large.push(large_acc);
            }
            large_acc += c as u32;
        }

        Self { large, small }
    }

    fn rank(&self, n: usize, b: &SimpleBitVec) -> usize {
        let large_acc = self.large[n / LARGE_LEN] as usize;
        let small_acc = self.small[n / SMALL_LEN] as usize;
        let il = n / SMALL_LEN * SMALL_LEN;
        let ir = b.len().min(il + SMALL_LEN);
        let w = b.get::<true>(il..ir);
        let small = Self::WORD[w as usize][n % SMALL_LEN] as usize;
        large_acc + small_acc + small
    }
}

impl<
    const SMALL_LEN: usize,
    const LARGE_SPARSE_LEN: usize,
    const LARGE_POPCNT: usize,
    const LARGE_NODE_LEN: usize,
    const LARGE_BRANCH: usize,
    const WORD_BIT_PATTERNS: usize,
    const TREE_BIT_PATTERNS: usize,
>
    SelectIndex<
        SMALL_LEN,
        LARGE_SPARSE_LEN,
        LARGE_POPCNT,
        LARGE_NODE_LEN,
        LARGE_BRANCH,
        WORD_BIT_PATTERNS,
        TREE_BIT_PATTERNS,
    >
{
    fn new<const X: bool>(b: &SimpleBitVec) -> Self {
        let n = b.len();

        let mut cur = vec![];
        let mut res = vec![];
        let mut start = 0;
        for i in 0..n {
            if b.get_single(i) == X {
                cur.push(i);
            }
            if cur.len() >= LARGE_POPCNT || i == n - 1 {
                let tmp = std::mem::take(&mut cur);
                res.push(SelectIndexInner::new::<X>(tmp, start..=i, b));
                start = i + 1
            }
        }
        Self { inner: res }
    }

    fn select<const X: bool>(&self, i: usize, b: &SimpleBitVec) -> usize {
        self.inner[i / LARGE_POPCNT].select::<X>(i % LARGE_POPCNT, b)
    }
}

impl<
    const SMALL_LEN: usize,
    const LARGE_SPARSE_LEN: usize,
    const LARGE_POPCNT: usize,
    const LARGE_NODE_LEN: usize,
    const LARGE_BRANCH: usize,
    const WORD_BIT_PATTERNS: usize,
    const TREE_BIT_PATTERNS: usize,
>
    SelectLookup<
        LARGE_NODE_LEN,
        LARGE_POPCNT,
        TREE_BIT_PATTERNS,
        SMALL_LEN,
        WORD_BIT_PATTERNS,
    >
    for SelectIndexInner<
        SMALL_LEN,
        LARGE_SPARSE_LEN,
        LARGE_POPCNT,
        LARGE_NODE_LEN,
        LARGE_BRANCH,
        WORD_BIT_PATTERNS,
        TREE_BIT_PATTERNS,
    >
{
    const TREE: [[(u8, u8); LARGE_POPCNT]; TREE_BIT_PATTERNS] =
        select_tree_lookup::<
            LARGE_NODE_LEN,
            LARGE_POPCNT,
            LARGE_BRANCH,
            TREE_BIT_PATTERNS,
        >();
    const WORD: [[u8; SMALL_LEN]; WORD_BIT_PATTERNS] =
        select_word_lookup::<SMALL_LEN, WORD_BIT_PATTERNS>();
}

impl<
    const SMALL_LEN: usize,
    const LARGE_SPARSE_LEN: usize,
    const LARGE_POPCNT: usize,
    const LARGE_NODE_LEN: usize,
    const LARGE_BRANCH: usize,
    const WORD_BIT_PATTERNS: usize,
    const TREE_BIT_PATTERNS: usize,
>
    SelectIndexInner<
        SMALL_LEN,
        LARGE_SPARSE_LEN,
        LARGE_POPCNT,
        LARGE_NODE_LEN,
        LARGE_BRANCH,
        WORD_BIT_PATTERNS,
        TREE_BIT_PATTERNS,
    >
{
    fn new<const X: bool>(
        a: Vec<usize>,
        range: RangeInclusive<usize>,
        b: &SimpleBitVec,
    ) -> Self {
        let start = *range.start();
        let end = range.end() + 1;
        if end - start >= LARGE_SPARSE_LEN {
            Self::Sparse(a)
        } else {
            Self::new_dense::<X>(b, start..end)
        }
    }

    fn new_dense<const X: bool>(
        b: &SimpleBitVec,
        Range { start, end }: Range<usize>,
    ) -> Self {
        let rl = &RankIndex::<0, SMALL_LEN, WORD_BIT_PATTERNS>::WORD;
        let len = end - start;

        let leaf = {
            let mut leaf = SimpleBitVec::new();
            for i in 0..(len + SMALL_LEN - 1) / SMALL_LEN {
                let il = start + i * SMALL_LEN;
                let ir = end.min(il + SMALL_LEN);
                let w = b.get::<X>(il..ir);
                leaf.push(rl[w as usize][SMALL_LEN - 1] as u64, LARGE_NODE_LEN);
            }
            leaf
        };

        let mut tree = vec![];
        let mut last = leaf;
        while last.len() > LARGE_NODE_LEN {
            let mut cur = SimpleBitVec::new();
            let tmp = last;
            {
                let mut it = tmp.chunks::<true>(LARGE_NODE_LEN);
                let mut trunc = None;
                while let Some(mut sum) = it.next() {
                    sum += (1..LARGE_BRANCH)
                        .filter_map(|_| it.next())
                        .sum::<u64>();
                    if sum & (!0 << LARGE_NODE_LEN) != 0 {
                        trunc = Some(sum);
                        sum &= !(!0 << LARGE_NODE_LEN);
                    }
                    cur.push(sum, LARGE_NODE_LEN);
                }
                if let Some(sum) = trunc {
                    if cur.len() > LARGE_NODE_LEN {
                        panic!("invalid popcount, {}", sum);
                    }
                }
            }
            tree.push(tmp);
            last = cur;
        }

        let mut level_len = LARGE_NODE_LEN * LARGE_BRANCH;
        let mut tree_len = level_len;
        let mut tree_flatten = SimpleBitVec::new();
        for level in tree.into_iter().rev() {
            tree_flatten.push_vec(level);
            tree_flatten.pad_zero(tree_len);
            level_len *= LARGE_BRANCH;
            tree_len += level_len;
        }

        Self::Dense(tree_flatten, start)
    }

    // [0, 1, 2]
    // [3, 4, 5], [6, 7, 8], [9, 10, 11]
    // [12, 13, 14], ...,
    fn select<const X: bool>(&self, i: usize, b: &SimpleBitVec) -> usize {
        match self {
            Self::Sparse(index) => index[i],
            Self::Dense(tree, start) => {
                let mut i = i;
                let mut nth_word = 0;
                let mut level_off = 0;
                let mut level_count = LARGE_BRANCH;
                while level_off * LARGE_NODE_LEN < tree.len() {
                    let il = (level_off + nth_word) * LARGE_NODE_LEN;
                    let ir = il + LARGE_NODE_LEN * LARGE_BRANCH;
                    let w = tree.get::<true>(il..ir);
                    let (branch, rank) = Self::TREE[w as usize][i];
                    nth_word = (nth_word + branch as usize) * LARGE_BRANCH;
                    level_off += level_count;
                    level_count *= LARGE_BRANCH;
                    i -= rank as usize;
                }

                nth_word /= LARGE_BRANCH;
                let il = start + nth_word * SMALL_LEN;
                let ir = b.len().min(il + SMALL_LEN);
                let w = b.get::<X>(il..ir);
                start
                    + nth_word * SMALL_LEN
                    + Self::WORD[w as usize][i] as usize
            }
        }
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

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_rank_lookup() {
        let table = rank_lookup::<3, 8>();

        assert_eq!(&table[0b000][..3], [0, 0, 0]);
        assert_eq!(&table[0b100][..3], [0, 0, 1]);
        assert_eq!(&table[0b010][..3], [0, 1, 1]);
        assert_eq!(&table[0b110][..3], [0, 1, 2]);
        assert_eq!(&table[0b001][..3], [1, 1, 1]);
        assert_eq!(&table[0b101][..3], [1, 1, 2]);
        assert_eq!(&table[0b011][..3], [1, 2, 2]);
        assert_eq!(&table[0b111][..3], [1, 2, 3]);
    }

    #[test]
    #[cfg(any())]
    fn test_select_tree_lookup() {
        let table = select_tree_lookup::<3, 9, 3, 512>();
        // [3, 4, 2]
        let tmp: [_; 9] = table[0b_010_100_011][..9].try_into().unwrap();

        assert_eq!(tmp.map(|x| x.0), [0, 0, 0, 1, 1, 1, 1, 2, 2]);
        assert_eq!(tmp.map(|x| x.1), [0, 0, 0, 3, 3, 3, 3, 7, 7]);
    }

    #[test]
    fn test_select_word_lookup() {
        let table = select_word_lookup::<3, 8>();

        assert_eq!(&table[0b001][..1], [0]);
        assert_eq!(&table[0b010][..1], [1]);
        assert_eq!(&table[0b011][..2], [0, 1]);
        assert_eq!(&table[0b100][..1], [2]);
        assert_eq!(&table[0b101][..2], [0, 2]);
        assert_eq!(&table[0b110][..2], [1, 2]);
        assert_eq!(&table[0b111][..3], [0, 1, 2]);
    }

    #[test]
    fn test_select_index() {
        let a = bitvec!(b"110 001 001 000 010 010");
        let b = SimpleBitVec::from(a.as_slice());
        let slt = SelectIndex::<3, 1000, 12, 4, 3, 8, 4096>::new::<true>(&b);

        // [6]
        // [4, 2]
        // [2, 1, 1, 0, 1, 1]

        let expected: Vec<_> = (0..a.len()).filter(|&i| a[i]).collect();
        for i in 0..expected.len() {
            assert_eq!(slt.select::<true>(i, &b), expected[i]);
        }
    }

    #[test]
    fn test_all_zero() {
        let n = 1000;
        let a = vec![false; n];
        let rs = Rs01Dict::new(&a);

        for i in 0..n {
            assert_eq!(rs.rank1(i), 0);
            assert_eq!(rs.rank0(i), i + 1);
            assert_eq!(rs.select0(i), i);
        }
    }
}
