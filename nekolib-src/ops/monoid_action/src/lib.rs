use monoid::{BinaryOp, Monoid};

pub trait MonoidAction {
    type Operator: Monoid;
    type Operand: Monoid;

    fn operator(&self) -> &Self::Operator;
    fn operand(&self) -> &Self::Operand;
    fn act(
        &self,
        x: &<Self::Operand as BinaryOp>::Set,
        op: &<Self::Operator as BinaryOp>::Set,
    ) -> <Self::Operand as BinaryOp>::Set;
}
