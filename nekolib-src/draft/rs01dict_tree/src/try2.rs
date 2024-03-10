use std::ops::Range;

const W: usize = u64::BITS as usize;

struct IntVec {
    unit: usize,
    buf: Vec<u64>,
    len: usize,
}

pub struct Rs01DictTree {
    buf: IntVec,
    rank_index: RankIndex,
    select_index: (SelectIndex, SelectIndex),
}

struct RankIndex {
    large: IntVec,
    small: IntVec,
    table: IntVec,
    large_len: usize,
    small_len: usize,
}

struct SelectIndex {
    indir: IntVec,
    sparse: IntVec,
    dense: IntVec,
    table_tree: IntVec,
    table_word: IntVec,
    large_popcnt: usize,
    branch: usize,
    small_len: usize,
}

impl IntVec {
    pub fn new(unit: usize) -> Self { Self { unit, buf: vec![], len: 0 } }
    pub fn len(&self) -> usize { self.len }
    pub fn bitlen(&self) -> usize { self.len * self.unit }

    pub fn push(&mut self, w: u64) {
        let unit = self.unit;
        debug_assert!(unit == W || w & (!0 << unit) == 0);

        let bitlen = self.bitlen();
        if unit == 0 {
            // nothing to do
        } else if bitlen % W == 0 {
            self.buf.push(w);
        } else {
            self.buf[bitlen / W] |= w << (bitlen % W);
            if bitlen % W + unit > W {
                self.buf.push(w >> (W - bitlen % W));
            }
        }
        self.len += 1;
    }

    #[inline(always)]
    pub fn get_usize(&self, i: usize) -> usize { self.get::<true>(i) as _ }

    #[inline(always)]
    pub fn get<const X: bool>(&self, i: usize) -> u64 {
        let start = i * self.unit;
        let end = start + self.unit;
        self.bits_range::<X>(start..end)
    }

    #[inline(always)]
    pub fn bits_range<const X: bool>(
        &self,
        Range { start, end }: Range<usize>,
    ) -> u64 {
        let end = end.min(self.bitlen()); // (!)
        // let mask = if end - start == W { !0 } else { !(!0 << (end - start)) };
        let mask = !(!0 << (end - start));
        // let res = if start % W == 0 {
        //     self.buf[start / W]
        //     // unsafe { *self.buf.get_unchecked(start / W) }
        // } else if end <= (start / W + 1) * W {
        //     self.buf[start / W] >> (start % W)
        //     // (unsafe { *self.buf.get_unchecked(start / W) }) >> (start % W)
        // } else {
        //     self.buf[start / W] >> (start % W)
        //         | self.buf[end / W] << (W - start % W)
        //     // (unsafe { *self.buf.get_unchecked(start / W) }) >> (start % W)
        //     //     | (unsafe { *self.buf.get_unchecked(end / W) })
        //     //         << (W - start % W)
        // };

        let mut res = self.buf[start / W] >> (start % W);
        if end > (start / W + 1) * W {
            res |= self.buf[end / W] << (W - start % W);
        }

        // let mask = !(!0_u128 << (end - start));
        // let mut res = self.buf[start / W] as u128;
        // res |= (*self.buf.get(start / W + 1).unwrap_or(&0) as u128) << W;
        // res >>= start % W;

        ((if X { res } else { !res }) & mask) as _
    }
}

impl std::fmt::Debug for IntVec {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_list()
            .entries((0..self.len).map(|i| self.get::<true>(i)))
            .finish()
    }
}

fn bitlen(n: usize) -> usize {
    // max {1, ceil(log2(|{0, 1, ..., n-1}|))}
    1.max((n + 1).next_power_of_two().trailing_zeros() as usize)
}

