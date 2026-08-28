use std::{
    collections::VecDeque,
    fmt,
    iter::Product,
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, Div, DivAssign, Mul, MulAssign,
        Neg, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
    },
};

use convolution::{NttFriendly, butterfly, butterfly_inv, convolve};
use modint::RemEuclidU32;

#[derive(Clone, Eq, PartialEq)]
pub struct Polynomial<M: NttFriendly>(Vec<M>);

impl<M: NttFriendly> fmt::Display for Polynomial<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return write!(f, "0");
        }

        let mut out = false;
        for (i, &c) in self.0.iter().enumerate().filter(|&(_, c)| c.get() > 0) {
            if out {
                write!(f, "+")?;
            }
            match (i, c.get()) {
                (0, c) => write!(f, "{c}")?,
                (1, 1) => write!(f, "x")?,
                (1, c) => write!(f, "{c}x")?,
                (_, 1) => write!(f, "x^{i}")?,
                (_, c) => write!(f, "{c}x^{i}")?,
            }
            out = true;
        }
        Ok(())
    }
}

impl<M: NttFriendly> fmt::Debug for Polynomial<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Polynomial")
            .field("f", &self.0.iter().map(|x| x.get()).collect::<Vec<_>>())
            .field("mod", &M::MOD)
            .finish()
    }
}

