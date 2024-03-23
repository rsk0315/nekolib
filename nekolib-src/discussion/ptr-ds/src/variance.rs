//! variance。
//!
//! ## Preliminaries
//!
//! 型 $`S`$, $`T`$ に対して、$`S`$ が $`T`$ の **部分型** (*subtype*) であることを
//! $`S \subtype T`$ と書く。Rust においては、部分型は lifetime の文脈でのみ話題になる[^subtype]。
//! $`S`$ が $`T`$ の満たすべき制約を全て満たしている（必要に応じて追加の制約があってもよい）ことに相当する。
//!
//! [^subtype]: trait に関してはどうか？
//!
//! ### 簡単な lifetime に関する例
//!
//! 有名な例として、Safe Rust では次のようなコードはコンパイル時にエラーになってくれる。
//!
//! ```compile_fail
//! let long = "long".to_owned();
//! let mut dang = &long;
//! {
//!     let short = "short".to_owned();
//!     dang = &short;
//! }
//! let _invalid = dang; // CE
//! ```
//!
//! 生ポインタを介すことでコンパイルエラーを回避できるが、当然これは未定義動作となる。
//!
//! ```ignore
//! let long = "long".to_owned();
//! let mut dang = &long as *const String;
//! {
//!     let short = "short".to_owned();
//!     dang = &short as *const _;
//! }
//! let _invalid = unsafe { (*dang).clone() }; // UB
//! ```
//!
//! Miri によって検出され、次のような出力が得られるであろう。
//!
//! ```text
//! error: Undefined Behavior: out-of-bounds pointer use: alloc943 has been freed, so this pointer is dangling
//!   --> src/lib.rs:9:25
//!    |
//! 9  | let _invalid = unsafe { (*dang).clone() }; // UB
//!    |                         ^^^^^^^ out-of-bounds pointer use: alloc943 has been freed, so this pointer is dangling
//! ```
//!
//! ### Variance
//!
//! さて、`long`, `short` の lifetime をそれぞれ $`\lifetime{long}`$ (`'long`),
//! $`\lifetime{short}`$ (`'short`) とする。
//! $`\lifetime{long}`$ は、$`\lifetime{short}`$ が満たすべき制約（該当の期間を生き延びる）を満たし、
//! さらなる制約（より長い期間を生き延びる）も満たすため、$`\lifetime{long} \subtype \lifetime{short}`$
//! となる[^1]。
//!
//! [^1]: コードの範囲で $`\lifetime{short} \subseteq \lifetime{long}`$ であることから連想して
//! $`\lifetime{short} \subtype \lifetime{long}`$ だと思ってはいけない。
//!
//! ここで次のようなコードを考える。
//!
//! ```
//! fn foo<'a>(lhs: &'a String, rhs: &'a String) {
//!     println!("{lhs} {rhs}");
//! }
//!
//! let long = "long".to_owned();
//! let long_ref = &long;
//! {
//!     let short = "short".to_owned();
//!     let short_ref = &short;
//!     foo(long_ref, short_ref);
//! }
//! ```
//! `long_ref` と `short_ref` は異なる lifetime を持っているが、どちらも `&'a String` として受け取っている。
//! すなわち、`&'long String` を `&'short String` と見做し、どちらも `&'short String` として扱っている。
//! いつでも `'long` を `'short` として扱ってよいということはなく、次のような反例が挙げられる。
//!
//! ```compile_fail
//! fn foo_immut<'a>(_: &&'a String, _: &&'a String) {}
//! fn foo_mut<'a>(_: &mut &'a String, _: &mut &'a String) {}
//!
//! let long = "long".to_owned();
//! let mut long_ref = &long;
//! {
//!     let short = "short".to_owned();
//!     let mut short_ref = &short;
//!     foo_immut(&long_ref, &short_ref); // ok, as before
//!     foo_mut(&mut long_ref, &mut short_ref); // CE
//! }
//! println!("{long_ref}");
//! ```
//!
//! `foo_immut<'a>(..)` に関しては、先の例と同様、`&long_ref: &'short String` として扱うことで解決できる。
//! 一方、`foo_mut<'a>(..)` に関してはそうできずに失敗してしまう。次のいずれも不可能なためである。
//!
//! - `&mut long_ref: &mut &'short String` として扱う (`'a == 'short`)
//! - `&mut short_ref: &mut &'long String` として扱う (`'a == 'long`)
//!
//! 実際、`*long_ref = *short_ref` のようなことができてしまうと、`'short` が終わった時点で
//! `long_ref` が不正なものを指すことになり、困ってしまう[^caller]。
//!
//! [^caller]: 実際には `foo_mut<'a>(..)` は `long_ref` を書き換えていないが、呼び出し側はそのことには関与しない。
//!
//! ```text
//! error[E0597]: `short` does not live long enough
//!   --> src/lib.rs:10:25
//!    |
//! 9  |     let short = "short".to_owned();
//!    |         ----- binding `short` declared here
//! 10 |     let mut short_ref = &short;
//!    |                         ^^^^^^ borrowed value does not live long enough
//! ...
//! 13 | }
//!    | - `short` dropped here while still borrowed
//! 14 | println!("{long_ref}");
//!    |           ---------- borrow later used here
//! ```
//!
//! `'a <: 'b` のときは `&'a T <: &'b T` となるため、`&T` は `'a <: 'b`
//! の関係を保存する操作と見做すことができる。こうした操作を *covariant* (`&'a T` is *covariant* over `'a`)
//! と言う。一方、`S <: T` であっても `&mut S <: &mut T` や `&mut T <: &mut S`
//! は成り立つとは限らない[^mut-invariant]。こうした操作を *invariant* と言う
//! (`&mut T` is *invariant* over `T`)。また、`S <: T` のとき `F<T> <: F<S>`
//! となるような操作も存在し、*contravariant* と言う (`F<T>` is *contravariant* over `T`)。
//!
//! [^mut-invariant]: 上記の例では `S = &'long String`, `T = &'short String` である。`'long <: 'short`
//! より、`S <: T` であることがわかっている。
//!
//! 典型的な例は次の通りである。
//!
//! | generic type | variance |
//! |---|---|
//! | `&T`, `Box<T>`, `Vec<T>`, `*const T` | covariant over `T` |
//! | `&mut T`, `UnsafeCell<T>`, `*mut T` | invariant over `T` |
//! | `fn(T)` | *contra*variant over `T` |
//! | `fn() -> T` | covariant over `T` |
//! | `&'a T`, `&'a mut T` | covariant over `'a` |
//!
//! `fn(T)` が contravariant であることに関して補足しておく。
//! `S <: T` として、`fn(T)` は引数として `S` も `T` も受け取ることができるが、`fn(S)` は
//! `S` のみ受け取ることができる。そのため、`fn(T) <: fn(S)` となっている。
//!
//! `fn(T)` は、制約の言い方でいえば「`T` を受け取ることができる関数である」となることに注意せよ。
//! 一方、`fn() -> T` は「返す値が `T` である関数である」であり、`fn() -> S` は `fn() -> T`
//! の制約も満たしているため、`fn() -> S <: fn() -> T` となる。
//!
//! ## Notes
//!
//! さて、Unsafe Rust の文脈で variance がどのように重要になるのかを整理する必要がある。
//!
//! 生ポインタを介すことで lifetime erasure ができてしまうので、自分で気をつける必要があるということ？
//!
//! ```ignore
//! fn foo_mut<'a>(s: &mut &'a String, t: &mut &'a String) { *s = *t; }
//!
//! let long = "long".to_owned();
//! let mut long_ref = unsafe { &*(&long as *const String) };
//! {
//!     let short = "short".to_owned();
//!     let mut short_ref = unsafe { &*(&short as *const String) };
//!     foo_mut(&mut long_ref, &mut short_ref);
//! }
//! let _invalid = long_ref.clone(); // UB
//! ```
//!
//! ```text
//! error: Undefined Behavior: out-of-bounds pointer use: alloc943 has been freed, so this pointer is dangling
//!   --> src/lib.rs:12:16
//!    |
//! 12 | let _invalid = long_ref.clone(); // UB
//!    |                ^^^^^^^^ out-of-bounds pointer use: alloc943 has been freed, so this pointer is dangling
//! ```
//!
//! ところで、[`std::ptr::NonNull`] のドキュメントにおける一行の説明は下記の通りである。
//!
//! > `*mut T` but non-zero and covariant.
//!
//! `*mut T` は invariant であるから、`*mut T` ではコンパイルエラーになるが
//! `NonNull` ではコンパイルできるような（未定義動作の）例を考えてみよう。
//!
//! ```ignore
//! use std::ptr::NonNull;
//!
//! fn foo_ptrmut<'a>(lhs: *mut &'a String, rhs: *mut &'a String) {
//!     unsafe { *lhs = *rhs };
//! }
//! fn foo_nonnull<'a>(lhs: NonNull<&'a String>, rhs: NonNull<&'a String>) {
//!     unsafe { *lhs.as_ptr() = *rhs.as_ptr() };
//! }
//!
//! let long = "long".to_owned();
//! let mut long_ref = &long;
//! {
//!     let short = "short".to_owned();
//!     let mut short_ref = &short;
//!     // foo_ptrmut(&mut long_ref, &mut short_ref); // CE
//!     foo_nonnull(NonNull::from(&mut long_ref), NonNull::from(&mut short_ref));
//! }
//! let _invalid = long_ref.clone(); // UB
//! ```
//!
//! 当然、out-of-bounds pointer use となるため、こうした処理をしないように気をつける必要がある。
//!
//! ## See also
//!
//! - [Rustonomicon, Subtyping and Variance](https://doc.rust-lang.org/nightly/nomicon/subtyping.html)
//! - [The Rust RFC Book, 0738 Variance](https://rust-lang.github.io/rfcs/0738-variance.html)
