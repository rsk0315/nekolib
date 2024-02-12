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

#[macro_export]
macro_rules! def_monoid_generics {
    (
        $name:ident[$($gen:tt)*] where [$($where:tt)*] =
            ($ty:ty, $op:expr, $id:expr $(,)?)
    ) => {
        struct $name<$($gen)*>(std::marker::PhantomData<fn() -> ($($gen)*)>)
        where $($where)*;
        impl<$($gen)*> $name<$($gen)*>
        where $($where)*
        {
            fn new() -> Self { Self(std::marker::PhantomData) }
        }
        impl<$($gen)*> $crate::BinaryOp for $name<$($gen)*>
        where $($where)*
        {
            type Set = $ty;
            fn op(&self, lhs: &Self::Set, rhs: &Self::Set) -> Self::Set {
                ($op)(lhs, rhs)
            }
        }
        impl<$($gen)*> $crate::Identity for $name<$($gen)*>
        where $($where)*
        {
            fn id(&self) -> Self::Set { ($id)() }
        }
        impl<$($gen)*> $crate::Associative for $name<$($gen)*>
        where $($where)*
        {}
        impl<$($gen)*> Default for $name<$($gen)*>
        where $($where)*
        {
            fn default() -> Self { Self::new() }
        }
    };
    (
        $name:ident[$($gen:tt)*] where [$($where:tt)*] =
            ($ty:ty, $op:expr, $id:expr, Commutative $(,)?)
    ) => {
        $crate::def_monoid_generics! {
            $name[$($gen)*] where [$($where)*] = ($ty, $op, $id)
        }
        impl<$($gen)*> $crate::Commutative for $name<$($gen)*> where $($where)* {}
    };
    (
        $($name:ident[$($gen:tt)*] where [$($where:tt)*] = ($($impl:tt)*)),*
    ) => { $(
        $crate::def_monoid_generics! {
            $name[$($gen)*] where [$($where)*] = ($($impl)*)
        }
    )* };
    (
        $($name:ident[$($gen:tt)*] where [$($where:tt)*] = ($($impl:tt)*),)*
    ) => { $(
        $crate::def_monoid_generics! {
            $name[$($gen)*] where [$($where)*] = ($($impl)*)
        }
    )* };
}

#[macro_export]
macro_rules! def_group_generics {
    (
        $name:ident[$($gen:tt)*] where [$($where:tt)*] =
            ($ty:ty, $op:expr, $id:expr, $recip:expr $(,)?)
    ) => {
        $crate::def_monoid_generics! {
            $name[$($gen)*] where [$($where)*] = ($ty, $op, $id)
        }
        impl<$($gen)*> $crate::Recip for $name<$($gen)*>
        where $($where)*
        {
            fn recip(&self, lhs: &Self::Set) -> Self::Set { ($recip)(lhs) }
        }
    };
    (
        $name:ident[$($gen:tt)*] where [$($where:tt)*] =
            ($ty:ty, $op:expr, $id:expr, $recip:expr, Commutative $(,)?)
    ) => {
        $crate::def_group_generics! {
            $name[$($gen)*] where [$($where)*] = ($ty, $op, $id, $recip)
        }
        impl<$($gen)*> $crate::Commutative for $name<$($gen)*> where $($where)* {}
    };
    (
        $($name:ident[$($gen:tt)*] where [$($where:tt)*] = ($($impl:tt)*)),*
    ) => { $(
        $crate::def_group_generics! {
            $name[$($gen)*] where [$($where)*] = ($($impl)*)
        }
    )* };
    (
        $($name:ident[$($gen:tt)*] where [$($where:tt)*] = ($($impl:tt)*),)*
    ) => { $(
        $crate::def_group_generics! {
            $name[$($gen)*] where [$($where)*] = ($($impl)*)
        }
    )* };
}