impl RankIndex {
    pub fn new(buf: &[bool]) -> Self {
        let len = buf.len();
        let small_len = (1_usize..)
            .find(|&i| 4_usize.saturating_pow(i as _) >= len)
            .unwrap(); // log(n)/2
        let large_len = (2 * small_len).pow(2); // log(n)^2

        let small_bitlen = bitlen(len.min(large_len));
        let large_bitlen = bitlen(len);

        let mut small = IntVec::new(small_bitlen);
        let mut large = IntVec::new(large_bitlen);
        let mut small_acc = 0;
        let mut large_acc = 0;
        let per = large_len / small_len;
        for (c, i) in buf
            .chunks(small_len)
            .map(|ch| ch.iter().filter(|&&x| x).count() as u64)
            .zip((0..per).cycle())
        {
            small.push(small_acc);
            small_acc = if i < per - 1 { small_acc + c } else { 0 };

            if i == 0 {
                large.push(large_acc);
            }
            large_acc += c as u64;
        }

        let table = Self::table(small_len);
        Self { large, small, table, large_len, small_len }
    }

    fn table(len: usize) -> IntVec {
        let unit = bitlen(len);
        let mut table = IntVec::new(unit);
        for i in 0..1 << len {
            let mut cur = 0;
            for j in 0..len {
                table.push(cur);
                if i >> j & 1 != 0 {
                    cur += 1;
                }
            }
        }
        table
    }

    #[inline(always)]
    fn lookup(&self, w: u64, i: usize) -> usize {
        let wi = w as usize * self.small_len + i;
        self.table.get_usize(wi)
    }

    pub fn rank1(&self, i: usize, b: &IntVec) -> usize {
        let large_acc = self.large.get_usize(i / self.large_len);
        let small_acc = self.small.get_usize(i / self.small_len);
        let il = i / self.small_len * self.small_len;
        let ir = il + self.small_len;
        let w = b.bits_range::<true>(il..ir);
        let small = self.lookup(w, i % self.small_len);
        large_acc + small_acc + small
    }
    pub fn rank0(&self, i: usize, b: &IntVec) -> usize { i - self.rank1(i, b) }
    pub fn rank<const X: bool>(&self, i: usize, b: &IntVec) -> usize {
        if X { self.rank1(i, b) } else { self.rank0(i, b) }
    }

    #[cfg(test)]
    pub fn size_info(&self) -> (usize, usize) {
        // eprintln!("large: {} bits", self.large.bitlen());
        // eprintln!("small: {} bits", self.small.bitlen());
        // eprintln!("table: {} bits", self.table.bitlen());

        let rt = self.large.bitlen() + self.small.bitlen();
        (rt, rt + self.table.bitlen())
    }
}

impl SelectIndex {
    pub fn new<const X: bool>(buf: &[bool]) -> Self {
        let len = buf.len();
        let len_lg = (len as f64).log2().max(1.0);

        // eprintln!("len_lg: {len_lg}");

        let dense_max = (len_lg.powi(4) / 128.0).ceil() as usize;
        let large_popcnt = (len_lg.powi(2) / 16.0).ceil() as usize;
        let small_len = (len_lg / 2.0).ceil().max(2.0) as usize;
        let branch = len_lg.cbrt().ceil() as usize;

        // eprintln!("large_popcnt: {large_popcnt}");

        let mut indir = IntVec::new(bitlen(len) + 2);
        let mut sparse = IntVec::new(bitlen(len));
        let mut dense = IntVec::new(bitlen(large_popcnt));

        let mut start = 0;
        let mut pos = vec![];
        for i in 0..len {
            if buf[i] == X {
                pos.push(i);
            }
            if !(pos.len() == large_popcnt || i == len - 1) {
                continue;
            }

            let end = i;
            if end + 1 - start > dense_max {
                indir.push((sparse.len() << 1 | 0) as _);
                indir.push(0);
                indir.push(0);
                for &p in &pos {
                    sparse.push(p as _);
                }
            } else {
                indir.push((dense.len() << 1 | 1) as _);
                let ceil_len = (1..)
                    .map(|i| branch.pow(i) * small_len)
                    .find(|&b| b >= end + 1 - start)
                    .unwrap();
                let mut cur = dense.len();
                for i in (start..start + ceil_len).step_by(small_len).rev() {
                    let il = i.min(end + 1);
                    let ir = (il + small_len).min(end + 1);
                    let w = (il..ir).filter(|&i| buf[i] == X).count();
                    dense.push(w as _);
                }
                while cur + branch < dense.len() {
                    let mut sum = 0;
                    for _ in 0..branch {
                        sum += dense.get::<true>(cur);
                        cur += 1;
                    }
                    dense.push(sum);
                }
                indir.push(dense.len() as _);
                indir.push(start as _);
            }

            pos.clear();
            start = i + 1;
        }

        let table_tree = Self::table_tree(large_popcnt, branch);
        let table_word = Self::table_word(small_len);

        Self {
            indir,
            sparse,
            dense,
            table_tree,
            table_word,
            large_popcnt,
            branch,
            small_len,
        }
    }

