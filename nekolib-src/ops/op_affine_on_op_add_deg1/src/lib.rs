use std::ops::{Add, Mul};

use has_one::HasOne;
use has_zero::HasZero;
use monoid::BinaryOp;
use monoid_action::MonoidAction;
use op_add_deg1::OpAddDeg1;
use op_affine::OpAffine;

#[derive(Clone, Debug)]
pub struct OpAffineOnOpAddDeg1<T> {
    operator: OpAffine<T>,
    operand: OpAddDeg1<T>,
}

impl<T> Default for OpAffineOnOpAddDeg1<T> {
    fn default() -> Self {
        Self {
            operator: OpAffine::default(),
            operand: OpAddDeg1::default(),
        }
    }
}

impl<T> MonoidAction for OpAffineOnOpAddDeg1<T>
where
    T: Eq + Clone + HasZero + HasOne,
    for<'a> &'a T: Add<&'a T, Output = T> + Mul<&'a T, Output = T>,
{
    type Operator = OpAffine<T>;
    type Operand = OpAddDeg1<T>;

    fn operator(&self) -> &Self::Operator { &self.operator }
    fn operand(&self) -> &Self::Operand { &self.operand }

    fn act(
        &self,
        (x0, x1): &<Self::Operand as BinaryOp>::Set,
        (a0, a1): &<Self::Operator as BinaryOp>::Set,
    ) -> <Self::Operand as BinaryOp>::Set {
        // Sum(a0*x0+a1*x1) = a0*Sum(x0) + a1*Sum(x1)
        let y1 = &(a0 * x0) + &(a1 * x1);
        (x0.clone(), y1)
    }
}
