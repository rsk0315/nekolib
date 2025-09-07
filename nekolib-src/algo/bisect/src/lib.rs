use std::ops::{Range, RangeFrom, RangeTo};

pub trait Bisect {
    type Input;
    type Output;
    fn bisect(&self, pred: impl FnMut(&Self::Input) -> bool) -> Self::Output;
}

macro_rules! impl_bisect_uint {
    ( $($ty:ty)* ) => { $(
        impl Bisect for Range<$ty> {
            type Input = $ty;
            type Output = $ty;
            fn bisect(&self, mut pred: impl FnMut(&$ty) -> bool) -> $ty {
                let Range { start: mut ok, end: mut bad } = *self;
                if !pred(&ok) {
                    return ok;
                }
                while bad - ok > 1 {
                    let mid = ok + (bad - ok) / 2;
                    *(if pred(&mid) { &mut ok } else { &mut bad }) = mid;
                }
                bad
            }
        }
        impl Bisect for RangeFrom<$ty> {
            type Input = $ty;
            type Output = $ty;
            fn bisect(&self, mut pred: impl FnMut(&$ty) -> bool) -> $ty {
                let RangeFrom { start: ok } = *self;
                if !pred(&ok) {
                    return ok;
                }
                let mut w = 1;
                while pred(&(ok + w)) {
                    w *= 2;
                }
                (ok + w / 2..ok + w).bisect(pred)
            }
        }
        impl Bisect for RangeTo<$ty> {
            type Input = $ty;
            type Output = $ty;
            fn bisect(&self, mut pred: impl FnMut(&$ty) -> bool) -> $ty {
                let RangeTo { end: bad } = *self;
                if pred(&bad) {
                    return bad;
                }
                let mut w = 1;
                while !pred(&(bad - w)) {
                    w *= 2;
                }
                (bad - w..bad - w / 2).bisect(pred)
            }
        }
    )* }
}

impl_bisect_uint! { u8 u16 u32 u64 u128 usize }

macro_rules! impl_bisect_int {
    ( $( ($ity:ty, $uty:ty, $i2u:ident, $u2i:ident), )* ) => { $(
        impl Bisect for Range<$ity> {
            type Input = $ity;
            type Output = $ity;
            fn bisect(&self, mut pred: impl FnMut(&$ity) -> bool) -> $ity {
                let Range { start, end } = *self;
                let start = $i2u(start);
                let end = $i2u(end);
                $u2i((start..end).bisect(|&u| pred(&$u2i(u))))
            }
        }
        impl Bisect for RangeFrom<$ity> {
            type Input = $ity;
            type Output = $ity;
            fn bisect(&self, mut pred: impl FnMut(&$ity) -> bool) -> $ity {
                let RangeFrom { start } = *self;
                let start = $i2u(start);
                $u2i((start..).bisect(|&u| pred(&$u2i(u))))
            }
        }
        impl Bisect for RangeTo<$ity> {
            type Input = $ity;
            type Output = $ity;
            fn bisect(&self, mut pred: impl FnMut(&$ity) -> bool) -> $ity {
                let RangeTo { end } = *self;
                let end = $i2u(end);
                $u2i((..end).bisect(|&u| pred(&$u2i(u))))
            }
        }
        fn $i2u(i: $ity) -> $uty { !(!(0 as $uty) >> 1) ^ i as $uty }
        fn $u2i(u: $uty) -> $ity { (!(!0 >> 1) ^ u) as $ity }
    )* }
}

impl_bisect_int! {
    (i8, u8, i2u8, u2i8),
    (i16, u16, i2u16, u2i16),
    (i32, u32, i2u32, u2i32),
    (i64, u64, i2u64, u2i64),
    (i128, u128, i2u128, u2i128),
    (isize, usize, i2usize, u2isize),
}