    #[inline(always)]
    fn lookup_tree(&self, w: u64, i: usize) -> (usize, usize) {
        let bitlen_branch = bitlen(self.branch);
        let wi = w as usize * self.large_popcnt + i;
        // let res = self.table_tree[wi] as usize;
        let res = self.table_tree.get_usize(wi);
        (res >> bitlen_branch, res & !(!0 << bitlen_branch))
    }

    #[inline(always)]
    fn lookup_word(&self, w: u64, i: usize) -> usize {
        let wi = w as usize * self.small_len + i;
        self.table_word.get_usize(wi)
    }

    fn table_tree(popcnt: usize, branch: usize) -> IntVec {
        let len = bitlen(popcnt);
        let unit = len + bitlen(branch);
        // let bits = (unit * len * branch) << (len * branch);
        // let words = (bits + 63) / 64;

        // [4, 3, 2] in a natrual order.
        // [2, 3, 4] in a reversed order.
        // 100_011_010 in a word.
        // 0..4 -> (0, 0)
        // 4..7 -> (4, 1)
        // 7..9 -> (7, 2)

        let enc = |i, j| i << bitlen(branch) | j;
        // let mut table = vec![];
        let mut table = IntVec::new(unit);
        for i in 0..1 << (len * branch) {
            let mut count = 0;
            for b in 0..branch {
                let sh = (branch - 1 - b) * len;
                let c = i >> sh & !(!0 << len);
                if count + c > popcnt {
                    break;
                }
                for _ in 0..c {
                    table.push(enc(count, b) as _);
                }
                count += c;
            }
            for _ in count..popcnt {
                table.push(0);
            }
        }
        table
    }

    fn table_word(len: usize) -> IntVec {
        let unit = bitlen(len);
        let mut table = IntVec::new(unit);
        for i in 0..1 << len {
            let mut cur = 0;
            for j in 0..len {
                if i >> j & 1 != 0 {
                    table.push(j as _);
                    cur += 1;
                }
            }
            for _ in cur..len {
                table.push(0);
            }
        }
        // eprintln!(
        //     "table size: {} bits, {} words",
        //     table.bitlen(),
        //     table.bitlen() / 64
        // );
        table
    }

    pub fn select<const X: bool>(&self, i: usize, b: &IntVec) -> usize {
        let (il_div, il_mod) = (i / self.large_popcnt, i % self.large_popcnt);
        let large = self.indir.get_usize(3 * il_div);
        let (large_i, large_ty) = (large >> 1, large & 1);
        if large_ty == 0 {
            self.sparse.get_usize(large_i + il_mod)
        } else {
            let start = large_i;
            let end = self.indir.get_usize(3 * il_div + 1);
            let b_start = self.indir.get_usize(3 * il_div + 2);
            let unit = bitlen(self.large_popcnt);
            let branch = self.branch;
            let mut cur = 0;
            let mut i = il_mod;
            let mut b_i = 0;
            loop {
                let il = (end - (cur + branch)) * unit;
                let ir = (end - cur) * unit;
                let w = self.dense.bits_range::<true>(il..ir);
                let (acc, br) = self.lookup_tree(w, i);
                let tmp = (cur + br + 1) * branch;
                if end - start <= tmp {
                    let il = b_start + (b_i * branch + br) * self.small_len;
                    let ir = il + self.small_len;
                    let w = b.bits_range::<X>(il..ir);
                    break il + self.lookup_word(w, i - acc);
                }
                b_i = b_i * branch + br;
                cur = tmp;
                i -= acc;
            }
        }
    }

