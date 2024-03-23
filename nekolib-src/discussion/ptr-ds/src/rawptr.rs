//! 生ポインタ。
//!
//! ## Preliminaries
//!
//! ### 参照とポインタの作成
//!
//! Safe Rust においては、下記のようなコードはコンパイルエラーとなる。
//!
//! ```compile_fail
//! let mut a = 1;
//! let mut_ref_1 = &mut a;
//! let mut_ref_2 = &mut *mut_ref_1;
//! *mut_ref_2 = 2;
//! *mut_ref_1 = 3;
//! let _invalid = *mut_ref_2; // (!)
//! ```
//!
//! Unsafe Rust において、生ポインタを使うことで、コンパイルを通すことはできる。
//!
//! ```ignore
//! let mut a = 1_u32;
//! let mut_ref_1 = &mut a;
//! let mut_ref_2 = unsafe { &mut *(mut_ref_1 as *mut u32) };
//! *mut_ref_2 = 2;
//! *mut_ref_1 = 3;
//! let _invalid = *mut_ref_2; // (?)
//! ```
//!
//! ただし、（たとえば C++ がそうであるように）コンパイルが通り、プログラムが正常終了したというのは、
//! コードに問題がないということの証明にはならない。実際、Miri でテストすることで未定義動作が検出され、
//! 次のような出力が得られるであろう。
//!
//! ```zsh
//! % cargo miri test
//! ```
//!
//! ```txt
//! error: Undefined Behavior: attempting a read access using <90194> at alloc23256[0x0], but that tag does not exist in the borrow stack for this location
//!  --> src/lib.rs:8:20
//!   |
//! 8 |     let _invalid = *mut_ref_2; // (?)
//!   |                    ^^^^^^^^^^
//!   |                    |
//!   |                    attempting a read access using <90194> at alloc23256[0x0], but that tag does not exist in the borrow stack for this location
//!   |                    this error occurs as part of an access at alloc23256[0x0..0x4]
//! ```
//!
//! こうした検出は Stacked Borrows と呼ばれる機構によって行われている。
//! Miri では Stacked Borrows のサポートは experimental とのことであるが、一旦はこれを信用することにする。
//!
//! さて、safe なスマートポインタとして [`Box`] があるので、それを使ってみよう。
//!
//! ```
//! struct Foo(u32);
//! impl Foo {
//!     pub fn new() -> Self { Foo(0) }
//! }
//!
//! let mut foo = Box::new(Foo::new());
//! assert_eq!(foo.0, 0);
//! foo.0 = 10;
//! assert_eq!(foo.0, 10);
//! ```
//!
//! [`Box::leak`] を用いることで [`&'a mut T`][reference] を取り出すことができる。
//!
//! ```ignore
//! # struct Foo(u32);
//! # impl Foo {
//! #     pub fn new() -> Self { Foo(0) }
//! # }
//! let foo = Box::new(Foo::new());
//! let foo_mut_ref = Box::leak(foo);
//! assert_eq!(foo_mut_ref.0, 0);
//! foo_mut_ref.0 = 10;
//! assert_eq!(foo_mut_ref.0, 10);
//! // (?)
//! ```
//!
//! [`&'a mut T`][reference] は、生ポインタ (raw pointer) [`*mut T`][pointer] にキャストすることもできる。
//! 参照外し (dereference) は `unsafe` となる。
//!
//! ```ignore
//! # struct Foo(u32);
//! # impl Foo {
//! #     pub fn new() -> Self { Foo(0) }
//! # }
//! let foo = Box::new(Foo::new());
//! let foo_mut_ptr: *mut _ = Box::leak(foo);
//! assert_eq!(unsafe { (*foo_mut_ptr).0 }, 0);
//! unsafe { (*foo_mut_ptr).0 = 10 };
//! assert_eq!(unsafe { (*foo_mut_ptr).0 }, 10);
//! // (?)
//! ```
//!
//! ところで、スマートポインタから生ポインタを取り出したままだと、drop
//! されないのでメモリリークしてしまう。
//! これも Miri によって検出でき、次のような出力が得られるであろう。
//!
//! ```txt
//! error: memory leaked: alloc23417 (Rust heap, size: 4, align: 4), allocated here:
//!   --> .../lib/rustlib/src/rust/library/alloc/src/alloc.rs:98:9
//!    |
//! 98 |         __rust_alloc(layout.size(), layout.align())
//!    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//! ```
//!
//! [`Box::into_raw`] にある例に従い、次のようにすることで、`Box` 側に destructor を呼ばせることができる。
//! 逆に、勝手に destructor を呼ばれると困るような状況においては、`Box` を使うと厄介なことになる。
//!
//! ```
//! # struct Foo(u32);
//! # impl Foo {
//! #     pub fn new() -> Self { Foo(0) }
//! # }
//! let foo_mut_ptr: *mut _ = Box::leak(Box::new(Foo::new()));
//! unsafe { drop(Box::from_raw(foo_mut_ptr)) };
//! ```
//!
//! [`*mut T`][pointer] の代わりに [`std::ptr::NonNull`] を用いることもできる。
//!
//! ```
//! use std::ptr::NonNull;
//!
//! # struct Foo(u32);
//! # impl Foo {
//! #     pub fn new() -> Self { Foo(0) }
//! # }
//! let foo_mut_ref = Box::leak(Box::new(Foo::new()));
//! let foo_nonnull = NonNull::from(foo_mut_ref);
//! unsafe { drop(Box::from_raw(foo_nonnull.as_ptr())) };
//! ```
//!
//! `*mut T` に対する `NonNull` の違いは、null でないことの他に covariant
//! であることが挙げられるが、これに関しては [`variance`] を参照せよ。
//!
//! [`variance`]: ../variance/index.html
//!
//! ## See also
//! - [rust-lang / **miri**](https://github.com/rust-lang/miri)
