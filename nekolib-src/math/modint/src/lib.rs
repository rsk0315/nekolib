use std::{
    fmt,
    hash::{Hash, Hasher},
    iter::{Product, Sum},
    ops::{
        Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign,
    },
};

use bin_iter::BinIter;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct StaticModInt<const MOD: u32>(u32);

impl<const MOD: u32> StaticModInt<MOD> {
    pub fn new(val: impl RemEuclidU32) -> Self { Self::from(val) }
    pub fn modulus() -> u32 { MOD }
}

impl<const MOD: u32> AddAssign for StaticModInt<MOD> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        if self.0 >= MOD {
            self.0 -= MOD;
        }
    }
}

impl<const MOD: u32> SubAssign for StaticModInt<MOD> {
    fn sub_assign(&mut self, rhs: Self) {
        if self.0 < rhs.0 {
            self.0 += MOD;
        }
        self.0 -= rhs.0
    }
}

impl<const MOD: u32> MulAssign for StaticModInt<MOD> {
    fn mul_assign(&mut self, rhs: Self) {
        let tmp = (self.0 as u64) * (rhs.0 as u64) % MOD as u64;
        self.0 = tmp as u32;
    }
}

impl<const MOD: u32> DivAssign for StaticModInt<MOD> {
    fn div_assign(&mut self, rhs: Self) { *self *= rhs.recip() }
}

impl<const MOD: u32> StaticModInt<MOD> {
    pub fn recip(self) -> Self { self.checked_recip().unwrap() }
    // XXX use Euclidean algorithm
    pub fn checked_recip(self) -> Option<Self> { Some(self.pow(MOD - 2)) }
    pub fn pow(self, exp: impl BinIter) -> Self {
        let mut res = Self::new(1);
        let mut dbl = self;
        for b in exp.bin_iter() {
            if b {
                res *= dbl;
            }
            dbl *= dbl;
        }
        res
    }
}

macro_rules! impl_bin_op_inner {
    ( $(
        impl<_> $op_trait:ident<$(&$ltr:lifetime)? Self> for $(&$ltl:lifetime)? Self {
            fn $op:ident(..) -> _ { self.$op_assign:ident() }
        }
    )* ) => { $(
        impl<const MOD: u32> $op_trait<$(&$ltr)? StaticModInt<MOD>> for $(&$ltl)? StaticModInt<MOD> {
            type Output = StaticModInt<MOD>;
            fn $op(self, rhs: $(&$ltr)? StaticModInt<MOD>) -> Self::Output {
                let mut tmp = self.to_owned();
                tmp.$op_assign(rhs);
                tmp
            }
        }
    )* };
}

macro_rules! impl_bin_op {
    ( $( ($op:ident, $op_trait:ident, $op_assign:ident, $op_assign_trait:ident), )* ) => { $(
        impl_bin_op_inner! {
            impl<_> $op_trait<Self> for Self { fn $op(..) -> _ { self.$op_assign() } }
            impl<_> $op_trait<&'_ Self> for Self { fn $op(..) -> _ { self.$op_assign() } }
            impl<_> $op_trait<Self> for &'_ Self { fn $op(..) -> _ { self.$op_assign() } }
            impl<_> $op_trait<&'_ Self> for &'_ Self { fn $op(..) -> _ { self.$op_assign() } }
        }
        impl<const MOD: u32> $op_assign_trait<&Self> for StaticModInt<MOD> {
            fn $op_assign(&mut self, rhs: &Self) { self.$op_assign(rhs.to_owned()) }
        }
    )* }
}

impl_bin_op! {
    ( add, Add, add_assign, AddAssign ),
    ( sub, Sub, sub_assign, SubAssign ),
    ( mul, Mul, mul_assign, MulAssign ),
    ( div, Div, div_assign, DivAssign ),
}

impl<const MOD: u32> Neg for StaticModInt<MOD> {
    type Output = StaticModInt<MOD>;
    fn neg(self) -> Self::Output {
        if self.0 == 0 { self } else { StaticModInt(MOD - self.0) }
    }
}

impl<const MOD: u32> Neg for &StaticModInt<MOD> {
    type Output = StaticModInt<MOD>;
    fn neg(self) -> Self::Output {
        if self.0 == 0 { *self } else { StaticModInt(MOD - self.0) }
    }
}

impl<const MOD: u32> fmt::Display for StaticModInt<MOD> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
}

impl<const MOD: u32> fmt::Debug for StaticModInt<MOD> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} mod {}", self, MOD)
    }
}

pub trait RemEuclidU32 {
    fn rem_euclid_u32(&self, rem: u32) -> u32;
}

macro_rules! impl_rem_euclid_u32 {
    ( $( ($lhs:ty, $common:ty) ),* ) => { $(
        impl RemEuclidU32 for $lhs {
            fn rem_euclid_u32(&self, rem: u32) -> u32 {
                (*self as $common).rem_euclid(rem as $common) as u32
            }
        }
    )* }
}

macro_rules! impl_rem_euclid_u32_small {
    ( $($lhs:ty)* ) => { impl_rem_euclid_u32! { $( ($lhs, u32) ),* } }
}

macro_rules! impl_rem_euclid_u32_large {
    ( $($lhs:ty)* ) => { impl_rem_euclid_u32! { $( ($lhs, $lhs) ),* } }
}