    #[cfg(test)]
    pub fn size_info(&self) -> (usize, usize) {
        // eprintln!("indir:  {} bits", self.indir.bitlen());
        // eprintln!("sparse: {} bits", self.sparse.bitlen());
        // eprintln!("dense:  {} bits", self.dense.bitlen());

        let rt =
            self.indir.bitlen() + self.sparse.bitlen() + self.dense.bitlen();

        (rt, rt + self.table_tree.bitlen() + self.table_word.bitlen())
    }
}

impl Rs01DictTree {
    pub fn new(a: &[bool]) -> Self {
        let rank_index = RankIndex::new(&a);
        let mut buf = IntVec::new(1);
        for &x in a {
            buf.push(x as _);
        }
        let select_index =
            (SelectIndex::new::<false>(&a), SelectIndex::new::<true>(&a));

        // select.0 と select.1 で同じ lookup table を作るの無駄だから、
        // rank も含めてそれらは親のクラスで持つ設計でもいいかも？
        // buf と一緒に table も渡す感じで
        Self { buf, rank_index, select_index }
    }

    pub fn rank<const X: bool>(&self, i: usize) -> usize {
        self.rank_index.rank::<X>(i, &self.buf)
    }
    pub fn rank0(&self, i: usize) -> usize { self.rank::<false>(i) }
    pub fn rank1(&self, i: usize) -> usize { self.rank::<true>(i) }

    pub fn select<const X: bool>(&self, i: usize) -> usize {
        if X {
            self.select_index.1.select::<X>(i, &self.buf)
        } else {
            self.select_index.0.select::<X>(i, &self.buf)
        }
    }
    pub fn select0(&self, i: usize) -> usize { self.select::<false>(i) }
    pub fn select1(&self, i: usize) -> usize { self.select::<true>(i) }

    #[cfg(test)]
    pub fn size_info(&self) {
        let len = self.buf.bitlen();
        let naive = 3 * len * bitlen(len);
        eprintln!("* naive: {naive:>10} bits, {:>10} words", naive / 64);

        let (r, r_table) = self.rank_index.size_info();
        let (s0, s0_table) = self.select_index.0.size_info();
        let (s1, s1_table) = self.select_index.1.size_info();
        let sum = r + s0 + s1;
        let sum_table = r_table + s0_table + s1_table;

        let ratio = sum as f64 / naive as f64;
        eprintln!(
            "- table: {sum:>10} bits, {:>10} words (x{ratio:.03})",
            sum / 64
        );
        let ratio = sum_table as f64 / naive as f64;
        eprintln!(
            "+ table: {sum_table:>10} bits, {:>10} words (x{ratio:.03})",
            sum_table / 64
        );
    }
}

#[cfg(test)]
mod tests {
    use rand::{
        distributions::{Bernoulli, Distribution},
        Rng, SeedableRng,
    };
    use rand_chacha::ChaCha20Rng;

    use crate::*;

    fn rng() -> ChaCha20Rng {
        ChaCha20Rng::from_seed([
            0x55, 0xEF, 0xE0, 0x3C, 0x71, 0xDA, 0xFC, 0xAB, 0x5C, 0x1A, 0x9F,
            0xEB, 0xA4, 0x9E, 0x61, 0xE6, 0x1E, 0x7E, 0x29, 0x77, 0x38, 0x9A,
            0xF5, 0x67, 0xF5, 0xDD, 0x07, 0x06, 0xAE, 0xE4, 0x5A, 0xDC,
        ])
    }

