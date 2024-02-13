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

const LG_N: usize = 8;

const LARGE: usize = LG_N * LG_N;
const SMALL: usize = LG_N / 2;
const POW2_SMALL: usize = 1 << SMALL;
const RANK_LOOKUP: [[u16; SMALL]; POW2_SMALL] =
    rank_lookup::<SMALL, POW2_SMALL>();

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

const fn select_lookup<
    const BIT_PATTERNS: usize, // 2^(branch * large)
    const BRANCH: usize,       // sqrt(log(n))
    const MAX_ONES: usize,     // log(n)^2
    const LG2_MAX_ONES: usize, // O(log(log(n)))
>() -> [[u16; MAX_ONES]; BIT_PATTERNS] {
    let mut table = [[0; MAX_ONES]; BIT_PATTERNS];
    let mut i = 0;
    while i < BIT_PATTERNS {
        let mut j = 0;
        let mut index = 0;
        while j < BRANCH {
            // [0011, 0100, 0010] (0b_0010_0100_0011)
            // [0, 0, 0, 1, 1, 1, 1, 2, 2, 3, ...]
            let count = i >> (j * LG2_MAX_ONES) & !(!0 << LG2_MAX_ONES);
            let mut k = 0;
            while k < count && index < MAX_ONES {
                table[i][index] = j as u16;
                index += 1;
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }
    table
}

struct RankIndex<const LARGE: usize, const SMALL: usize> {
    buf: Vec<u64>,

    /// $`\log(n)^2`$-bit blocks.
    large: Vec<u16>,

    /// $`O(\tfrac12\log(n))`$-bit blocks.
    small: Vec<u16>,
}

impl<const LARGE: usize, const SMALL: usize> RankIndex<LARGE, SMALL> {
    // `a` should be the value returned by `compress_vec_bool::<SMALL>(_)`
    fn new(a: Vec<u64>) -> Self {
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

    fn rank1(&self, n: usize) -> usize {
        let large = self.large[n / LARGE];
        let small = self.small[n / SMALL];
        let rem = RANK_LOOKUP[self.buf[n / SMALL] as usize][n % SMALL];
        large as usize + small as usize + rem as usize
    }
}

enum SelectIndexInner<const LARGE: usize, const SMALL: usize> {
    /// at least $`\log(n)^4`$-bit blocks.
    Sparse(Vec<usize>),

    /// less than $`\log(n)^4`$-bit blocks.
    Dense(Vec<u64>),
}

struct SelectIndex<const LARGE: usize, const SMALL: usize> {
    ds: Vec<SelectIndexInner<LARGE, SMALL>>,
}

impl<const LARGE: usize, const SMALL: usize> SelectIndexInner<LARGE, SMALL> {
    fn new(a: Vec<usize>, range: RangeInclusive<usize>) -> Self {
        let start = *range.start();
        let end = *range.end() + 1;
        if end - start + 1 >= LARGE * LARGE {
            Self::Sparse(a)
        } else {
            let len = end - start;
            let mut tmp = vec![0_u64; (len + W - 1) / W];
            for &ai in &a {
                tmp[ai / W] |= 1 << (ai % W);
            }
            Self::Dense(tmp)
        }
    }
}

impl<const LARGE: usize, const SMALL: usize> SelectIndex<LARGE, SMALL> {
    fn new<const X: bool>(a: &[bool]) -> Self {
        let n = a.len();
        let mut cur = vec![];
        let mut res = vec![];
        let mut start = 0;
        for i in 0..n {
            if a[i] == X {
                cur.push(i);
            }
            if cur.len() == LARGE || i == n - 1 {
                let tmp = std::mem::take(&mut cur);
                res.push(SelectIndexInner::new(tmp, start..=i));
                start = i + 1;
            }
        }
        Self { ds: res }
    }
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
    let table = select_lookup::<4096, 3, 16, 4>();
    assert_eq!(&table[0b_0010_0100_0011][0..9], [0, 0, 0, 1, 1, 1, 1, 2, 2]);
}

#[test]
fn sanity_check() {
    let a = bitvec!(b"000 010 110 000; 111 001 000 011; 000 000 010 010");
    let b = compress_vec_bool::<3>(&a);
    let rp = RankIndex::<12, 3>::new(b.clone());
    for i in 0..a.len() {
        eprintln!("{i} -> {}", rp.rank1(i));
    }
}
