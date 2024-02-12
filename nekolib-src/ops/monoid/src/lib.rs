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
macro_rules! impl_monoid_generics {
    (
        $name:ident[$($gen:tt)*] where [$($where:tt)*] =
            ([$ty:ty, $op:expr, $id:expr] [])
    ) => {
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
            ([$ty:ty, $op:expr, $id:expr] [$marker:ident])
    ) => {
        impl_monoid_generics! {
            $name[$($gen)*] where [$($where)*] = ([$ty, $op, $id] [])
        }
        impl<$($gen)*> $crate::$marker for $name<$($gen)*>
        where $($where)*
        {}
    };
}

#[macro_export]
macro_rules! def_monoid_generics {
    (
        $(#[$attr:meta])*
        $name:ident[$($gen:tt)*] where [$($where:tt)*] =
            ($ty:ty, $op:expr, $id:expr $(, $marker:ident)? $(,)?) $(,)?
    ) => {
        $(#[$attr])*
        #[allow(unused_parens)]
        struct $name<$($gen)*>(std::marker::PhantomData<fn() -> ($($gen)*)>)
        where $($where)*;
        $crate::impl_monoid_generics! {
            $name[$($gen)*] where [$($where)*] = ([$ty, $op, $id] [$($marker)?])
        }
    };
    (
        $(#[$attr:meta])*
        pub $name:ident[$($gen:tt)*] where [$($where:tt)*] =
            ($ty:ty, $op:expr, $id:expr $(, $marker:ident)? $(,)?) $(,)?
    ) => {
        $(#[$attr])*
        #[allow(unused_parens)]
        pub struct $name<$($gen)*>(std::marker::PhantomData<fn() -> ($($gen)*)>)
        where $($where)*;
        $crate::impl_monoid_generics! {
            $name[$($gen)*] where [$($where)*] = ([$ty, $op, $id] [$($marker)?])
        }
    };
    (
        $(#[$attr:meta])*
        pub($($vis:tt)+) $name:ident[$($gen:tt)*] where [$($where:tt)*] =
            ($ty:ty, $op:expr, $id:expr $(, $marker:ident)? $(,)?) $(,)?
    ) => {
        $(#[$attr])*
        #[allow(unused_parens)]
        pub($($vis)+) struct $name<$($gen)*>(std::marker::PhantomData<fn() -> ($($gen)*)>)
        where $($where)*;
        $crate::impl_monoid_generics! {
            $name[$($gen)*] where [$($where)*] = ([$ty, $op, $id] [$($marker)?])
        }
    };
}

#[macro_export]
macro_rules! impl_group_generics {
    (
        $name:ident[$($gen:tt)*] where [$($where:tt)*] =
            ([$ty:ty, $op:expr, $id:expr, $recip:expr] [])
    ) => {
        impl_monoid_generics! {
            $name[$($gen)*] where [$($where)*] = ([$ty, $op, $id] [])
        }
        impl<$($gen)*> $crate::Recip for $name<$($gen)*>
        where $($where)*
        {
            fn recip(&self, lhs: &Self::Set) -> Self::Set { ($recip)(lhs) }
        }
    };
    (
        $name:ident[$($gen:tt)*] where [$($where:tt)*] =
            ([$ty:ty, $op:expr, $id:expr, $recip:expr] [$marker:ident])
    ) => {
        impl_group_generics! {
            $name[$($gen)*] where [$($where)*] = ([$ty, $op, $id, $recip] [])
        }
        impl<$($gen)*> $crate::$marker for $name<$($gen)*>
        where $($where)*
        {}
    };
}

#[macro_export]
macro_rules! def_group_generics {
    (
        $(#[$attr:meta])*
        $name:ident[$($gen:tt)*] where [$($where:tt)*] =
            ($ty:ty, $op:expr, $id:expr, $recip:expr $(, $marker:ident)? $(,)?) $(,)?
    ) => {
        $(#[$attr])*
        #[allow(unused_parens)]
        struct $name<$($gen)*>(std::marker::PhantomData<fn() -> ($($gen)*)>)
        where $($where)*;
        $crate::impl_group_generics! {
            $name[$($gen)*] where [$($where)*] = ([$ty, $op, $id, $recip] [$($marker)?])
        }
    };
    (
        $(#[$attr:meta])*
        pub $name:ident[$($gen:tt)*] where [$($where:tt)*] =
            ($ty:ty, $op:expr, $id:expr, $recip:expr $(, $marker:ident)? $(,)?) $(,)?
    ) => {
        $(#[$attr])*
        #[allow(unused_parens)]
        pub struct $name<$($gen)*>(std::marker::PhantomData<fn() -> ($($gen)*)>)
        where $($where)*;
        $crate::impl_group_generics! {
            $name[$($gen)*] where [$($where)*] = ([$ty, $op, $id, $recip] [$($marker)?])
        }
    };
    (
        $(#[$attr:meta])*
        pub($($vis:tt)+) $name:ident[$($gen:tt)*] where [$($where:tt)*] =
            ($ty:ty, $op:expr, $id:expr, $recip:expr $(, $marker:ident)? $(,)?) $(,)?
    ) => {
        $(#[$attr])*
        #[allow(unused_parens)]
        pub($($vis)+) struct $name<$($gen)*>(std::marker::PhantomData<fn() -> ($($gen)*)>)
        where $($where)*;
        $crate::impl_group_generics! {
            $name[$($gen)*] where [$($where)*] = ([$ty, $op, $id, $recip] [$($marker)?])
        }
    };
}

#[macro_export]
macro_rules! def_monoid {
    (
        $(#[$attr:meta])*
        $name:ident = ($ty:ty, $op:expr, $id:expr $(, $marker:ident)? $(,)?)
    ) => {
        $crate::def_monoid_generics! {
            $(#[$attr])* $name[] where [] = ($ty, $op, $id $(, $marker)?)
        }
    };
    (
        $(#[$attr:meta])*
        pub $name:ident = ($ty:ty, $op:expr, $id:expr $(, $marker:ident)? $(,)?)
    ) => {
        $crate::def_monoid_generics! {
            $(#[$attr])* pub $name[] where [] = ($ty, $op, $id $(, $marker)?)
        }
    };
    (
        $(#[$attr:meta])*
        pub($(vis:tt)+) $name:ident = ($ty:ty, $op:expr, $id:expr $(, $marker:ident)? $(,)?)
    ) => {
        $crate::def_monoid_generics! {
            $(#[$attr])* pub($(vis)+) $name[] where [] = ($ty, $op, $id $(, $marker)?)
        }
    };
}

#[macro_export]
macro_rules! def_group {
    (
        $(#[$attr:meta])*
        $name:ident = ($ty:ty, $op:expr, $id:expr, $recip:expr $(, $marker:ident)? $(,)?) ) => {
        $crate::def_group_generics! {
            $(#[$attr])* $name[] where [] = ($ty, $op, $id, $recip $(, $marker)?)
        }
    };
    (
        $(#[$attr:meta])*
        pub $name:ident = ($ty:ty, $op:expr, $id:expr, $recip:expr $(, $marker:ident)? $(,)?) ) => {
        $crate::def_group_generics! {
            $(#[$attr])* pub $name[] where [] = ($ty, $op, $id, $recip $(, $marker)?)
        }
    };
    (
        $(#[$attr:meta])*
        pub($($vis:tt)+) $name:ident = ($ty:ty, $op:expr, $id:expr, $recip:expr $(, $marker:ident)? $(,)?) ) => {
        $crate::def_group_generics! {
            $(#[$attr])* pub($(vis)+) $name[] where [] = ($ty, $op, $id, $recip $(, $marker)?)
        }
    };
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
        def_monoid! { OpXor = (u32, |x, y| x ^ y, || 0) }
        def_monoid! { OpAdd = (i32, |x, y| x + y, || 0) }

        let xor = OpXor::new();
        assert_eq!(xor.id(), 0);
        assert_eq!(xor.op(&2, &3), 1);

        let add = OpAdd::new();
        assert_eq!(add.id(), 0);
        assert_eq!(add.op(&2, &3), 5);
    }

    #[test]
    fn simple_group() {
        def_group! { OpXor = (u32, |x, y| x ^ y, || 0, |&x: &u32| x) }
        def_group! { OpAdd = (i32, |x, y| x + y, || 0, |&x: &i32| -x, Commutative) }

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
            #[allow(non_camel_case_types)]
            Op_Xor[T] where [
                for<'a> &'a T: BitXor<Output = T>,
                T: for<'a> Sum<&'a T>,
            ] = (T, |x, y| x ^ y, || None.into_iter().sum(), Commutative),
        }

        let xor = Op_Xor::<u64>::new();
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
        }
        def_group_generics! {
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