impl<M: NttFriendly + 'static> Polynomial<M> {
    pub fn new() -> Self { Self(vec![]) }

    fn normalize(&mut self) {
        if self.0.is_empty() {
            return;
        }
        if let Some(i) = (0..self.0.len()).rev().find(|&i| self.0[i].get() > 0)
        {
            self.0.truncate(i + 1);
        } else {
            self.0.clear();
        }
    }

    pub fn recip(&self, len: usize) -> Self {
        if len == 0 {
            return Self(vec![]);
        }
        let mut res = Self(vec![self.0[0].recip()]);
        let mut cur_len = 1;
        while cur_len < len {
            cur_len *= 2;

            let mut ff = Self(self.0[..self.0.len().min(cur_len)].to_vec());
            let mut gg = res.clone();
            ff.0.resize(cur_len, M::new(0));
            gg.0.resize(cur_len, M::new(0));
            butterfly(&mut ff.0);
            butterfly(&mut gg.0);
            for i in 0..cur_len {
                ff.0[i] *= gg.0[i];
            }
            butterfly_inv(&mut ff.0);
            let iz = M::new(cur_len).recip();
            for i in 0..cur_len / 2 {
                ff.0[i] = M::new(0);
                ff.0[cur_len / 2 + i] = -ff.0[cur_len / 2 + i] * iz;
            }
            butterfly(&mut ff.0);
            for i in 0..cur_len {
                ff.0[i] *= gg.0[i];
            }
            butterfly_inv(&mut ff.0);
            for i in 0..cur_len / 2 {
                ff.0[i] = res.0[i];
                ff.0[cur_len / 2 + i] *= iz;
            }
            res = ff;
        }
        res.truncated(len)
    }

    pub fn truncated(mut self, len: usize) -> Self {
        self.truncate(len);
        self
    }

    pub fn ref_truncated(&self, len: usize) -> Self {
        Self(self.0[..len.min(self.0.len())].to_vec())
    }

    pub fn truncate(&mut self, len: usize) {
        self.0.truncate(len);
        self.normalize();
    }

    pub fn reversed(mut self) -> Self {
        self.reverse();
        self
    }

    pub fn reverse(&mut self) {
        self.0.reverse();
        self.normalize();
    }

    pub fn differential(mut self) -> Self {
        self.differentiate();
        self
    }

    pub fn differentiate(&mut self) {
        if self.0.is_empty() {
            return;
        }
        for i in 1..self.0.len() {
            self.0[i] *= M::new(i);
        }
        self.0.remove(0);
    }

    pub fn integral(mut self) -> Self {
        self.integrate();
        self
    }

    pub fn integrate(&mut self) {
        if self.0.is_empty() {
            return;
        }
        let n = self.0.len();
        let recip = M::recip_table_prime(n);
        for i in 0..n {
            self.0[i] *= recip[i + 1];
        }
        self.0.insert(0, M::new(0));
    }

    pub fn log(&self, len: usize) -> Self {
        assert_eq!(self.0[0].get(), 1);

        let mut diff = self.clone().differential();
        diff *= self.recip(len);
        diff.integrate();
        diff.truncate(len);
        diff
    }

    pub fn exp(&self, _len: usize) -> Self {
        todo!();
    }

    pub fn pow<I: Into<M>>(&self, k: I, len: usize) -> Self {
        let k = k.into();
        let k_ = k.get() as usize;

        // 0^0 = 1
        if k_ == 0 {
            return Self::from([1]).truncated(len);
        } else if self.is_zero() {
            return Self::new();
        }

        // f(x) = (a_l x^l) (1+g(x))
        let l = (0..).find(|&i| self.0[i].get() != 0).unwrap();
        let a_l = self.0[l];
        if len <= l * k_ {
            return Self::new();
        }

        let g = (self >> l) / a_l;
        let g_pow = (g.log(len) * k).exp(len - l * k_);
        (g_pow << (l * k_)) * a_l.pow(k_ as u64)
    }

    pub fn circular(&self, _im: &Self, _len: usize) -> (Self, Self) { todo!() }

    pub fn cos(&self, len: usize) -> Self { Self::new().circular(self, len).0 }
    pub fn sin(&self, len: usize) -> Self { Self::new().circular(self, len).1 }

    pub fn tan(&self, len: usize) -> Self {
        let (cos, sin) = Self::new().circular(self, len);
        (sin * cos.recip(len)).truncated(len)
    }

    pub fn polyeqn(
        mut self,
        n: usize,
        f_over_df: impl Fn(&Self, usize) -> Self, // f(y0)/f'(y0)
    ) -> Self {
        if self.0.is_empty() {
            self.0.push(M::new(0));
        }
        let mut d = self.0.len();
        let mut y = self;
        while d < n {
            d *= 2;
            y -= f_over_df(&y, d).truncated(d);
        }
        y.truncated(n)
    }

    pub fn fode(
        mut self,
        n: usize,
        f_df: impl Fn(&Self, usize) -> (Self, Self),
    ) -> Self {
        if self.0.is_empty() {
            self.0.push(M::new(0))
        }
        let mut d = self.0.len();
        let mut y = self;
        while d < n {
            d *= 2;
            let (f, df) = f_df(&y, d);
            let h = f - y.clone().differential();
            let u = (-df).integral().exp(d);
            y += (y.recip(d) * (u * h).truncated(d).integral()).truncated(d);
        }
        y.truncated(n)
    }

    pub fn get(&self, i: usize) -> M {
        self.0.get(i).copied().unwrap_or(M::new(0))
    }

    pub fn eval(&self, t: impl Into<M>) -> M {
        let t = t.into();
        let mut ft = M::new(0);
        for &a in self.0.iter().rev() {
            ft *= t;
            ft += a;
        }
        ft
    }

    pub fn into_inner(self) -> Vec<M> { self.0 }

    pub fn fft_butterfly(&mut self, len: usize) {
        let ceil_len = len.next_power_of_two();
        self.0.resize(ceil_len, M::new(0));
        butterfly(&mut self.0);
        self.normalize();
    }

    pub fn fft_inv_butterfly(&mut self, len: usize) {
        let ceil_len = len.next_power_of_two();
        self.0.resize(ceil_len, M::new(0));
        butterfly_inv(&mut self.0);
        self.0.truncate(len);
        let iz = M::new(ceil_len).recip();
        for c in &mut self.0 {
            *c *= iz;
        }
        self.normalize();
    }

    pub fn fft_butterfly_double(&mut self, to_len: usize) {
        if self.is_zero() {
            return;
        }

        let mut dbl = self.clone();
        let g = M::new(M::PRIMITIVE_ROOT);
        let zeta = g.pow((M::MOD as u64 - 1) / (to_len as u64));

        dbl.fft_inv_butterfly(to_len / 2);
        let mut r = M::new(1);
        for i in 0..dbl.0.len() {
            dbl.0[i] *= r;
            r *= zeta;
        }
        dbl.fft_butterfly(to_len / 2);
        self.0.resize(to_len / 2, M::new(0));
        self.0.append(&mut dbl.0);
    }

    pub fn is_zero(&self) -> bool { self.0.is_empty() }

    pub fn len(&self) -> usize { self.0.len() }

    pub fn div_mod(&self, other: &Self) -> (Self, Self) {
        let q = self / other;
        let r = self - &q * other;
        (q, r)
    }

    pub fn div_nth(&self, _other: &Self, _n: usize) -> M { todo!() }

    pub fn taylor_shift(&self, _i: usize) -> Self { todo!() }

    pub fn multieval(&self, _xs: &[M]) -> Vec<M> { todo!() }

    pub fn interpolate(_ys: &[M]) -> Self { todo!() }
}

