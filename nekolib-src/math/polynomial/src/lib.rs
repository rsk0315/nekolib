use std::{
    collections::VecDeque,
    fmt,
    iter::Product,
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, Div, DivAssign, Mul, MulAssign,
        Neg, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
    },
};

use bin_iter::{BinIter, TryFromBoolIter};
use convolution::{NttFriendly, butterfly, butterfly_inv, convolve};
use factorial_table::FactorialTable;

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
        let recip = M::recip_table(n);
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

    pub fn exp(&self, len: usize) -> Self {
        let zero = M::new(0);
        let one = M::new(1);
        let mut b = Self::from([one, self.get(1)]);
        let mut c = Self::from([one]);
        let mut z2 = Self::from([one, one]);

        let mut cur_len = 2;
        while cur_len < len {
            let m = cur_len;
            cur_len *= 2;

            let mut y = b.clone();
            y.0.resize(2 * m, zero);
            y.fft_butterfly(2 * m);
            let z1 = z2;
            let mut z = &y & &z1;
            z.fft_inv_butterfly(m);
            z.0.resize(m, zero);
            z.0[..m / 2].fill(zero);
            z.fft_butterfly(m);
            z &= -&z1;
            z.fft_inv_butterfly(m);
            c.0.resize(m / 2, zero);
            c.0.extend_from_slice(&z.0[z.0.len().min(m / 2)..]);
            z2 = c.clone();
            z2.fft_butterfly(2 * m);
            let mut x = Self::from(&self.0[..m.min(self.0.len())]);
            x.differentiate();
            x.fft_butterfly(m);
            x &= &y;
            x.fft_inv_butterfly(m);
            x -= b.clone().differential();
            x.0.resize(2 * m, zero);
            for i in 0..m - 1 {
                x.0[m + i] = x.0[i];
                x.0[i] = zero;
            }
            x.fft_butterfly(2 * m);
            x &= &z2;
            x.fft_inv_butterfly(2 * m);
            x.integrate();
            x.0.resize(2 * m, zero);
            for i in m..self.0.len().min(2 * m) {
                x.0[i] += self.0[i];
            }
            x.0[..m].fill(zero);
            x.fft_butterfly(2 * m);
            x &= &y;
            x.fft_inv_butterfly(2 * m);
            b.0.resize(m, zero);
            b.0.extend_from_slice(&x.0[x.0.len().min(m)..]);
        }
        b.truncated(len)
    }

    pub fn pow<I: Copy + Into<M> + BinIter>(&self, k: I, len: usize) -> Self {
        let k_is_zero = k.bin_iter().next().is_none();

        // 0^0 = 1
        if k_is_zero {
            return Self::const_1().truncated(len);
        } else if self.is_zero() || len == 0 {
            return Self::const_0();
        }

        // f(x) = (a_l x^l) (1+g(x))
        let l = (0..).find(|&i| self.0[i].get() != 0).unwrap();
        let a_l = self.0[l];
        let k_as_usize = match usize::try_from_lsb(k.bin_iter()) {
            Some(k) if l < 1 + (len - 1) / k => k,
            _ => return Self::const_0(),
        };

        let g = (self >> l) / a_l;
        let k_mod_p = M::new(k_as_usize);
        let g_pow = (g.log(len) * k_mod_p).exp(len - l * k_as_usize);
        (g_pow << (l * k_as_usize)) * a_l.pow(k)
    }

    pub fn circular(&self, im: &Self, len: usize) -> (Self, Self) {
        let re = self;
        assert_eq!(re.get(0).get(), 0);
        assert_eq!(im.get(0).get(), 0);
        if len == 0 {
            return (Self::const_0(), Self::const_0());
        }

        let zero = M::new(0);
        let one = M::new(1);
        let mut cos = Self::from([one]);
        let mut sin = Self::from([zero]);
        let mut cur_len = 1;
        while cur_len < len {
            cur_len *= 2;

            let mut dcos = cos.clone().differential();
            let mut dsin = sin.clone().differential();
            cos.fft_butterfly(cur_len);
            sin.fft_butterfly(cur_len);
            dcos.fft_butterfly(cur_len);
            dsin.fft_butterfly(cur_len);

            let mut hypot = (&cos & &cos) + (&sin & &sin);
            let mut ecos = (&dcos & &cos) + (&dsin & &sin);
            let mut esin = (&dsin & &cos) - (&dcos & &sin);
            hypot.fft_inv_butterfly(cur_len);
            hypot = hypot.recip(cur_len);
            hypot.fft_butterfly(2 * cur_len);
            ecos.fft_butterfly_double(2 * cur_len);
            esin.fft_butterfly_double(2 * cur_len);

            let mut logcos = &ecos & &hypot;
            let mut logsin = &esin & &hypot;
            logcos.fft_inv_butterfly(2 * cur_len);
            logsin.fft_inv_butterfly(2 * cur_len);
            logcos = logcos.truncated(cur_len - 1).integral();
            logsin = logsin.truncated(cur_len - 1).integral();

            let mut gcos = -logcos + one + re.ref_truncated(cur_len);
            let mut gsin = -logsin + im.ref_truncated(cur_len);
            gcos.fft_butterfly(2 * cur_len);
            gsin.fft_butterfly(2 * cur_len);
            cos.fft_butterfly_double(2 * cur_len);
            sin.fft_butterfly_double(2 * cur_len);

            let mut hcos = (&cos & &gcos) - (&sin & &gsin);
            let mut hsin = (&cos & &gsin) + (&sin & &gcos);
            hcos.fft_inv_butterfly(2 * cur_len);
            hsin.fft_inv_butterfly(2 * cur_len);

            cos = hcos.truncated(cur_len);
            sin = hsin.truncated(cur_len);
        }

        (cos.truncated(len), sin.truncated(len))
    }

    pub fn cos(&self, len: usize) -> Self {
        Self::const_0().circular(self, len).0
    }
    pub fn sin(&self, len: usize) -> Self {
        Self::const_0().circular(self, len).1
    }

    pub fn tan(&self, len: usize) -> Self {
        let (cos, sin) = Self::const_0().circular(self, len);
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

    pub fn const_0() -> Self { Self::new() }
    pub fn const_1() -> Self { Self::from([M::new(1)]) }
    pub fn const_x() -> Self { Self::from([M::new(0), M::new(1)]) }

    pub fn ge1(&self) -> Self {
        let mut res = self.clone();
        if !res.0.is_empty() {
            res.0[0] = M::new(0)
        }
        res.normalize();
        res
    }

    pub fn seq(&self, len: usize) -> Self {
        (Self::const_1() - self).recip(len)
    }
    pub fn set(&self, len: usize) -> Self { self.exp(len) }
    pub fn cyc(&self, len: usize) -> Self { self.seq(len).log(len) }

    pub fn get(&self, i: usize) -> M {
        self.0.get(i).copied().unwrap_or(M::new(0))
    }

    pub fn ogf_into_egf(mut self) -> Self {
        let n = self.0.len();
        if n == 0 {
            return Self::const_0();
        }
        let fact_table = FactorialTable::new(n - 1);
        for i in 0..n {
            self.0[i] *= fact_table.factorial_recip(i);
        }
        self
    }

    pub fn egf_into_ogf(mut self) -> Self {
        let n = self.0.len();
        if n == 0 {
            return Self::const_0();
        }
        let fact_table = FactorialTable::new(n - 1);
        for i in 0..n {
            self.0[i] *= fact_table.factorial(i);
        }
        self
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

    pub fn div_nth(&self, other: &Self, mut n: usize) -> M {
        let mut p = self.clone();
        let mut q = other.clone();
        while n > 0 {
            let d = (2 * q.0.len() - 1).next_power_of_two();
            p.fft_butterfly(d);
            q.fft_butterfly(d);
            let pq_: Vec<_> = (0..d).map(|i| p.get(i) * q.get(i ^ 1)).collect();
            let qq_: Vec<_> =
                (0..d).step_by(2).map(|i| q.get(i) * q.get(i + 1)).collect();
            let [mut pq_, mut qq_]: [Self; 2] = [pq_.into(), qq_.into()];
            pq_.fft_inv_butterfly(d);
            qq_.fft_inv_butterfly(d / 2);
            let u: Vec<_> = (n % 2..d).step_by(2).map(|i| pq_.get(i)).collect();
            p = u.into();
            q = qq_.into();
            n /= 2;
        }
        p.get(0)
    }

    pub fn taylor_shift<I: Into<M>>(&self, shift: I) -> Self {
        if self.is_zero() {
            return Self::const_0();
        }

        let shift = shift.into();
        let n = self.0.len() - 1;
        let pow_table = shift.pow_table(n);
        let fact_table = FactorialTable::new(n);

        let lhs: Vec<_> = (0..=n)
            .map(|i| self.get(n - i) * fact_table.factorial(n - i))
            .collect();
        let rhs: Vec<_> = (0..=n)
            .map(|i| pow_table[i] * fact_table.factorial_recip(i))
            .collect();
        let mut res = (Self::from(lhs) * Self::from(rhs)).into_inner();
        res.resize(n + 1, M::new(0));
        res.reverse();
        for i in 0..=n {
            res[i] *= fact_table.factorial_recip(i);
        }
        res.into()
    }

    pub fn multieval<I: Into<M>>(&self, _xs: &[I]) -> Vec<M> { todo!() }

    pub fn interpolate<I: Into<M>>(_ys: &[I]) -> Self { todo!() }

    pub fn interpolate_arithmetic<I1, I2, I3>(
        _x0: I1,
        _d: I2,
        _ys: &[I3],
    ) -> Vec<M>
    where
        I1: Into<M>,
        I2: Into<M>,
        I3: Into<M>,
    {
        todo!()
    }
}

impl<I: Copy + Into<M>, M: NttFriendly + 'static> From<Vec<I>>
    for Polynomial<M>
{
    fn from(value: Vec<I>) -> Self {
        let value: Vec<_> = value.into_iter().map(|x| x.into()).collect();
        let mut res = Self(value);
        res.normalize();
        res
    }
}

impl<'a, I: Copy + Into<M>, M: NttFriendly + 'static> From<&'a [I]>
    for Polynomial<M>
{
    fn from(value: &'a [I]) -> Self {
        let value: Vec<_> = value.iter().map(|&x| x.into()).collect();
        let mut res = Self(value);
        res.normalize();
        res
    }
}

impl<I: Copy + Into<M>, M: NttFriendly + 'static, const N: usize> From<[I; N]>
    for Polynomial<M>
{
    fn from(value: [I; N]) -> Self {
        let value: Vec<_> = value.iter().map(|&x| x.into()).collect();
        let mut res = Self(value);
        res.normalize();
        res
    }
}

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
        return [M::new(1)].into();
    }
}