macro_rules! impl_bisect_float {
    (
        $(
            (
                $fty:ty, $ity:ty, $uty:ty, $w:literal,
                $f2u:ident, $u2f:ident, $mask:ident
            ),
        )*
    ) => { $(
        impl Bisect for Range<$fty> {
            type Input = $fty;
            type Output = $fty;
            fn bisect(&self, mut pred: impl FnMut(&$fty) -> bool) -> $fty {
                let Range { start, end } = *self;
                let start = $f2u(start);
                let end = $f2u(end);
                $u2f((start..end).bisect(|&u| pred(&$u2f(u))))
            }
        }
        impl Bisect for RangeFrom<$fty> {
            type Input = $fty;
            type Output = $fty;
            fn bisect(&self, mut pred: impl FnMut(&$fty) -> bool) -> $fty {
                let RangeFrom { start } = *self;
                let start = $f2u(start);
                $u2f((start..).bisect(|&u| pred(&$u2f(u))))
            }
        }
        impl Bisect for RangeTo<$fty> {
            type Input = $fty;
            type Output = $fty;
            fn bisect(&self, mut pred: impl FnMut(&$fty) -> bool) -> $fty {
                let RangeTo { end } = *self;
                let end = $f2u(end);
                $u2f((..end).bisect(|&u| pred(&$u2f(u))))
            }
        }
        fn $mask(u: $uty) -> $uty {
            ((u as $ity >> ($w - 1)) as $uty >> 1) | 1 << ($w - 1)
        }
        fn $f2u(f: $fty) -> $uty { f.to_bits() ^ $mask(f.to_bits()) }
        fn $u2f(u: $uty) -> $fty { <$fty>::from_bits(u ^ $mask(!u)) }
    )* };
}

impl_bisect_float! {
    (f32, i32, u32, 32, f2u32, u2f32, mask32),
    (f64, i64, u64, 64, f2u64, u2f64, mask64),
}

impl<T> Bisect for [T] {
    type Input = T;
    type Output = usize;
    fn bisect(&self, mut pred: impl FnMut(&T) -> bool) -> usize {
        if self.is_empty() || !pred(&self[0]) {
            return 0;
        }
        let mut ok = 0;
        let mut bad = self.len();
        while bad - ok > 1 {
            let mid = ok + (bad - ok) / 2;
            *(if pred(&self[mid]) { &mut ok } else { &mut bad }) = mid;
        }
        bad
    }
}

#[test]
fn sanity_check() {
    {
        let pred = |&x: &i64| x < 100;
        assert_eq!((0_i64..200).bisect(pred), 100);
        assert_eq!((0_i64..).bisect(pred), 100);
        assert_eq!((..200_i64).bisect(pred), 100);
    }

    {
        let pred = |&x: &i64| x.abs() < 100;
        assert_eq!((0_i64..200).bisect(pred), 100);
        assert_eq!((0_i64..).bisect(pred), 100);
        assert_eq!((..0_i64).bisect(|x| !pred(x)), -99);
    }

    {
        let pred = |&x: &i64| x < 5;
        let a = vec![0, 1, 4, 5, 5, 9];
        assert_eq!(a.bisect(pred), 3);
        assert_eq!(a[4..].bisect(pred), 0);
    }

    {
        let pred = |&x: &f64| 2.0_f64.powf(x) < 3.0;
        assert!(((1.0_f64..2.0).bisect(pred) - 3.0_f64.log2()) <= 5.0e-324);
        // println!("{:.40}", 3.0_f64.log2());
        // println!("{:.40}", (1.0_f64..2.0).bisect(pred));
    }
    {
        assert_eq!([0, 1, 4, 5, 9].bisect(|&x: &i32| x < 5), 3);
        assert_eq!((0..100).bisect(|&x: &i32| x * x < 200), 15);
        assert_eq!((0..).bisect(|&x: &i32| x * x < 200), 15);
    }
    {
        let pred = |&x: &i32| x * x < 200;
        assert_eq!((0..100).bisect(pred), 15);
        assert_eq!((0..).bisect(pred), 15);
    }
}
