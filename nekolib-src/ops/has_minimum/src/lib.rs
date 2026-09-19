pub trait HasMinimum: Ord {
    fn minimum() -> Self;
}

macro_rules! impl_has_minimum {
    ($($t:ident)*) => { $(
        impl HasMinimum for $t {
            fn minimum() -> Self { $t::MIN }
        }
    )* }
}
impl_has_minimum! { i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize }

impl HasMinimum for () {
    fn minimum() -> Self { () }
}

impl HasMinimum for char {
    fn minimum() -> Self { char::MIN }
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
    }
}

impl_tuples!(E D C B A Z Y X W V U T);

#[test]
fn tuple() {
    assert_eq!(<()>::minimum(), ());
    assert_eq!(<(u8,)>::minimum(), (0,));
    assert_eq!(<(u8, i32)>::minimum(), (0, -2147483648));
    assert_eq!(<(u8, i32, char)>::minimum(), (0, -2147483648, '\0'));
}
