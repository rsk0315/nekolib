pub trait HasMinimum: Ord {
    fn minimum() -> Self;
}
pub trait HasMaximum: Ord {
    fn maximum() -> Self;
}

macro_rules! impl_prim {
    ($($t:ident)*) => { $(
        impl HasMinimum for $t {
            fn minimum() -> Self { $t::MIN }
        }
        impl HasMaximum for $t {
            fn maximum() -> Self { $t::MAX }
        }
    )* }
}
impl_prim! { i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize char }

impl HasMinimum for () {
    fn minimum() -> Self { () }
}
impl HasMaximum for () {
    fn maximum() -> Self { () }
}

macro_rules! impl_tuples {
    ($t:ident) => {
        impl_tuples!(@impl $t);
    };
    ($t:ident $($u:ident)+) => {
        impl_tuples!($($u)+);
        impl_tuples!(@impl $t $($u)+);
    };
    (@impl $($t:ident)+) => {
        impl<$($t: HasMinimum),+> HasMinimum for ($($t,)+) {
            fn minimum() -> Self { ($(<$t as HasMinimum>::minimum(),)+) }
        }
        impl<$($t: HasMaximum),+> HasMaximum for ($($t,)+) {
            fn maximum() -> Self { ($(<$t as HasMaximum>::maximum(),)+) }
        }
    }
}

impl_tuples!(E D C B A Z Y X W V U T);

impl<T: Copy + HasMinimum, const N: usize> HasMinimum for [T; N] {
    fn minimum() -> Self { [T::minimum(); N] }
}
impl<T: Copy + HasMaximum, const N: usize> HasMaximum for [T; N] {
    fn maximum() -> Self { [T::maximum(); N] }
}

macro_rules! impl_empty_collections {
    ( $( [$($generics:tt)*] $ty:ty; )* ) => { $(
        impl<$($generics)*> HasMinimum for $ty {
            fn minimum() -> Self { Default::default() }
        }
    )* };
}

impl_empty_collections! {
    [] String;
    [T: Ord] Vec<T>;
    [T: Ord] Option<T>;
    [T: Ord] std::collections::VecDeque<T>;
    [T: Ord] std::collections::LinkedList<T>;
    [T: Ord] std::collections::BTreeSet<T>;
    [K: Ord, V: Ord] std::collections::BTreeMap<K, V>;
}

impl<T: HasMaximum> HasMinimum for std::cmp::Reverse<T> {
    fn minimum() -> Self { Self(T::maximum()) }
}
impl<T: HasMinimum> HasMaximum for std::cmp::Reverse<T> {
    fn maximum() -> Self { Self(T::minimum()) }
}

impl<T: HasMaximum> HasMaximum for Option<T> {
    fn maximum() -> Self { Some(T::maximum()) }
}

#[test]
fn test_tuple() {
    assert_eq!(<()>::minimum(), ());
    assert_eq!(<(u8,)>::minimum(), (0,));
    assert_eq!(<(u8, i32)>::minimum(), (0, -2147483648));
    assert_eq!(<(u8, i32, char)>::minimum(), (0, -2147483648, '\0'));

    assert_eq!(<()>::maximum(), ());
    assert_eq!(<(u8,)>::maximum(), (255,));
    assert_eq!(<(u8, i32)>::maximum(), (255, 2147483647));
    assert_eq!(<(u8, i32, char)>::maximum(), (255, 2147483647, '\u{10ffff}'));
}

#[test]
fn test_collections() {
    assert_eq!(<[u32; 3]>::minimum(), [0; 3]);
    assert_eq!(Vec::<u32>::minimum(), []);
    assert_eq!(String::minimum(), "");

    assert_eq!(<[u32; 3]>::maximum(), [4294967295; 3]);
}

#[test]
fn test_option() {
    assert_eq!(Option::<i32>::minimum(), None);
    assert_eq!(Option::<i32>::maximum(), Some(2147483647));
}

#[test]
fn test_reverse() {
    use std::cmp::Reverse;

    assert_eq!(Reverse::<i32>::minimum(), Reverse(2147483647));
    assert_eq!(Reverse::<i32>::maximum(), Reverse(-2147483648));

    assert_eq!(Reverse::<Option<i32>>::minimum(), Reverse(Some(2147483647)));
    assert_eq!(Reverse::<Option<i32>>::maximum(), Reverse(None));

    assert_eq!(Option::<Reverse<i32>>::minimum(), None);
    assert_eq!(Option::<Reverse<i32>>::maximum(), Some(Reverse(-2147483648)));

    assert_eq!(Reverse::<String>::maximum(), Reverse("".to_owned()));
}
