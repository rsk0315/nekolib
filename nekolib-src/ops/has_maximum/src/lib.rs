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
