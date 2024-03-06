#![allow(unused)]

use std::ops::Range;

const W: usize = u64::BITS as usize;

struct IntVec {
    unit: usize,
    buf: Vec<u64>,
    len: usize,
}

pub struct Rs01DictRuntime {
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
    small_popcnt: usize,
    small_start: IntVec,
    small_indir: IntVec,
    small_sparse: IntVec,
    small_sparse_offset: IntVec,
    small_dense_max: usize,
    large_popcnt: usize,
    large_start: IntVec,
    large_indir: IntVec,
    large_sparse: IntVec,
    table: IntVec,
}

impl IntVec {
    pub fn new(unit: usize) -> Self { Self { unit, buf: vec![], len: 0 } }
    pub fn len(&self) -> usize { self.len }
    pub fn bitlen(&self) -> usize { self.len * self.unit }

    pub fn push(&mut self, w: u64) {
        let unit = self.unit;
        assert!(unit == W || w & (!0 << unit) == 0);

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

    pub fn get_usize(&self, i: usize) -> usize { self.get::<true>(i) as _ }

    pub fn get<const X: bool>(&self, i: usize) -> u64 {
        let start = i * self.unit;
        let end = start + self.unit;
        self.bits_range::<X>(start..end)
    }

    pub fn bits_range<const X: bool>(
        &self,
        Range { start, end }: Range<usize>,
    ) -> u64 {
        let end = end.min(self.bitlen()); // (!)
        let mask = if end - start == W { !0 } else { !(!0 << (end - start)) };
        let res = if start == end {
            0
        } else if start % W == 0 {
            self.buf[start / W]
        } else if end <= (start / W + 1) * W {
            self.buf[start / W] >> (start % W)
        } else {
            self.buf[start / W] >> (start % W)
                | self.buf[end / W] << (W - start % W)
        };
        (if X { res } else { !res }) & mask
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
fn lg_half(n: usize) -> usize {
    // log(n)/2
    (1_usize..).find(|&i| 4_usize.saturating_pow(i as _) >= n).unwrap()
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
    pub fn size_info(&self) {
        eprintln!("large: {} bits", self.large.bitlen());
        eprintln!("small: {} bits", self.small.bitlen());
        eprintln!("table: {} bits", self.table.bitlen());
    }
}

impl SelectIndex {
    pub fn new<const X: bool>(buf: &[bool]) -> Self {
        let len = buf.len();
        let small_popcnt = lg_half(len);
        let large_popcnt = (2 * small_popcnt).pow(2); // log(n)^2
        let small_dense_max =
            (((len as f64).log2().max(1.0).log2().max(1.0).powi(4) / 20.0)
                .ceil()) as usize;
        let large_dense_max = large_popcnt.pow(2); // log(n)^4
        let mut large_start = IntVec::new(bitlen(len));
        let mut large_indir = IntVec::new(bitlen(len) + 1);
        let mut large_sparse = IntVec::new(bitlen(len));
        let mut small_start = IntVec::new(bitlen(large_dense_max));
        let mut small_indir = IntVec::new(bitlen(large_dense_max) + 1);
        let mut small_sparse = IntVec::new(bitlen(large_dense_max));
        let mut small_sparse_offset = IntVec::new(bitlen(len));

        eprintln!("large_popcnt: {large_popcnt}");
        eprintln!("small_popcnt: {small_popcnt}");
        eprintln!("large_dense_max: {large_dense_max}");
        eprintln!("small_dense_max: {small_dense_max}");

        let mut start = 0;
        let mut pos = vec![];
        for i in 0..len {
            if buf[i] == X {
                pos.push(i);
            }
            if !(pos.len() == large_popcnt || i == len - 1) {
                continue;
            }

            let cur_large_start = start;
            let cur_large_end = i;
            large_start.push(cur_large_start as _);
            small_sparse_offset.push(small_sparse.len() as _);
            if cur_large_end + 1 - cur_large_start > large_dense_max {
                large_indir.push((large_sparse.len() << 1 | 0) as _);
                for p in pos.drain(..) {
                    large_sparse.push(p as _);
                }
            } else {
                large_indir.push((small_start.len() << 1 | 1) as _);
                let small_start_offset = small_start.len();
                let small_sparse_offset = small_sparse.len();
                let mut cur_small_start = cur_large_start;
                eprintln!("pos: {pos:?}");
                for j in (0..pos.len()).step_by(small_popcnt) {
                    let start = cur_small_start;
                    let end = if j + small_popcnt < pos.len() {
                        pos[j + small_popcnt] - 1
                    } else if i == len - 1 {
                        i
                    } else {
                        pos[pos.len() - 1]
                    };
                    small_start.push((start - cur_large_start) as _);
                    eprintln!("end: {end}, start: {start}");
                    if end + 1 - start > small_dense_max {
                        let tmp = (small_sparse.len() - small_sparse_offset)
                            / small_popcnt;
                        small_indir.push((tmp << 1 | 0) as _);
                        for &p in &pos[j..pos.len().min(j + small_popcnt)] {
                            let pos_offset = p - start;
                            small_sparse.push(pos_offset as _);
                        }
                    } else {
                        small_indir.push(0 << 1 | 1);
                    }
                    cur_small_start = end + 1;
                }

                pos.clear();
                start = i + 1;
            }
        }

        eprintln!("large_indir: {large_indir:?}");
        eprintln!("large_start: {large_start:?}");
        eprintln!("large_sparse: {large_sparse:?}");
        eprintln!("small_indir: {small_indir:?}");
        eprintln!("small_start: {small_start:?}");
        eprintln!("small_sparse: {small_sparse:?}");
        eprintln!("small_sparse_offset: {small_sparse_offset:?}");

        let table = Self::table(small_dense_max);
        Self {
            small_popcnt,
            small_start,
            small_indir,
            small_sparse,
            small_sparse_offset,
            small_dense_max,
            large_popcnt,
            large_start,
            large_indir,
            large_sparse,
            table,
        }
    }

    fn table(len: usize) -> IntVec {
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
        table
    }

    fn lookup(&self, w: u64, i: usize) -> usize {
        let wi = w as usize * self.small_dense_max + i;
        self.table.get_usize(wi)
    }

    pub fn select<const X: bool>(&self, i: usize, b: &IntVec) -> usize {
        let (il_div, il_mod) = (i / self.large_popcnt, i % self.large_popcnt);
        let large = self.large_indir.get_usize(il_div);
        let (large_i, large_ty) = (large >> 1, large & 1);
        if large_ty == 0 {
            self.large_sparse.get_usize(large_i + il_mod)
        } else {
            let large_start = self.large_start.get_usize(il_div);
            let per = self.large_popcnt / self.small_popcnt;
            let is_div = i / self.small_popcnt % per;
            let is_mod = i % self.small_popcnt;

            eprintln!(
                "small_indir[(large_i = {large_i}) + (is_div = {is_div})]"
            );
            let small = self.small_indir.get_usize(large_i + is_div);
            let (small_i, small_ty) = (small >> 1, small & 1);
            eprintln!("small_i: {small_i}");
            let small_start = self.small_start.get_usize(large_i + is_div);
            if small_ty == 0 {
                eprintln!("small_i: {small_i}, is_mod: {is_mod}");
                eprintln!(
                    "small_sparse[(small_i = {small_i}) + (is_mod = {is_mod})]"
                );
                let offset = self.small_sparse_offset.get_usize(il_div);
                let small_sparse = self
                    .small_sparse
                    .get_usize(offset + small_i * self.small_popcnt + is_mod);
                eprintln!(
                    "large_start: {large_start}, small_start: {small_start}, small_sparse: {small_sparse}"
                );
                large_start + small_start + small_sparse
            } else {
                let offset = large_start + small_start;
                let w =
                    b.bits_range::<X>(offset..offset + self.small_dense_max);
                eprintln!("offset: {offset}, w: {w:064b}, i: {is_mod}");
                eprintln!("-> {}", offset + self.lookup(w, is_mod));
                offset + self.lookup(w, is_mod)
            }
        }
    }
}

impl Rs01DictRuntime {
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
        self.rank_index.size_info();
        //
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
        let dict = Rs01DictRuntime::new(&a);
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
        let mut rng = rng();
        let dist = Bernoulli::new(p).unwrap();
        let a: Vec<_> = (0..len).map(|_| dist.sample(&mut rng)).collect();
        let naive: (Vec<_>, _) = (0..len).partition(|&i| !a[i]);
        let dict = Rs01DictRuntime::new(&a);
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
