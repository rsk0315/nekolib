macro_rules! impl_is_sprp {
    ( $( ($fn_name:ident, $word:ty, $dword:ty) ),* ) => { $(
        pub const fn $fn_name(n: $word, a: $word) -> bool {
            let s = (n - 1).trailing_zeros();
            let d = n >> s;
            let mut cur = {
                let mut cur = 1;
                let mut pow = d;
                let mut a = a;
                while pow > 0 {
                    if pow & 1 != 0 {
                        cur = (cur as $dword * a as $dword % n as $dword) as $word;
                    }
                    a = ((a as $dword).pow(2) % n as $dword) as $word;
                    pow >>= 1;
                }
                cur
            };
            if cur == 1 {
                return true;
            }
            let mut i = 0;
            while i < s {
                if cur == n - 1 {
                    return true;
                }
                cur = ((cur as $dword).pow(2) % n as $dword) as $word;
                i += 1;
            }
            false
        }
    )* }
}

impl_is_sprp! { (is_sprp_32, u32, u64), (is_sprp_64, u64, u128) }

#[rustfmt::skip]
const BASES: [u16; 256] = [
    0x3CE7, 0x07E2, 0x00A6, 0x1D05, 0x1F80, 0x3EAD, 0x2907, 0x112F,
    0x079D, 0x050F, 0x0AD8, 0x0E24, 0x0230, 0x0C38, 0x145C, 0x0A61,
    0x08FC, 0x07E5, 0x122C, 0x05BF, 0x2478, 0x0FB2, 0x095E, 0x4FEE,
    0x2825, 0x1F5C, 0x08A5, 0x184B, 0x026C, 0x0EB3, 0x12F4, 0x1394,
    0x0C71, 0x0535, 0x1853, 0x14B2, 0x0432, 0x0957, 0x13F9, 0x1B95,
    0x0323, 0x04F5, 0x0F23, 0x01A6, 0x02EF, 0x0244, 0x1279, 0x27FF,
    0x02EA, 0x0B87, 0x022C, 0x089E, 0x0EC2, 0x01E1, 0x05F2, 0x0D94,
    0x01E1, 0x09B7, 0x0CC2, 0x1601, 0x01E8, 0x0D2D, 0x1929, 0x0D10,
    0x0011, 0x3B01, 0x05D2, 0x103A, 0x07F4, 0x075A, 0x0715, 0x01D3,
    0x0CEB, 0x36DA, 0x18E3, 0x0292, 0x03ED, 0x0387, 0x02E1, 0x075F,
    0x1D17, 0x0760, 0x0B20, 0x06F8, 0x1D87, 0x0D48, 0x03B7, 0x3691,
    0x10D0, 0x00B1, 0x0029, 0x4DA3, 0x0C26, 0x33A5, 0x2216, 0x023B,
    0x1B83, 0x1B1F, 0x04AF, 0x0160, 0x1923, 0x00A5, 0x0491, 0x0CF3,
    0x03D2, 0x00E9, 0x0BBB, 0x0A02, 0x0BB2, 0x295B, 0x272E, 0x0949,
    0x076E, 0x14EA, 0x115F, 0x0613, 0x0107, 0x6993, 0x08EB, 0x0131,
    0x029D, 0x0778, 0x0259, 0x182A, 0x01AD, 0x078A, 0x3A19, 0x06F8,
    0x067D, 0x020C, 0x0DF9, 0x00EC, 0x0938, 0x1802, 0x0B22, 0xD955,
    0x06D9, 0x1052, 0x2112, 0x00DE, 0x0A13, 0x0AB7, 0x07EF, 0x08B2,
    0x08E4, 0x0176, 0x0854, 0x032D, 0x5CEC, 0x064A, 0x1146, 0x1427,
    0x06BD, 0x0E0D, 0x0D26, 0x3800, 0x0243, 0x00A5, 0x055F, 0x2722,
    0x3148, 0x2658, 0x055B, 0x0218, 0x074B, 0x2A70, 0x0359, 0x089E,
    0x169C, 0x01B2, 0x1F95, 0x44D2, 0x02D7, 0x0E37, 0x063B, 0x1350,
    0x0851, 0x07ED, 0x2003, 0x2098, 0x1858, 0x23DF, 0x1FBE, 0x074E,
    0x0CE0, 0x1D1F, 0x22F3, 0x61B9, 0x021D, 0x4AAB, 0x0170, 0x0236,
    0x162A, 0x019B, 0x020A, 0x0403, 0x2017, 0x0802, 0x1990, 0x2741,
    0x0266, 0x0306, 0x091D, 0x0BBF, 0x8981, 0x1262, 0x0480, 0x06F9,
    0x0404, 0x0604, 0x0E9F, 0x01ED, 0x117A, 0x09D9, 0x68DD, 0x20A2,
    0x0360, 0x49E3, 0x1559, 0x098F, 0x002A, 0x119F, 0x067C, 0x00A6,
    0x04E1, 0x1873, 0x09F9, 0x0130, 0x0110, 0x1C76, 0x0049, 0x199A,
    0x0383, 0x0B00, 0x144D, 0x3412, 0x1B8E, 0x0B02, 0x0C7F, 0x032B,
    0x039A, 0x015E, 0x1D5A, 0x1164, 0x0D79, 0x0A67, 0x1264, 0x01A2,
    0x0655, 0x0493, 0x0D8F, 0x0058, 0x2C51, 0x019C, 0x0617, 0x00C2,
];

