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

// struct SelectIndex {
//     large: Vec<SelectIndexLarge>,
//     table: IntVec,
// }

// enum SelectIndexLarge {
//     Sparse(IntVec),
//     Dense(IntVec, usize),
// }

struct SelectIndex;

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

fn bitlen(n: usize) -> usize {
    // max {1, ceil(log2(|{0, 1, ..., n-1}|))}
    1.max((n + 1).next_power_of_two().trailing_zeros() as usize)
}
fn lg_half(n: usize) -> usize {
    // log(n)/2
    (1_usize..).find(|&i| 4_usize.saturating_pow(i as _) >= n).unwrap()
}
// fn lglg(n: usize) -> usize {
//     // log(log(n))
//     (1_usize..).find(|&i| (1_u128 << (1 << i)) >= n).unwrap()
// }
fn lglg2(n: usize) -> usize {
    // log(log(n))^2
    todo!()
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
        self.table.get::<true>(wi) as _
    }

    pub fn rank1(&self, i: usize, b: &IntVec) -> usize {
        let large_acc = self.large.get::<true>(i / self.large_len);
        let small_acc = self.small.get::<true>(i / self.small_len);
        let il = i / self.small_len * self.small_len;
        let ir = il + self.small_len;
        let w = b.bits_range::<true>(il..ir);
        let small = self.lookup(w, i % self.small_len);
        (large_acc + small_acc) as usize + small
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
        // ceil(lg(lg(len)))^2 だと大きすぎるので ceil(lg(lg(len))^2) とかにしたい
        let small_dense_max = lglg2(len);
        let large_dense_max = large_popcnt.pow(2); // log(n)^4
        let mut large_start = IntVec::new(bitlen(len) + 1);
        let mut large_sparse = IntVec::new(bitlen(len));
        let mut small_start = IntVec::new(bitlen(large_dense_max) + 1);
        let mut small_sparse = IntVec::new(bitlen(large_dense_max));

        let mut start = 0;
        let mut pos = vec![];
        for i in 0..len {
            if buf[i] == X {
                pos.push(i);
                if pos.len() == large_popcnt || i == len - 1 {
                    let end = i;
                    if end + 1 - start > large_dense_max {
                        large_start.push((large_sparse.len() << 1 | 0) as _);
                        for j in pos.drain(..) {
                            large_sparse.push(j as _);
                        }
                    } else {
                        large_start.push((small_start.len() << 1 | 1) as _);
                        // 必要な情報足りてる？ 簡潔になってる？
                        for ch in pos.chunks(small_popcnt) {
                            let start = ch[0];
                            let end = ch[ch.len() - 1];
                            if end + 1 - start > small_dense_max {
                                small_start
                                    .push((small_sparse.len() << 1 | 0) as _);
                                for j in ch {
                                    small_sparse.push((j - start) as _); // (?)
                                }
                            } else {
                                small_start
                                    .push(((start - pos[0]) << 1 | 1) as _);
                            }
                        }
                        pos.clear();
                    }

                    start = i + 1;
                }
            }
        }

        Self
    }
}

// impl SelectIndex {}

// impl SelectIndexLarge {}

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
        for i in 9999990..len {
            assert_eq!(dict.rank1(i), naive[i], "i: {}", i);
            assert_eq!(dict.rank0(i), i - naive[i], "i: {}", i);
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
}
