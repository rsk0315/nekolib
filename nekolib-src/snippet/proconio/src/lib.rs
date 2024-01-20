//! `proconio` crate
//!
//! ## Setup
//!
//! ```zsh
//! % cargo add -F derive proconio
//! ```
//!
//! ```toml
//! # Cargo.toml
//! proconio = { version = "=0.4.5", features = ["derive"] }
//! ```
//!
//! ## `impl Readable`
//!
//! ```
//! use std::io::BufRead;
//!
//! use proconio::{
//!     fastout, input,
//!     marker::Usize1,
//!     source::{Readable, Source},
//! };
//!
//! #[derive(Copy, Clone, Debug, Eq, PartialEq)]
//! enum Query {
//!     Q1(usize, char),
//!     Q2(usize, usize),
//! }
//! use Query::{Q1, Q2};
//!
//! impl Readable for Query {
//!     type Output = Query;
//!     fn read<R: BufRead, S: Source<R>>(source: &mut S) -> Self::Output {
//!         match u32::read(source) {
//!             1 => {
//!                 input! {
//!                     from source,
//!                     x: Usize1,
//!                     c: char,
//!                 }
//!                 Q1(x, c)
//!             }
//!             2 => {
//!                 input! {
//!                     from source,
//!                     l: Usize1,
//!                     r: usize,
//!                 }
//!                 Q2(l, r)
//!             }
//!             _ => unreachable!(),
//!         }
//!     }
//! }
//!
//! #[fastout]
//! fn main() {
//!     input! {
//!
//!     }
//!
//!     unimplemented!()
//! }
//! ```
