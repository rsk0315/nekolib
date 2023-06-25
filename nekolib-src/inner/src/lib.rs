#[macro_export]
macro_rules! doc_inline_reexport {
    ( $($lib:ident,)* ) => { $(
        #[doc(inline)]
        pub use $lib::{self, *};
    )* };
}
