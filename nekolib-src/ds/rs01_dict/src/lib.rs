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

const LG_N: usize = 8;

const LARGE: usize = LG_N * LG_N;
const SMALL: usize = LG_N / 2;
const POW2_SMALL: usize = 1 << SMALL;
const RANK_LOOKUP: [[u16; SMALL]; POW2_SMALL] =
    rank_lookup::<SMALL, POW2_SMALL>();

const fn rank_lookup<const SMALL: usize, const POW2_SMALL: usize>()
-> [[u16; SMALL]; POW2_SMALL] {
    let mut table = [[0; SMALL]; POW2_SMALL];
    let mut i = 0;
    while i < (1 << SMALL) {
        table[i][0] = (i & 1) as u16;
        let mut j = 1;
        while j < SMALL {
            table[i][j] = table[i][j - 1] + (i >> j & 1) as u16;
            j += 1;
        }
        i += 1;
    }
    table
}

struct RankPreprocess<const LARGE: usize, const SMALL: usize> {
    buf: Vec<u64>,

    /// $`\log(n)^2`$-bit blocks.
    large: Vec<u16>,

    /// $`O(\tfrac12\log(n))`$-bit blocks.
    small: Vec<u16>,
}

impl<const LARGE: usize, const SMALL: usize> RankPreprocess<LARGE, SMALL> {
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

enum SelectPreprocessDs<const LARGE: usize, const SMALL: usize> {
    /// at least $`\log(n)^4`$-bit blocks.
    Sparse(Vec<usize>),

    /// less than $`\log(n)^4`$-bit blocks.
    Dense,
}

struct SelectPreprocess<const LARGE: usize, const SMALL: usize> {
    ds: Vec<SelectPreprocessDs<LARGE, SMALL>>,
}

impl<const LARGE: usize, const SMALL: usize> SelectPreprocessDs<LARGE, SMALL> {
    fn new(a: Vec<usize>, range: RangeInclusive<usize>) -> Self {
        let start = *range.start();
        let end = *range.end();
        if end - start + 1 >= LARGE * LARGE {
            // sparse
            Self::Sparse(a)
        } else {
            // dense
            todo!()
        }
    }
}

impl<const LARGE: usize, const SMALL: usize> SelectPreprocess<LARGE, SMALL> {
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
                res.push(SelectPreprocessDs::new(tmp, start..=i));
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
fn sanity_check() {
    let a = bitvec!(b"000 010 110 000; 111 001 000 011; 000 000 010 010");
    let b = compress_vec_bool::<3>(&a);
    let rp = RankPreprocess::<12, 3>::new(b.clone());
    for i in 0..a.len() {
        eprintln!("{i} -> {}", rp.rank1(i));
    }
}
