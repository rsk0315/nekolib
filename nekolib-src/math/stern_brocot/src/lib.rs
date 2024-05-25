use std::{
    fmt,
    iter::FusedIterator,
    ops::{Add, ControlFlow, Div, Mul, Sub},
};

#[derive(Clone)]
pub struct Fraction<I> {
    pub numer: I,
    pub denom: I,
}

impl<I> Fraction<I> {
    fn mediant(&self, other: &Self) -> Self
    where
        for<'a> &'a I: Add<&'a I, Output = I>,
    {
        Self {
            numer: &self.numer + &other.numer,
            denom: &self.denom + &other.denom,
        }
    }
    fn k_mediant(&self, other: &Self, k: &I) -> Self
    where
        for<'a> &'a I: Add<&'a I, Output = I> + Mul<&'a I, Output = I>,
    {
        let tmp = Self { numer: &other.numer * k, denom: &other.denom * k };
        self.mediant(&tmp)
    }
}

impl<I: fmt::Display> fmt::Display for Fraction<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numer, self.denom)
    }
}

impl<I: fmt::Display> fmt::Debug for Fraction<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numer, self.denom)
    }
}

#[derive(Clone, Debug)]
pub enum ApproxBound<T> {
    Lower(T),
    Upper(T),
}

impl<T> ApproxBound<T> {
    pub fn as_ref(&self) -> &T {
        match self {
            Self::Lower(x) | Self::Upper(x) => x,
        }
    }
    pub fn into_inner(self) -> T {
        match self {
            Self::Lower(x) | Self::Upper(x) => x,
        }
    }
}

pub struct SbTreeUnsigned<I, F> {
    lower: Fraction<I>,
    upper: Fraction<I>,
    pred: F,
    bound: I,
}

impl<I: SbUnsignedInt, F> SbTreeUnsigned<I, F>
where
    for<'a> &'a I: Add<&'a I, Output = I>
        + Sub<&'a I, Output = I>
        + Mul<&'a I, Output = I>
        + Div<&'a I, Output = I>,
    F: FnMut(&Fraction<I>) -> bool,
{
    fn new(pred: F, bound: I) -> Self {
        Self {
            lower: I::frac_0(),
            upper: I::frac_oo(),
            pred,
            bound,
        }
    }
    fn next_approx(
        &mut self,
    ) -> ControlFlow<ApproxBound<Fraction<I>>, ApproxBound<Fraction<I>>> {
        let pred = &mut self.pred;
        let cur = self.lower.mediant(&self.upper);
        assert!(cur.denom <= self.bound);

        let init_tf = pred(&cur);
        let (from, to) = if init_tf {
            (&self.lower, &self.upper)
        } else {
            (&self.upper, &self.lower)
        };

        let mut lo = I::const_1();
        let mut hi = I::const_2();
        while pred(&(from.k_mediant(&to, &hi))) == init_tf {
            lo = &lo + &lo;
            hi = &hi + &hi;
            let tmp = from.k_mediant(&to, &lo);
            if tmp.denom > self.bound {
                // `to.denom != 0`?
                let k = &(&self.bound - &from.denom) / &to.denom;
                let res = from.k_mediant(&to, &k);
                return if init_tf {
                    self.lower = res.clone();
                    ControlFlow::Break(ApproxBound::Lower(res))
                } else {
                    self.upper = res.clone();
                    ControlFlow::Break(ApproxBound::Upper(res))
                };
            }
        }

        while &hi - &lo > I::const_1() {
            let mid = &lo + &(&(&hi - &lo) / &I::const_2());
            let tmp = from.k_mediant(&to, &mid);
            let cur = tmp.denom <= self.bound && pred(&tmp) == init_tf;
            *(if cur { &mut lo } else { &mut hi }) = mid;
        }

        let next = from.k_mediant(&to, &lo);
        let res = if init_tf {
            self.lower = next.clone();
            ApproxBound::Lower(next)
        } else {
            self.upper = next.clone();
            ApproxBound::Upper(next)
        };
        if self.lower.mediant(&self.upper).denom <= self.bound {
            ControlFlow::Continue(res)
        } else {
            ControlFlow::Break(res)
        }
    }
}

impl<I: SbUnsignedInt, F> Iterator for SbTreeUnsigned<I, F>
where
    for<'a> &'a I: Add<&'a I, Output = I>
        + Sub<&'a I, Output = I>
        + Mul<&'a I, Output = I>
        + Div<&'a I, Output = I>,
    F: FnMut(&Fraction<I>) -> bool,
{
    type Item = ControlFlow<ApproxBound<Fraction<I>>, ApproxBound<Fraction<I>>>;

    fn next(&mut self) -> Option<Self::Item> {
        (self.lower.mediant(&self.upper).denom <= self.bound)
            .then(|| self.next_approx())
    }
}

impl<I: SbUnsignedInt, F> FusedIterator for SbTreeUnsigned<I, F>
where
    for<'a> &'a I: Add<&'a I, Output = I>
        + Sub<&'a I, Output = I>
        + Mul<&'a I, Output = I>
        + Div<&'a I, Output = I>,
    F: FnMut(&Fraction<I>) -> bool,
{
}

pub trait FracApprox<F>: Sized {
    fn approx(self, pred: F) -> [ApproxBound<Fraction<Self>>; 2];
    fn approx_iter(self, pred: F) -> SbTreeUnsigned<Self, F>;
}

impl<I: SbUnsignedInt, F> FracApprox<F> for I
where
    for<'a> &'a I: Add<&'a I, Output = I>
        + Sub<&'a I, Output = I>
        + Mul<&'a I, Output = I>
        + Div<&'a I, Output = I>,
    F: FnMut(&Fraction<I>) -> bool,
{
    fn approx(self, pred: F) -> [ApproxBound<Fraction<I>>; 2] {
        use ApproxBound::*;
        let mut lower = Lower(I::frac_0());
        let mut upper = Upper(I::frac_oo());
        for x in self.approx_iter(pred) {
            let x = match x {
                ControlFlow::Continue(x) | ControlFlow::Break(x) => x,
            };
            match x {
                Lower(x) => lower = Lower(x),
                Upper(x) => upper = Upper(x),
            }
        }
        [lower, upper]
    }
    fn approx_iter(self, pred: F) -> SbTreeUnsigned<Self, F> {
        SbTreeUnsigned::new(pred, self)
    }
}

pub trait SbUnsignedInt: Clone + Ord {
    fn const_0() -> Self;
    fn const_1() -> Self;
    fn const_2() -> Self;
    fn frac_0() -> Fraction<Self>;
    fn frac_oo() -> Fraction<Self>;
}

macro_rules! impl_uint {
    ( $($ty:ty)* ) => { $(
        impl SbUnsignedInt for $ty {
            fn const_0() -> $ty { 0 }
            fn const_1() -> $ty { 1 }
            fn const_2() -> $ty { 2 }
            fn frac_0() -> Fraction<$ty> {
                Fraction { numer: Self::const_0(), denom: Self::const_1() }
            }
            fn frac_oo() -> Fraction<$ty> {
                Fraction { numer: Self::const_1(), denom: Self::const_0() }
            }
        }
    )* }
}

impl_uint! { u8 u16 u32 u64 u128 usize }