#[test]
fn conversion() {
    type Mi = modint::ModInt998244353;
    type Poly = Polynomial<Mi>;
    let _: Poly = vec![1].into();
    let _: Poly = [1].into();
    let _: Poly = [1][..].into();
    let x = Mi::new(1);
    let _: Poly = vec![x].into();
    let _: Poly = [x].into();
    let _: Poly = [x][..].into();
}

#[test]
fn format() {
    type Poly = Polynomial<modint::ModInt998244353>;
    let f: Poly = [1, 0, 2, 3].into();
    assert_eq!(format!("{f}"), "1+2x^2+3x^3");
    assert_eq!(
        format!("{f:?}"),
        "Polynomial { f: [1, 0, 2, 3], mod: 998244353 }"
    );
}

#[test]
fn mul_product_pow() {
    type Poly = Polynomial<modint::ModInt998244353>;
    let f: Poly = [31, 41, 59, 26, 53, 58, 97].into();
    let exp = 93;

    let mul = (0..exp).map(|_| f.clone()).reduce(|x, y| x * y).unwrap();
    let prod: Poly = (0..exp).map(|_| f.clone()).product();
    let pow = f.pow(exp, (f.len() - 1) * exp + 1);

    assert_eq!(prod, mul);
    assert_eq!(pow, mul);
}