impl<I: Copy + RemEuclidU32, M: NttFriendly + 'static> From<Vec<I>>
    for Polynomial<M>
{
    fn from(value: Vec<I>) -> Self {
        let value: Vec<_> = value.into_iter().map(M::new).collect();
        let mut res = Self(value);
        res.normalize();
        res
    }
}

impl<'a, I: Copy + RemEuclidU32, M: NttFriendly + 'static> From<&'a [I]>
    for Polynomial<M>
{
    fn from(value: &'a [I]) -> Self {
        let value: Vec<_> = value.iter().map(|&x| M::new(x)).collect();
        let mut res = Self(value);
        res.normalize();
        res
    }
}

impl<I: Copy + RemEuclidU32, M: NttFriendly + 'static, const N: usize>
    From<[I; N]> for Polynomial<M>
{
    fn from(value: [I; N]) -> Self {
        let value: Vec<_> = value.iter().map(|&x| M::new(x)).collect();
        let mut res = Self(value);
        res.normalize();
        res
    }
}

// impl<M: NttFriendly> From<Vec<M>> for Polynomial<M>
// impl<'a, M: NttFriendly> From<[&a' M]> for Polynomial<M>
// impl<M: NttFriendly, const N: usize> From<[M; N]> for Polynomial<M>

// Polynomial<M> @= Polynomial<M>

impl<'a, M: NttFriendly + 'static> AddAssign<&'a Polynomial<M>>
    for Polynomial<M>
{
    fn add_assign(&mut self, other: &'a Polynomial<M>) {
        let n = self.0.len().max(other.0.len());
        self.0.resize(n, M::new(0));
        for i in 0..other.0.len() {
            self.0[i] += other.0[i];
        }
        self.normalize();
    }
}

impl<M: NttFriendly + 'static> AddAssign for Polynomial<M> {
    fn add_assign(&mut self, other: Self) { *self += &other; }
}

impl<'a, M: NttFriendly + 'static> SubAssign<&'a Polynomial<M>>
    for Polynomial<M>
{
    fn sub_assign(&mut self, other: &'a Polynomial<M>) {
        let n = self.0.len().max(other.0.len());
        self.0.resize(n, M::new(0));
        for i in 0..other.0.len() {
            self.0[i] -= other.0[i];
        }
        self.normalize();
    }
}

impl<M: NttFriendly + 'static> SubAssign for Polynomial<M> {
    fn sub_assign(&mut self, other: Self) { *self -= &other; }
}

impl<'a, M: NttFriendly + 'static> MulAssign<&'a Polynomial<M>>
    for Polynomial<M>
{
    fn mul_assign(&mut self, other: &'a Polynomial<M>) {
        self.mul_assign(other.clone());
    }
}

impl<M: NttFriendly + 'static> MulAssign for Polynomial<M> {
    fn mul_assign(&mut self, other: Polynomial<M>) {
        self.0 = convolve(std::mem::take(&mut self.0), other.0);
        self.normalize();
    }
}

impl<'a, M: NttFriendly + 'static> DivAssign<&'a Polynomial<M>>
    for Polynomial<M>
{
    fn div_assign(&mut self, other: &'a Polynomial<M>) {
        *self /= other.clone();
    }
}

impl<M: NttFriendly + 'static> DivAssign for Polynomial<M> {
    fn div_assign(&mut self, mut other: Polynomial<M>) {
        let deg = self.0.len() - other.0.len();
        self.reverse();
        other.reverse();
        *self *= other.recip(deg + 1);
        self.0.resize(deg + 1, M::new(0));
        self.reverse();
    }
}

impl<'a, M: NttFriendly + 'static> RemAssign<&'a Polynomial<M>>
    for Polynomial<M>
{
    fn rem_assign(&mut self, other: &'a Polynomial<M>) {
        *self %= other.clone();
    }
}

impl<M: NttFriendly + 'static> RemAssign for Polynomial<M> {
    fn rem_assign(&mut self, other: Polynomial<M>) {
        let div = &*self / &other;
        *self -= div * other;
    }
}

impl<'a, M: NttFriendly + 'static> BitAndAssign<&'a Polynomial<M>>
    for Polynomial<M>
{
    fn bitand_assign(&mut self, other: &'a Polynomial<M>) {
        self.0.truncate(other.0.len());
        for (ai, &bi) in self.0.iter_mut().zip(&other.0) {
            *ai *= bi;
        }
        self.normalize();
    }
}