    fn test_rank_internal(len: usize, p: f64) {
        let mut rng = rng();
        let dist = Bernoulli::new(p).unwrap();
        let a: Vec<_> = (0..len).map(|_| dist.sample(&mut rng)).collect();
        let naive: Vec<_> = a
            .iter()
            .map(|&x| x as usize)
            .scan(0, |acc, x| Some(std::mem::replace(acc, *acc + x)))
            .collect();
        let dict = Rs01DictTree::new(&a);
        for i in 0..len {
            assert_eq!(dict.rank1(i), naive[i], "i: {}", i);
            assert_eq!(dict.rank0(i), i - naive[i], "i: {}", i);
        }
        if p == 1.0 {
            eprintln!("---");
            eprintln!("a.len(): {}", a.len());
            dict.size_info();
        }
    }

    fn test_select_internal(len: usize, p: f64) {
        eprintln!("{:?}", (len, p));
        let mut rng = rng();
        let dist = Bernoulli::new(p).unwrap();
        let a: Vec<_> = (0..len).map(|_| dist.sample(&mut rng)).collect();
        let naive: (Vec<_>, _) = (0..len).partition(|&i| !a[i]);
        let dict = Rs01DictTree::new(&a);

        for i in 0..naive.0.len() {
            assert_eq!(dict.select0(i), naive.0[i], "i: {}", i);
        }
        for i in 0..naive.1.len() {
            assert_eq!(dict.select1(i), naive.1[i], "i: {}", i);
        }
        if p == 1.0 {
            eprintln!("---");
            eprintln!("a.len(): {}", a.len());
            dict.size_info();
        }
    }

    #[test]
    fn test_rank() {
        for len in Some(0).into_iter().chain((0..=7).map(|e| 10_usize.pow(e))) {
            for &p in &[1.0, 0.999, 0.9, 0.5, 0.1, 1.0e-3, 0.0] {
                test_rank_internal(len, p);
            }
        }
    }

    #[test]
    fn test_select() {
        for len in Some(0).into_iter().chain((0..=7).map(|e| 10_usize.pow(e))) {
            for &p in &[1.0, 0.999, 0.9, 0.5, 0.1, 1.0e-3, 0.0] {
                test_select_internal(len, p);
            }
        }
    }

    #[test]
    fn sanity_check() { test_select_internal(100, 0.2); }
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
fn simple() {
    // let a = bitvec!(b"110 100 000 101 001 010 111 110 010");
    // let _ = SelectIndex::new::<true>(&a);

    // let a = bitvec!(b"1101 1000 0001 1010 0010 0101 1110 1101 0100");
    // let _ = SelectIndex::new::<true>(&a);

    // let a = bitvec!(b"1101 1000 0001 1010 0010 0101 11");
    // let _ = SelectIndex::new::<true>(&a);

    //                 (13)
    //       3           4           6
    //   2   1   0   2   1   1   3   2   1  <- sum of length log(log(n)^2)
    // 110 100 000 101 001 010 111 110 010  <- block of length log(n)/2
    //
    // 3 4 6; 2 1 0; 2 1 1; 3 2 1
    //
    // 1 2 3; 1 1 2; 0 1 2; _ _ _; _
    //
    // 0..26
    // [1, 2, 3, 1, 1, 2, 0, 1, 2, 6, 4, 3]
    // ok

    //         5              5              7
    //    3    1    1    2    1    2    3    3    1
    // 1101 1000 0001 1010 0010 0101 1110 1101 0100
    //
    // [1, 3, 3, 2, 1, 2, 1, 1, 3, 7, 5, 5]
    // ok

    //         5              5              2
    //    3    1    1    2    1    2    2    0    0
    // 1101 1000 0001 1010 0010 0101 11__ ____ ____
    //
    // [0, 0, 2, 2, 1, 2, 1, 1, 3, 2, 5, 5]
    // ok

    for i in 0..=7 {
        let a = vec![false; 10_usize.pow(i)];
        let dict = Rs01DictTree::new(&a);
        dict.size_info();
    }
}
