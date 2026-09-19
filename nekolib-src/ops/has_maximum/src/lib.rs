pub trait HasMaximum: Ord {
    fn maximum() -> Self;
}

macro_rules! impl_has_maximum {
    ($($t:ident)*) => { $(
        impl HasMaximum for $t {
            fn maximum() -> Self { $t::MAX }
        }
    )* }
}
impl_has_maximum! { i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize }

impl HasMaximum for () {
    fn maximum() -> Self { () }
}

impl HasMaximum for char {
    fn maximum() -> Self { char::MAX }
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
        impl<$($t: HasMaximum),+> HasMaximum for ($($t,)+) {
            fn maximum() -> Self { ($(<$t as HasMaximum>::maximum(),)+) }
        }
    }
}

impl_tuples!(E D C B A Z Y X W V U T);

#[test]
fn tuple() {
    assert_eq!(<()>::maximum(), ());
    assert_eq!(<(u8,)>::maximum(), (255,));
    assert_eq!(<(u8, i32)>::maximum(), (255, 2147483647));
    assert_eq!(<(u8, i32, char)>::maximum(), (255, 2147483647, '\u{10ffff}'));
}
