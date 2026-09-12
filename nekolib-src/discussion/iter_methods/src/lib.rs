//! イテレータのメソッド。
//!
//! ```
//! # mod std_partial {
//! pub trait Iterator {
//!     type Item;
//!
//!     fn sum<S>(self) -> S
//!     where
//!         Self: Sized,
//!         S: Sum<Self::Item>,
//!     {
//!         Sum::sum(self)
//!     }
//! }
//!
//! pub trait Sum<A = Self>: Sized {
//!     fn sum<I: Iterator<Item = A>>(iter: I) -> Self;
//! }
//! # }
//!
//! struct Input(u32);
//! #[derive(Debug, PartialEq)] struct Output(u32);
//!
//! # use std::iter::Sum;
//! impl<'a> Sum<&'a Input> for Output {
//!     fn sum<I: Iterator<Item = &'a Input>>(iter: I) -> Output {
//!         Output(iter.map(|&Input(x)| x).sum())
//!     }
//! }
//!
//! let output: Output = [&Input(1), &Input(2), &Input(3)].into_iter().sum();
//! assert_eq!(output, Output(6));
//! ```

#[test]
fn sanity_check() {
    #[derive(Copy, Clone)]
    struct Foo(i32);

    impl<'a> std::iter::Sum<&'a Foo> for i32 {
        fn sum<I: Iterator<Item = &'a Foo>>(iter: I) -> i32 {
            iter.map(|&Foo(x)| x).sum()
        }
    }

    let res: i32 = [&Foo(1), &Foo(2), &Foo(3)].into_iter().sum();
    assert_eq!(res, 6);
}