#[test]
fn taylor_shift() {
    type Poly = Polynomial<modint::ModInt998244353>;
    let f: Poly = [31, 41, 59, 26, 53, 58, 97].into();
    let shift = 93;
    let f_shift = f.taylor_shift(shift);

    let n = f.len() - 1;
    let actual: Vec<_> = (0..=n).map(|x| f_shift.eval(x)).collect();
    let expected: Vec<_> = (0..=n).map(|x| f.eval(shift + x)).collect();
    assert_eq!(actual, expected);
}

#[test]
fn egf() {
    type Mi = modint::ModInt998244353;
    type Poly = Polynomial<Mi>;
    let n = 8;
    let z = Poly::const_x();

    let surjections = z.set(n).ge1().seq(n).egf_into_ogf();
    let set_partitions = z.set(n).ge1().set(n).egf_into_ogf();
    let alignments = z.cyc(n).seq(n).egf_into_ogf();
    let permutations = z.cyc(n).ge1().set(n).egf_into_ogf();

    assert_eq!(surjections, Poly::from([1, 1, 3, 13, 75, 541, 4683, 47293]));
    assert_eq!(set_partitions, Poly::from([1, 1, 2, 5, 15, 52, 203, 877]));
    assert_eq!(alignments, Poly::from([1, 1, 3, 14, 88, 694, 6578, 72792]));
    assert_eq!(permutations, Poly::from([1, 1, 2, 6, 24, 120, 720, 5040]));

    // T = Z * Set{T}; T(z) = z exp(T(z)); T: [0, 1, 2, 9, 64, 625, ...]
    let t: Vec<_> = (1..n).map(|i| Mi::new(i).pow(i - 1)).collect();
    let t = (Poly::from(t) << 1).ogf_into_egf();
    let u = Poly::const_0().polyeqn(n, |y, n| {
        // T(z) = z exp(T(z))
        // f(y) = x exp(y) - y
        // f(y) / f'(y) = (x exp(y) - y) / (x exp(y) - 1)
        let exp = y.exp(n);
        let num = ((&exp << 1) - y).truncated(n);
        let den = ((&exp << 1) - Poly::const_1()).truncated(n);
        (num * den.recip(n)).truncated(n)
    });
    assert_eq!(u, t);
    assert_eq!((&z * t.exp(n)).truncated(n), t);
}
