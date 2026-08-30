use std::ops::{AddAssign, Mul};

use bin_iter::BinIter;
use has_one::HasOne;
use has_zero::HasZero;

#[derive(Clone)]
pub struct Matrix<I>(Vec<Vec<I>>);

impl<I> From<Vec<Vec<I>>> for Matrix<I> {
    fn from(value: Vec<Vec<I>>) -> Self { Self(value) }
}

impl<'a, I> Mul<&'a Matrix<I>> for &'a Matrix<I>
where
    I: Copy + AddAssign<I> + Mul<I, Output = I> + HasZero,
{
    type Output = Matrix<I>;
    fn mul(self, other: &'a Matrix<I>) -> Matrix<I> {
        let n1 = self.0.len();
        let n2 = other.0.len();
        let n3 = other.0[0].len();
        let mut res = vec![vec![I::zero(); n3]; n1];
        for i in 0..n1 {
            for j in 0..n3 {
                for k in 0..n2 {
                    res[i][j] += self.0[i][k] * other.0[k][j];
                }
            }
        }
        Matrix(res)
    }
}

impl<'a, I> Mul<&'a Matrix<I>> for Matrix<I>
where
    I: Copy + AddAssign<I> + Mul<I, Output = I> + HasZero,
{
    type Output = Matrix<I>;
    fn mul(self, rhs: &'a Matrix<I>) -> Self::Output { &self * rhs }
}

impl<'a, I> Mul<Matrix<I>> for &'a Matrix<I>
where
    I: Copy + AddAssign<I> + Mul<I, Output = I> + HasZero,
{
    type Output = Matrix<I>;
    fn mul(self, rhs: Matrix<I>) -> Self::Output { self * &rhs }
}

impl<I> Mul<Matrix<I>> for Matrix<I>
where
    I: Copy + AddAssign<I> + Mul<I, Output = I> + HasZero,
{
    type Output = Matrix<I>;
    fn mul(self, rhs: Matrix<I>) -> Self::Output { &self * &rhs }
}

impl<I> Matrix<I>
where
    I: Copy + AddAssign<I> + Mul<I, Output = I> + HasZero + HasOne,
{
    pub fn pow(&self, exp: impl BinIter) -> Self {
        let n = self.0.len();
        let mut res = Self::eye(n);
        let mut dbl = self.clone();
        for b in exp.bin_iter() {
            if b {
                res = &res * &dbl;
            }
            dbl = &dbl * &dbl;
        }
        res
    }

    pub fn eye(n: usize) -> Self {
        let mut res = vec![vec![I::zero(); n]; n];
        for i in 0..n {
            res[i][i] = I::one();
        }
        Self(res)
    }
}

impl<I> Matrix<I> {
    pub fn into_inner(self) -> Vec<Vec<I>> { self.0 }
}