pub const fn is_prime_u8(n: u8) -> bool {
    if n == 2 || n == 3 || n == 5 || n == 7 || n == 11 || n == 13 {
        return true;
    }
    n > 1
        && n % 2 > 0
        && n % 3 > 0
        && n % 5 > 0
        && n % 7 > 0
        && n % 11 > 0
        && n % 13 > 0
}

pub const fn is_prime_u16(n: u16) -> bool { is_prime_u32(n as u32) }

pub const fn is_prime_u32(n: u32) -> bool {
    if n == 2 || n == 3 || n == 5 || n == 7 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 || n % 5 == 0 || n % 7 == 0 {
        return false;
    }
    if n < 121 {
        return n > 1;
    }
    let h = n as u64;
    let h = ((h >> 16) ^ h).wrapping_mul(0x45D9F3B);
    let h = ((h >> 16) ^ h).wrapping_mul(0x45D9F3B);
    let h = ((h >> 16) ^ h) & 0xFF;
    is_sprp_32(n, BASES[h as usize] as u32)
}

pub const fn is_prime_u64(n: u64) -> bool {
    if n == 2 || n == 3 || n == 5 || n == 7 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 || n % 5 == 0 || n % 7 == 0 {
        return false;
    }
    if n < 121 {
        return n > 1;
    }
    let bases = [2, 325, 9375, 28178, 450775, 9780504, 1795265022];
    let mut i = 0;
    while i < bases.len() {
        let b = bases[i];
        if !(b % n == 0 || is_sprp_64(n, b % n)) {
            return false;
        }
        i += 1;
    }
    true
}

#[test]
fn exhaustive_u32() {
    let w = 64;
    let n = 1_usize << 32;
    let is_prime = {
        let mut dp = vec![!0_u64; n / w + 1];
        dp[0] &= !0 << 2;
        for i in (2..=n).take_while(|&i| i <= n / i) {
            let (qi, ri) = (i / w, i % w);
            if dp[qi] >> ri & 1 == 0 {
                continue;
            }
            for j in i..=n / i {
                let (qj, rj) = (i * j / w, i * j % w);
                dp[qj] &= !(1 << rj);
            }
        }
        dp
    };

    for i in 2..n {
        let actual = is_prime_u32(i as u32);
        let expected = is_prime[i / w] >> (i % w) & 1 != 0;
        assert_eq!(actual, expected);
    }
}
