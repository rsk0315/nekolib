#![allow(dead_code)]

use std::ops::{Range, RangeBounds, RangeInclusive};

use rs01_dict::Rs01Dict;
use usize_bounds::UsizeBounds;

pub struct WaveletMatrix<I> {
    len: usize,
    bitlen: usize,
    buf: Vec<Rs01Dict>,
    zeros: Vec<usize>,
    orig: Vec<I>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Count3wayResult {
    lt: usize,
    eq: usize,
    gt: usize,
}

impl Count3wayResult {
    fn new(lt: usize, eq: usize, gt: usize) -> Self { Self { lt, eq, gt } }
    pub fn lt(self) -> usize { self.lt }
    pub fn le(self) -> usize { self.lt + self.eq }
    pub fn eq(self) -> usize { self.eq }
    pub fn ge(self) -> usize { self.eq + self.gt }
    pub fn gt(self) -> usize { self.gt }
    pub fn ne(self) -> usize { self.gt + self.lt }
}

impl<I: WmInt> From<Vec<I>> for WaveletMatrix<I> {
    fn from(orig: Vec<I>) -> Self {
        let len = orig.len();
        let bitlen =
            orig.iter().map(|ai| ai.bitlen()).max().unwrap_or(0) as usize;
        let mut whole = orig.clone();
        let mut zeros = vec![0; bitlen];
        let mut buf = vec![];
        for i in (0..bitlen).rev() {
            let mut zero = vec![];
            let mut one = vec![];
            let mut vb = vec![false; len];
            for (j, aj) in whole.into_iter().enumerate() {
                (if aj.test(i) { &mut one } else { &mut zero }).push(aj);
                vb[j] = aj.test(i);
            }
            zeros[i] = zero.len();
            buf.push(Rs01Dict::new(&vb));
            whole = zero;
            whole.append(&mut one);
        }
        buf.reverse();
        Self { len, bitlen, buf, zeros, orig }
    }
}

impl<I: WmInt> WaveletMatrix<I> {
    pub fn count<R: WmIntRange<Int = I>>(
        &self,
        range: impl RangeBounds<usize>,
        value: R,
    ) -> usize {
        self.count_3way(range, value).eq()
    }
    pub fn count_3way<R: WmIntRange<Int = I>>(
        &self,
        range: impl RangeBounds<usize>,
        value: R,
    ) -> Count3wayResult {
        let Range { start: il, end: ir } = range.to_range(self.len);
        let value = value.to_inclusive_range();
        let vl = *value.start();
        let vr = *value.end();
        let (lt, gt) = if vl == vr {
            self.count_3way_internal(il..ir, vl)
        } else {
            let lt = self.count_3way_internal(il..ir, vl).0;
            let gt = self.count_3way_internal(il..ir, vr).1;
            (lt, gt)
        };
        let eq = (ir - il) - (lt + gt);
        Count3wayResult::new(lt, eq, gt)
    }
    fn count_3way_internal(
        &self,
        Range { mut start, mut end }: Range<usize>,
        value: I,
    ) -> (usize, usize) {
        if start == end {
            return (0, 0);
        }
        if value.bitlen() > self.bitlen {
            return (end - start, 0);
        }
        let mut lt = 0;
        let mut gt = 0;
        for i in (0..self.bitlen).rev() {
            let tmp = end - start;
            if !value.test(i) {
                start = self.buf[i].count0(..start);
                end = self.buf[i].count0(..end);
            } else {
                start = self.zeros[i] + self.buf[i].count1(..start);
                end = self.zeros[i] + self.buf[i].count1(..end);
            }
            let len = end - start;
            *(if value.test(i) { &mut lt } else { &mut gt }) += tmp - len;
        }
        (lt, gt)
    }

    pub fn quantile(
        &self,
        range: impl RangeBounds<usize>,
        mut n: usize,
    ) -> Option<I> {
        let Range { mut start, mut end } = range.to_range(self.len);
        if end - start <= n {
            return None;
        }
        let mut res = I::zero();
        for i in (0..self.bitlen).rev() {
            let z = self.buf[i].count0(start..end);
            if n < z {
                start = self.buf[i].count0(..start);
                end = self.buf[i].count0(..end);
            } else {
                res.set(i);
                start = self.zeros[i] + self.buf[i].count1(..start);
                end = self.zeros[i] + self.buf[i].count1(..end);
                n -= z;
            }
        }
        Some(res)
    }
}

pub trait WmInt: Copy + Eq {
    fn test(self, i: usize) -> bool;
    fn set(&mut self, i: usize);
    fn bitlen(self) -> usize;
    fn zero() -> Self;
}

pub trait WmIntRange {
    type Int;
    fn to_inclusive_range(self) -> RangeInclusive<Self::Int>;
}

macro_rules! impl_uint {
    ( $($ty:ty)* ) => { $(
        impl WmInt for $ty {
            fn test(self, i: usize) -> bool { self >> i & 1 != 0 }
            fn set(&mut self, i: usize) { *self |= 1 << i; }
            fn bitlen(self) -> usize {
                let bits = <$ty>::BITS;
                (if self == 0 { 1 } else { bits - self.leading_zeros() }) as _
            }
            fn zero() -> $ty { 0 }
        }
        impl WmIntRange for $ty {
            type Int = $ty;
            fn to_inclusive_range(self) -> RangeInclusive<$ty> { self..=self }
        }
        impl WmIntRange for RangeInclusive<$ty> {
            type Int = $ty;
            fn to_inclusive_range(self) -> RangeInclusive<$ty> { self }
        }
    )* }
}

impl_uint! { u8 u16 u32 u64 u128 usize }
