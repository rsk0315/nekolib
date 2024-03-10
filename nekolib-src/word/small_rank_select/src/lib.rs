#[inline(always)]
fn splat(w: u8) -> u64 {
    // abcdefgh -> abcdefgh abcdefgh abcdefgh abcdefgh abcdefgh abcdefgh abcdefgh abcdefgh
    0x0101010101010101 * w as u64
}

#[inline(always)]
fn onehot(w: u8) -> u64 {
    // abcdefgh -> a0000000 0b000000 00c00000 000d0000 0000e000 00000f00 000000g0 0000000h
    splat(w) & 0x8040201008040201
}

#[inline(always)]
fn nonzero(w: u64) -> u64 {
    // 00000000 -> 00000000, _ -> 00000001 for each block
    const HI: u64 = 0x8080808080808080;
    // let hi = w & HI;
    // let lo = !(HI - (w & !HI)) & HI;
    ((w | !(HI - (w & !HI))) & HI) >> 7
}

#[inline(always)]
fn accumulate(w: u64) -> u64 {
    // [h, g, f, e, d, c, b, a] -> [h, h + g, h + g + f, ..., h + g + ... + a]
    // a, b, ..., h \in {00000000, 00000001}
    w.wrapping_mul(0x0101010101010101)
}

#[inline(always)]
pub fn rank(w: u8, i: usize) -> usize {
    ((accumulate(nonzero(onehot(w))) << 8) >> (8 * i) & 7) as _
}

#[inline(always)]
pub fn select(w: u8, i: usize) -> usize {
    const HI: u64 = 0x8080808080808080;

    let lhs = splat(i as _) | HI;
    let rhs = accumulate(nonzero(onehot(w)));
    (accumulate(((lhs - rhs) & HI) >> 7) >> 56) as _
}

#[cfg(test)]
mod tests {
    use crate::*;

    const fn rank_table<const LEN: usize, const PAT: usize>() -> [[u8; LEN]; PAT]
    {
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

    const fn select_table<const LEN: usize, const PAT: usize>()
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

    const RANK_TABLE: [[u8; 8]; 256] = rank_table::<8, 256>();
    const SELECT_TABLE: [[u8; 8]; 256] = select_table::<8, 256>();

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