impl_rem_euclid_u32_small! { u8 u16 u32 }
impl_rem_euclid_u32_large! { i64 i128 u64 u128 }
impl_rem_euclid_u32! { (i8, i32), (i16, i32), (i32, i64), (isize, isize), (usize, usize) }

impl<const MOD: u32, I: RemEuclidU32> From<I> for StaticModInt<MOD> {
    fn from(val: I) -> Self { Self(val.rem_euclid_u32(MOD)) }
}

macro_rules! impl_folding_inner {
    ( $(
        impl<$($lt:lifetime,)? _> $op_trait:ident<$(&$ltr:lifetime)? Self> for Self {
            fn $op:ident(..) -> _ { $unit:literal; self.$op_assign:ident($($deref:tt)?) }
        }
    )* ) => { $(
        impl<$($lt,)? const MOD: u32> $op_trait<$(&$ltr)? StaticModInt<MOD>> for StaticModInt<MOD> {
            fn $op<I>(iter: I) -> StaticModInt<MOD>
            where
                I: Iterator<Item = $(&$ltr)? StaticModInt<MOD>>,
            {
                let mut res = StaticModInt::new($unit);
                for x in iter {
                    res.$op_assign(x);
                }
                res
            }
        }
    )* };
}

macro_rules! impl_folding {
    ( $( ($op:ident, $op_trait:ident, $op_assign:ident, $unit:literal), )* ) => { $(
        impl_folding_inner! {
            impl<_> $op_trait<Self> for Self { fn $op(..) -> _ { $unit; self.$op_assign() } }
            impl<'a, _> $op_trait<&'a Self> for Self { fn $op(..) -> _ { $unit; self.$op_assign() } }
        }
    )* }
}

impl_folding! {
    ( sum, Sum, add_assign, 0 ),
    ( product, Product, mul_assign, 1 ),
}

impl<const MOD: u32> Hash for StaticModInt<MOD> {
    fn hash<H: Hasher>(&self, state: &mut H) { self.0.hash(state) }
}

pub type ModInt998244353 = StaticModInt<998244353>;
pub type ModInt1000000007 = StaticModInt<1000000007>;

#[test]
fn arithmetic() {
    type Mi = ModInt998244353;

    let zero = Mi::new(0);
    let half = Mi::new(499122177);
    let quarter = Mi::new(748683265);
    let one = Mi::new(1);
    let two = Mi::new(2);
    assert_eq!(Mi::new(Mi::modulus()), zero);
    assert_eq!(half + half, one);
    assert_eq!(zero - half, -half);
    assert_eq!(one - half, half);
    assert_eq!(half * two, one);
    assert_eq!(half * half, quarter);
    assert_eq!(one / two, half);
    assert_eq!(two.pow(998244352_u64), one);
}

#[test]
fn folding() {
    type Mi = ModInt998244353;

    let a: Vec<_> = [1, 2, 3, 4].iter().copied().map(Mi::new).collect();
    let sum = Mi::new(10);
    let prod = Mi::new(24);
    assert_eq!(a.iter().sum::<Mi>(), sum);
    assert_eq!(a.iter().product::<Mi>(), prod);
    assert_eq!(a.iter().copied().sum::<Mi>(), sum);
    assert_eq!(a.iter().copied().product::<Mi>(), prod);

    let two = Mi::new(2);
    assert_eq!(two.pow(100_u32), [two; 100].iter().product());
}

#[test]
fn fmt() {
    type Mi = ModInt998244353;

    let one = Mi::new(1);
    assert_eq!(format!("{}", one), "1");
    assert_eq!(format!("{:?}", one), "1 mod 998244353");
    assert_eq!(format!("{:?}", [one; 2]), "[1 mod 998244353, 1 mod 998244353]");
}

#[test]
fn conversion() {
    type Mi = ModInt998244353;

    assert_eq!(Mi::new(-1_i8).0, 998244352);
    assert_eq!(Mi::new(-1_i16).0, 998244352);
    assert_eq!(Mi::new(-1_i32).0, 998244352);
    assert_eq!(Mi::new(-1_i64).0, 998244352);
    assert_eq!(Mi::new(-1_i128).0, 998244352);

    assert_eq!(Mi::new(998244354_i32).0, 1);
    assert_eq!(Mi::new(998244354_i64).0, 1);
    assert_eq!(Mi::new(998244354_i128).0, 1);

    assert_eq!(Mi::new(998244354_u32).0, 1);
    assert_eq!(Mi::new(998244354_u64).0, 1);
    assert_eq!(Mi::new(998244354_u128).0, 1);

    assert_eq!(Mi::new(10_i8).0, 10);
    assert_eq!(Mi::new(10_i16).0, 10);
    assert_eq!(Mi::new(10_i32).0, 10);
    assert_eq!(Mi::new(10_i64).0, 10);
    assert_eq!(Mi::new(10_i128).0, 10);
    assert_eq!(Mi::new(10_isize).0, 10);

    assert_eq!(Mi::new(10_u8).0, 10);
    assert_eq!(Mi::new(10_u16).0, 10);
    assert_eq!(Mi::new(10_u32).0, 10);
    assert_eq!(Mi::new(10_u64).0, 10);
    assert_eq!(Mi::new(10_u128).0, 10);
    assert_eq!(Mi::new(10_usize).0, 10);
}
