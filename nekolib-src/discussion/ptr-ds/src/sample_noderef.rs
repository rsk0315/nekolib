//! 例：node-ref。
//!
//! TODO: 書く。
//!
//! 下記のようなもの。`BorrowType` に `marker::Owned` や `marker::Mut<'a>`
//! などを指定し、それに応じた挙動をするように制御する。
//!
//! ```
//! use std::{marker::PhantomData, ptr::NonNull};
//!
//! struct Foo(u32);
//! struct FooRef<BorrowType> {
//!     foo: NonNull<Foo>,
//!     _marker: PhantomData<BorrowType>,
//! }
//! ```