impl<M: NttFriendly + 'static> BitAndAssign for Polynomial<M> {
    fn bitand_assign(&mut self, other: Polynomial<M>) { *self &= &other; }
}

// Polynomial<M> @= M

impl<'a, M: NttFriendly + 'static> AddAssign<&'a M> for Polynomial<M> {
    fn add_assign(&mut self, &other: &'a M) { *self += other; }
}

impl<M: NttFriendly + 'static> AddAssign<M> for Polynomial<M> {
    fn add_assign(&mut self, other: M) {
        if other.get() == 0 {
            return;
        }
        if self.0.is_empty() {
            self.0.push(other);
        } else {
            self.0[0] += other;
        }
        self.normalize();
    }
}

impl<'a, M: NttFriendly + 'static> SubAssign<&'a M> for Polynomial<M> {
    fn sub_assign(&mut self, &other: &'a M) { *self -= other; }
}

impl<M: NttFriendly + 'static> SubAssign<M> for Polynomial<M> {
    fn sub_assign(&mut self, other: M) {
        if other.get() == 0 {
            return;
        }
        if self.0.is_empty() {
            self.0.push(-other);
        } else {
            self.0[0] -= other;
        }
        self.normalize();
    }
}

impl<'a, M: NttFriendly> MulAssign<&'a M> for Polynomial<M> {
    fn mul_assign(&mut self, &other: &'a M) { *self *= other; }
}

impl<M: NttFriendly> MulAssign<M> for Polynomial<M> {
    fn mul_assign(&mut self, other: M) {
        if other.get() == 0 {
            self.0.clear();
            return;
        }
        if self.0.is_empty() {
            return;
        }
        for c in &mut self.0 {
            *c *= other;
        }
    }
}

impl<'a, M: NttFriendly> DivAssign<&'a M> for Polynomial<M> {
    fn div_assign(&mut self, &other: &'a M) { *self /= other; }
}

impl<M: NttFriendly> DivAssign<M> for Polynomial<M> {
    fn div_assign(&mut self, other: M) {
        assert_ne!(other.get(), 0);
        if self.0.is_empty() {
            return;
        }
        let other_recip = other.recip();
        for c in &mut self.0 {
            *c *= other_recip;
        }
    }
}

impl<'a, M: NttFriendly> RemAssign<&'a M> for Polynomial<M> {
    fn rem_assign(&mut self, &other: &'a M) { *self %= other; }
}

impl<M: NttFriendly> RemAssign<M> for Polynomial<M> {
    fn rem_assign(&mut self, other: M) {
        assert_ne!(other.get(), 0);
        if self.0.is_empty() {
            return;
        }
        self.0.clear();
    }
}

impl<'a, M: NttFriendly + 'static> BitAndAssign<&'a M> for Polynomial<M> {
    fn bitand_assign(&mut self, &other: &'a M) { *self &= other; }
}

impl<M: NttFriendly + 'static> BitAndAssign<M> for Polynomial<M> {
    fn bitand_assign(&mut self, other: M) {
        if self.0.is_empty() {
            return;
        }
        if other.get() == 0 {
            self.0.clear();
        } else {
            self.0.truncate(1);
            self.0[0] *= other;
            self.normalize();
        }
    }
}

macro_rules! impl_binop {
    ( $( ($op:ident, $op_assign:ident, $op_trait:ident, $op_assign_trait:ident), )* ) => { $(
        // &'a Polynomial<M> @ Polynomial<M>
        impl<'a, M: NttFriendly + 'static> $op_trait<Polynomial<M>> for &'a Polynomial<M> {
            type Output = Polynomial<M>;
            fn $op(self, other: Polynomial<M>) -> Polynomial<M> {
                self.clone().$op(other)
            }
        }
        // Polynomial<M> @ &'a Polynomial<M>
        impl<'a, M: NttFriendly + 'static> $op_trait<&'a Polynomial<M>> for Polynomial<M> {
            type Output = Polynomial<M>;
            fn $op(mut self, other: &'a Polynomial<M>) -> Polynomial<M> {
                self.$op_assign(other);
                self
            }
        }
        // &'a Polynomial<M> @ &'a Polynomial<M>
        impl<'a, M: NttFriendly + 'static> $op_trait<&'a Polynomial<M>> for &'a Polynomial<M> {
            type Output = Polynomial<M>;
            fn $op(self, other: &'a Polynomial<M>) -> Polynomial<M> {
                self.clone().$op(other)
            }
        }
        // Polynomial<M> @ Polynomial<M>
        impl<M: NttFriendly + 'static> $op_trait for Polynomial<M> {
            type Output = Polynomial<M>;
            fn $op(mut self, other: Polynomial<M>) -> Polynomial<M> {
                self.$op_assign(other);
                self
            }
        }

