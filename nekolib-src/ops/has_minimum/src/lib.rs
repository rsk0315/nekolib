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
