pub trait BinaryOp {
    type Set;
    fn op(&self, lhs: &Self::Set, rhs: &Self::Set) -> Self::Set;
}

pub trait Identity: BinaryOp {
    fn id(&self) -> Self::Set;
}

pub trait Associative {}

pub trait Recip: BinaryOp {
    fn recip(&self, elt: &Self::Set) -> Self::Set;
}

pub trait Commutative {}

pub trait Magma: BinaryOp {}
pub trait Semigroup: BinaryOp + Associative {}
pub trait Monoid: BinaryOp + Associative + Identity {}
pub trait CommutativeMonoid:
    BinaryOp + Associative + Identity + Commutative
{
}
pub trait Group: BinaryOp + Associative + Identity + Recip {}
pub trait CommutativeGroup:
    BinaryOp + Associative + Identity + Recip + Commutative
{
}

impl<T: BinaryOp> Magma for T {}
impl<T: BinaryOp + Associative> Semigroup for T {}
impl<T: BinaryOp + Associative + Identity> Monoid for T {}
impl<T: BinaryOp + Associative + Identity + Commutative> CommutativeMonoid
    for T
{
}
impl<T: BinaryOp + Associative + Identity + Recip> Group for T {}
impl<T: BinaryOp + Associative + Identity + Recip + Commutative>
    CommutativeGroup for T
{
}