#[macro_export]
macro_rules! def_monoid {
    ( $name:ident = ($ty:ty, $op:expr, $id:expr $(,)?) ) => {
        $crate::def_monoid_generics! { $name[] where [] = ($ty, $op, $id) }
    };
    ( $name:ident = ($ty:ty, $op:expr, $id:expr, Commutative $(,)?) ) => {
        $crate::def_monoid_generics! { $name[] where [] = ($ty, $op, $id, Commutative) }
    };
    ( $($name:ident = ($($impl:tt)*)),* ) => { $(
        $crate::def_monoid! { $name = ($($impl)*) }
    )* };
    ( $($name:ident = ($($impl:tt)*),)* ) => { $(
        $crate::def_monoid! { $name = ($($impl)*) }
    )* };
}

#[macro_export]
macro_rules! def_group {
    ( $name:ident = ($ty:ty, $op:expr, $id:expr, $recip:expr $(,)?) ) => {
        $crate::def_group_generics! { $name[] where [] = ($ty, $op, $id, $recip) }
    };
    ( $name:ident = ($ty:ty, $op:expr, $id:expr, $recip:expr, Commutative $(,)?) ) => {
        $crate::def_group_generics! { $name[] where [] = ($ty, $op, $id, $recip, Commutative) }
    };
    ( $($name:ident = ($($impl:tt)*)),* ) => { $(
        $crate::def_group! { $name = ($($impl)*) }
    )* };
    ( $($name:ident = ($($impl:tt)*),)* ) => { $(
        $crate::def_group! { $name = ($($impl)*) }
    )* };
}

#[cfg(test)]
mod tests {
    use std::{
        iter::Sum,
        ops::{Add, BitXor, Neg},
    };

    use super::*;

    #[test]
    fn simple_monoid() {
        def_monoid! {
            OpXor = (u32, |x, y| x ^ y, || 0),
            OpAdd = (i32, |x, y| x + y, || 0),
        }

        let xor = OpXor::new();
        assert_eq!(xor.id(), 0);
        assert_eq!(xor.op(&2, &3), 1);

        let add = OpAdd::new();
        assert_eq!(add.id(), 0);
        assert_eq!(add.op(&2, &3), 5);
    }

    #[test]
    fn simple_group() {
        def_group! {
            OpXor = (u32, |x, y| x ^ y, || 0, |&x: &u32| x),
            OpAdd = (i32, |x, y| x + y, || 0, |&x: &i32| -x),
        }

        let xor = OpXor::new();
        assert_eq!(xor.id(), 0);
        assert_eq!(xor.op(&2, &3), 1);

        let add = OpAdd::new();
        assert_eq!(add.id(), 0);
        assert_eq!(add.op(&2, &3), 5);
    }

    #[test]
    fn generics_monoid() {
        def_monoid_generics! {
            OpXor[T] where [
                for<'a> &'a T: BitXor<Output = T>,
                T: for<'a> Sum<&'a T>,
            ] = (T, |x, y| x ^ y, || None.into_iter().sum(), Commutative),
        }

        let xor = OpXor::<u64>::new();
        assert_eq!(xor.id(), 0);
        assert_eq!(xor.op(&2, &3), 1);
    }

    #[test]
    fn generics_group() {
        def_group_generics! {
            OpXor[T] where [
                for<'a> &'a T: BitXor<Output = T>,
                T: for<'a> Sum<&'a T>,
            ] = (T, |x, y| x ^ y, || None.into_iter().sum(), |x| &(x ^ x) ^ x, Commutative),
            OpAdd[T] where [
                for<'a> &'a T: Add<Output = T> + Neg<Output = T>,
                T: for<'a> Sum<&'a T>,
            ] = (T, |x, y| x + y, || None.into_iter().sum(), |x: &T| -x, Commutative),
        }

        let xor = OpXor::<u64>::new();
        assert_eq!(xor.id(), 0);
        assert_eq!(xor.op(&2, &3), 1);
        assert_eq!(xor.recip(&1), 1);

        let add = OpAdd::<i32>::new();
        assert_eq!(add.id(), 0);
        assert_eq!(add.op(&-1, &2), 1);
        assert_eq!(add.recip(&2), -2);
    }
}
