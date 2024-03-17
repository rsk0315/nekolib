use std::ops::Range;

const W: usize = u64::BITS as usize;
const BLOCK_LEN: usize = 8;
const BLOCK: u64 = !(!0 << BLOCK_LEN);
const HI_POS: usize = BLOCK_LEN - 1;
const M_LO: u64 = 0x0101010101010101;
const M_IOTA: u64 = 0x8040201008040201;
const M_HI: u64 = M_LO << HI_POS;

#[inline(always)]
fn splat(w: u8) -> u64 { M_LO * w as u64 }
#[inline(always)]
fn nonzero(w: u64) -> u64 { ((w | (M_HI - M_LO + w)) & M_HI) >> HI_POS }
#[inline(always)]
fn expand(w: u8) -> u64 { nonzero(splat(w) & M_IOTA) }
#[inline(always)]
fn accumulate(w: u64) -> u64 { w.wrapping_mul(M_LO) }
#[inline(always)]
fn get(w: u64, i: usize) -> usize { (w >> (BLOCK_LEN * i) & BLOCK) as _ }
#[inline(always)]
fn gt_eq(wl: u64, wr: u64) -> u64 { (((wl | M_HI) - wr) & M_HI) >> HI_POS }
#[inline(always)]
fn shift(w: u64) -> u64 { w << BLOCK_LEN }
#[inline(always)]
fn popcnt(w: u64) -> usize { (accumulate(w) >> (W - BLOCK_LEN)) as _ }

#[inline(always)]
pub fn rank(w: u8, i: usize) -> usize { get(shift(accumulate(expand(w))), i) }
#[inline(always)]
pub fn select(w: u8, i: usize) -> usize {
    popcnt(gt_eq(splat(i as _), accumulate(expand(w))))
}

pub const fn const_rank_table<const LEN: usize, const PAT: usize>()
-> [[u8; LEN]; PAT] {
    let mut res = [[0; LEN]; PAT];
    let mut i = 0;
    while i < PAT {
        let mut cur = 0;
        let mut j = 0;
        while j < LEN {
            res[i][j] = cur;
            if i >> j & 1 != 0 {
                cur += 1;
            }
            j += 1;
        }
        i += 1;
    }
    res
}

pub const fn const_select_table<const LEN: usize, const PAT: usize>()
-> [[u8; LEN]; PAT] {
    let mut res = [[0; LEN]; PAT];
    let mut i = 0;
    while i < PAT {
        let mut cur = 0;
        let mut j = 0;
        while j < LEN {
            if i >> j & 1 != 0 {
                res[i][cur] = j as _;
                cur += 1;
            }
            j += 1;
        }
        i += 1;
    }
    res
}

#[cfg(test)]
mod tests {
    use crate::*;

    const RANK_TABLE: [[u8; 8]; 256] = const_rank_table::<8, 256>();
    const SELECT_TABLE: [[u8; 8]; 256] = const_select_table::<8, 256>();

    #[test]
    fn test_rank() {
        for w in 0_u8..=!0 {
            for i in 0..8 {
                assert_eq!(rank(w, i), RANK_TABLE[w as usize][i] as usize);
            }
        }
    }

    #[test]
    fn test_select() {
        for w in 0_u8..=!0 {
            for i in 0..w.count_ones() as _ {
                assert_eq!(select(w, i), SELECT_TABLE[w as usize][i] as usize);
            }
        }
    }
}

pub struct IntVec {
    unit: usize,
    buf: Vec<u64>,
    len: usize,
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
        let mask = !(!0 << (end - start));

        let mut res = self.buf[start / W] >> (start % W);
        if end > (start / W + 1) * W {
            res |= self.buf[end / W] << (W - start % W);
        }

        ((if X { res } else { !res }) & mask) as _
    }
}

pub struct RankTable(IntVec);
pub struct SelectTable(IntVec);

impl RankTable {
    pub fn new() -> Self {
        let len = 8;
        let unit = 3;
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
        Self(table)
    }
    pub fn rank(&self, w: u64, i: usize) -> usize {
        let wi = w as usize * 8 + i;
        self.0.get_usize(wi)
    }
}

impl SelectTable {
    pub fn new() -> Self {
        let len = 8;
        let unit = 3;
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
        Self(table)
    }

    pub fn select(&self, w: u64, i: usize) -> usize {
        let wi = w as usize * 8 + i;
        self.0.get_usize(wi)
    }
}