        // &'a Polynomial<M> @ M
        impl<'a, M: NttFriendly + 'static> $op_trait<M> for &'a Polynomial<M> {
            type Output = Polynomial<M>;
            fn $op(self, other: M) -> Polynomial<M> {
                self.clone().$op(other)
            }
        }
        // Polynomial<M> @ &'a M
        impl<'a, M: NttFriendly + 'static> $op_trait<&'a M> for Polynomial<M> {
            type Output = Polynomial<M>;
            fn $op(mut self, other: &'a M) -> Polynomial<M> {
                self.$op_assign(other);
                self
            }
        }
        // &'a Polynomial<M> @ &'a M
        impl<'a, M: NttFriendly + 'static> $op_trait<&'a M> for &'a Polynomial<M> {
            type Output = Polynomial<M>;
            fn $op(self, other: &'a M) -> Polynomial<M> {
                self.clone().$op(other)
            }
        }
        // Polynomial<M> @ M
        impl<M: NttFriendly + 'static> $op_trait<M> for Polynomial<M> {
            type Output = Polynomial<M>;
            fn $op(mut self, other: M) -> Polynomial<M> {
                self.$op_assign(other);
                self
            }
        }
    )* }
}

impl_binop! {
    (add, add_assign, Add, AddAssign),
    (sub, sub_assign, Sub, SubAssign),
    (mul, mul_assign, Mul, MulAssign),
    (div, div_assign, Div, DivAssign),
    (rem, rem_assign, Rem, RemAssign),
    (bitand, bitand_assign, BitAnd, BitAndAssign),
}

impl<M: NttFriendly> Neg for Polynomial<M> {
    type Output = Polynomial<M>;
    fn neg(mut self) -> Polynomial<M> {
        for c in &mut self.0 {
            *c = -*c;
        }
        self
    }
}

impl<'a, M: NttFriendly> Neg for &'a Polynomial<M> {
    type Output = Polynomial<M>;
    fn neg(self) -> Polynomial<M> { -self.clone() }
}

impl<M: NttFriendly> ShlAssign<usize> for Polynomial<M> {
    fn shl_assign(&mut self, sh: usize) {
        if !self.0.is_empty() {
            self.0.splice(0..0, (0..sh).map(|_| M::new(0)));
        }
    }
}

impl<M: NttFriendly> Shl<usize> for Polynomial<M> {
    type Output = Polynomial<M>;
    fn shl(mut self, sh: usize) -> Polynomial<M> {
        self <<= sh;
        self
    }
}

impl<'a, M: NttFriendly> Shl<usize> for &'a Polynomial<M> {
    type Output = Polynomial<M>;
    fn shl(self, sh: usize) -> Polynomial<M> { self.clone() << sh }
}

impl<M: NttFriendly> ShrAssign<usize> for Polynomial<M> {
    fn shr_assign(&mut self, sh: usize) {
        if !self.0.is_empty() {
            self.0.splice(0..sh.min(self.0.len()), None);
        }
    }
}

impl<M: NttFriendly> Shr<usize> for Polynomial<M> {
    type Output = Polynomial<M>;
    fn shr(mut self, sh: usize) -> Polynomial<M> {
        self >>= sh;
        self
    }
}

impl<'a, M: NttFriendly> Shr<usize> for &'a Polynomial<M> {
    type Output = Polynomial<M>;
    fn shr(self, sh: usize) -> Polynomial<M> { self.clone() >> sh }
}

impl<M: NttFriendly + 'static> Product<Polynomial<M>> for Polynomial<M> {
    fn product<I: Iterator<Item = Polynomial<M>>>(iter: I) -> Self {
        let mut q: VecDeque<_> = iter.collect();
        while let Some(lhs) = q.pop_front() {
            if let Some(rhs) = q.pop_front() {
                q.push_back(lhs * rhs);
            } else {
                return lhs;
            }
        }
        return [1].into();
    }
}

#[test]
fn format() {
    type Poly = Polynomial<modint::ModInt998244353>;
    let f: Poly = vec![1, 0, 2, 3].into();
    assert_eq!(format!("{f}"), "1+2x^2+3x^3");
    assert_eq!(
        format!("{f:?}"),
        "Polynomial { f: [1, 0, 2, 3], mod: 998244353 }"
    );
}
