//! ポインタ系データ構造。
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
//! [`NonNull`][`std::ptr::NonNull`] は、null でないことの他に variance
//! にも気をつける必要があるが、後述とする。
//!
//! ### 生ポインタの使用
//!
//! 構造体 `Foo` と、それを参照する構造体 `FooRef<BorrowType>` を考える。
//! `BorrowType` は今後追加していくが、一旦 `Boxed<Foo>` のように振る舞う `marker::Owned`
//! と、`&'a Foo` のように振る舞う `marker::Immut<'a>` を考える。
//!
//! ```
//! use std::{marker::PhantomData, ptr::NonNull};
//!
//! struct Foo(u32);
//! struct FooRef<BorrowType> {
//!     foo: NonNull<Foo>,
//!     _marker: PhantomData<BorrowType>,
//! }
//!
//! mod marker {
//!     use std::marker::PhantomData;
//!
//!     pub enum Owned {}
//!     pub struct Immut<'a>(PhantomData<&'a ()>);
//! }
//!
//! impl Foo {
//!     pub fn new() -> Box<Self> { Box::new(Self(0)) }
//! }
//! impl FooRef<marker::Owned> {
//!     pub fn new_ref() -> Self {
//!         Self { foo: NonNull::from(Box::leak(Foo::new())), _marker: PhantomData }
//!     }
//! }
//! impl<BorrowType> FooRef<BorrowType> {
//!     pub fn borrow(&self) -> FooRef<marker::Immut<'_>> {
//!         FooRef { foo: self.foo, _marker: PhantomData }
//!     }
//!     pub fn get(&self) -> &u32 { unsafe { &(*self.foo.as_ptr()).0 } }
//! }
//!
//! let foo_ref = FooRef::new_ref();
//! assert_eq!(*foo_ref.get(), 0);
//!
//! let foo_ref_immut = foo_ref.borrow();
//! assert_eq!(*foo_ref_immut.get(), 0);
//!
//! assert_eq!(*foo_ref.get(), 0);
//! assert_eq!(*foo_ref_immut.get(), 0);
//!
//! unsafe { drop(Box::from_raw(foo_ref.foo.as_ptr())) };
//! ```
//!
//! 当然、`Immut<'a>` だけでなく `Mut<'a>` も欲しい。
//! `drop` に関しては、`Boxed<Foo>` のように振る舞うところの `FooRef<marker::Owned>`
//! でのみ実装したいが、そういうことはできない ([E0366])。
//! ここでは、手動で呼び出すための `.drop()` を提供しつつ `Boxed<Foo>` のように振る舞う
//! `FooRef<marker::Dying>` を別途作ることにする。
//!
//! また、連想配列を実装することを見据えると、一部のメンバ変数についてのみ可変参照を公開したいこともある。
//! ここでは `struct Foo(u32, u32);` の `foo: Foo` に対して、`&foo.0` と `&mut foo.1`
//! を公開する `FooRef<marker::SndMut<'a>>` を作ることにする。
//! このとき、`&foo.0` にアクセスしても `&mut foo.1` が invalidate
//! されないように気をつける必要がある（逆も然り）。
//!
//! [E0366]: https://doc.rust-lang.org/error_codes/E0366.html
//!
//! 加えて、`marker::Mut<'a>` の lifetime を消して、一時的に静的解析の対象外にできる
//! `marker::DormantMut` も作っておく[^dormant]。もちろん、冒頭の例で触れたように
//! Stacked Borrows のルールには従う必要がある。
//!
//! [^dormant]: dormant や (re-)awaken という表現がしばしば使われている印象がある。
//!
//! 別の marker を返すようなメソッドにおいては、`FooRef { foo: self.foo, _marker: PhantomData }`
//! の boilerplate を都度書く必要があってややうれしくない。`PhantomData`
//! の型パラメータを陽に書く必要がないので、見かけ上は全く同じものになっている。
//! また、どの marker からどの marker への遷移をできるかを意識しておくとよいかもしれない。
//!
//! 他にも、`FooRef<marker::Immut<'a>>` は `Copy`/`Clone`
//! であるとか、諸々の変換などの実装が欲しくなるであろう。
//!
//! ### 参照の無効化
//!
//! さて、`marker::Mut<'a>` で `.get_mut()` のようなメソッドを公開するにあたり、
//! 無効な参照を返さないように気をつける必要がある。
//! 最初の例のように、下記のコードは未定義動作となる。
//!
//! ```
//! struct Foo(u32);
//! let mut foo = Foo(0);
//! ```
//!
//! ```ignore
//! # struct Foo(u32);
//! # let mut foo = Foo(0);
//! let foo_ptr_1: *mut _ = &mut foo;
//! let foo_ptr_2: *mut _ = &mut foo;
//! unsafe { (*foo_ptr_1).0 }; // UB
//! ```
//!
//! [`std::ptr`] には「`&mut foo` を `*mut _` にキャストするときは他の参照があってはいけない」とあり、
//! `foo_ptr_1` が生きている状態で `&mut foo` しているのが悪いように読める。Stacked Borrows
//! のルール的には、二つ目の `&mut foo` をした時点で `foo_ptr_1` が無効化されるので、
//! その後の参照が未定義動作になるという説明になると思われる。
//! 同じ可変参照から作ったポインタであれば問題ないようである。
//!
//! ```
//! # struct Foo(u32);
//! # let mut foo = Foo(0);
//! let mut foo_mut = &mut foo;
//! let foo_ptr_1: *mut _ = foo_mut;
//! let foo_ptr_2: *mut _ = foo_mut;
//! unsafe { (*foo_ptr_1).0 = 10 };
//! unsafe { (*foo_ptr_2).0 = 20 };
//! assert_eq!(foo.0, 20);
//! ```
//!
//! [`NonNull`][`std::ptr::NonNull`] を介して生ポインタを作っても、[`.as_mut()`][`std::ptr::NonNull::as_mut`]
//! を使って可変参照を作ってしまうとうまくいかない。
//!
//! ```ignore
//! use std::ptr::NonNull;
//!
//! # struct Foo(u32);
//! # let mut foo = Foo(0);
//! let mut foo_nonnull = NonNull::from(&mut foo);
//! let foo0_mut_1 = unsafe { &mut foo_nonnull.as_mut().0 };
//! let foo0_mut_2 = unsafe { &mut foo_nonnull.as_mut().0 };
//! *foo0_mut_1 = 10; // UB
//! ```
//!
//! ```ignore
//! use std::ptr::NonNull;
//!
//! # struct Foo(u32);
//! # let mut foo = Foo(0);
//! let mut foo_nonnull = NonNull::from(&mut foo);
//! let foo_mut_1 = unsafe { foo_nonnull.as_mut() };
//! let foo_mut_2 = unsafe { foo_nonnull.as_mut() };
//! foo_mut_1.0 = 10; // UB
//! ```
//!
//! 下記も同様。
//!
//! ```ignore
//! use std::ptr::NonNull;
//!
//! # struct Foo(u32);
//! # let mut foo = Foo(0);
//! let mut foo_nonnull = NonNull::from(&mut foo);
//! let foo0_mut_1 = unsafe { &mut (*foo_nonnull.as_ptr()).0 };
//! let foo0_mut_2 = unsafe { &mut (*foo_nonnull.as_ptr()).0 };
//! *foo0_mut_1 = 10; // UB
//! ```
//!
//! 今度は、メンバを複数持つような場合を考える。
//!
//! ```
//! struct Foo(u32, u32);
//! let mut foo = Foo(0, 0);
//! ```
//!
//! ```
//! # struct Foo(u32, u32);
//! # let mut foo = Foo(0, 0);
//! let foo0_mut = &mut foo.0;
//! let foo1_mut = &mut foo.1;
//! *foo0_mut = 10;
//! *foo1_mut = 20;
//! assert_eq!((foo.0, foo.1), (10, 20)); // ok
//! ```
//!
//! Safe Rust では上記のように書けるが、事情があって生ポインタを使う必要がある状況とする。
//!
//! ```
//! use std::ptr::NonNull;
//!
//! # struct Foo(u32, u32);
//! # let mut foo = Foo(0, 0);
//! let foo_nonnull = NonNull::from(&mut foo);
//! let foo0_mut = unsafe { &mut (*foo_nonnull.as_ptr()).0 };
//! let foo1_mut = unsafe { &mut (*foo_nonnull.as_ptr()).1 };
//! *foo0_mut = 10;
//! *foo1_mut = 20;
//! assert_eq!((foo.0, foo.1), (10, 20)); // ok
//! ```
//!
//! 下記のように [`std::ptr::NonNull::as_mut`] を使うと、アクセスしていない部分も無効化されるように見える。
//!
//! ```ignore
//! use std::ptr::NonNull;
//!
//! # struct Foo(u32, u32);
//! # let mut foo = Foo(0, 0);
//! let mut foo_nonnull = NonNull::from(&mut foo);
//! let foo0_mut = unsafe { &mut foo_nonnull.as_mut().0 };
//! let foo1_mut = unsafe { &mut foo_nonnull.as_mut().1 };
//! *foo0_mut = 10; // UB
//! ```
//!
//! [`std::ptr::NonNull::as_ptr`] を使うと無効化されない模様。
//!
//! ```
//! use std::ptr::{addr_of_mut, NonNull};
//!
//! # struct Foo(u32, u32);
//! # let mut foo = Foo(0, 0);
//! let mut foo_nonnull = NonNull::from(&mut foo);
//! let foo0_mut = unsafe { &mut *addr_of_mut!((*foo_nonnull.as_ptr()).0) };
//! let foo1_mut = unsafe { &mut *addr_of_mut!((*foo_nonnull.as_ptr()).1) };
//! *foo0_mut = 10;
//! *foo1_mut = 20;
//! assert_eq!((foo.0, foo.1), (10, 20)); // ok
//! ```
//!
//! [`std::ptr::addr_of_mut`] を使わなくても問題ない模様 (cf. [`&`/`&mut`](https://doc.rust-lang.org/reference/expressions/operator-expr.html#borrow-operators), [`*`](https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-dereference-operator), [place expressions](https://doc.rust-lang.org/reference/expressions.html#place-expressions-and-value-expressions))。`addr_of_mut` は dereferenceability
//! を無視したいときに使うもので、無効化を防ぎたいときに使うものではない？
//!
//! ```
//! use std::ptr::{addr_of_mut, NonNull};
//!
//! # struct Foo(u32, u32);
//! # let mut foo = Foo(0, 0);
//! let mut foo_nonnull = NonNull::from(&mut foo);
//! let foo0_mut = unsafe { &mut (*foo_nonnull.as_ptr()).0 };
//! let foo1_mut = unsafe { &mut (*foo_nonnull.as_ptr()).1 };
//! *foo0_mut = 10;
//! *foo1_mut = 20;
//! assert_eq!((foo.0, foo.1), (10, 20)); // ok
//! ```
//!
//! ### 可変な参照を返す例
//!
//! 上記を踏まえ、`marker::Mut<'a>` などを含めた `FooRef<BorrowType>` を考える。
//!
//! | `BorrowType` | like ... |
//! |---|---|
//! | `marker::Owned` | `Boxed<Foo>` |
//! | `marker::Dying` | `Boxed<Foo>` (\*1) |
//! | `marker::Immut<'a>` | `&'a Foo` |
//! | `marker::Mut<'a>` | `&'a mut Foo` |
//! | `marker::SndMut<'a>` | `&'a mut Foo` (\*2) |
//!
//! (\*1): `drop` 相当のメソッドを提供する。
//! (\*2): `foo.0` (fst) に関しては不変参照、`foo.1` (snd) に関しては可変参照を公開する。
//!
//! ```ignore
//! use std::{marker::PhantomData, ptr::NonNull};
//!
//! struct Foo(u32, u32);
//! struct FooRef<BorrowType> {
//!     foo: NonNull<Foo>,
//!     _marker: PhantomData<BorrowType>,
//! }
//!
//! todo!();
//! ```
//!
//! TODO: 使い方の例として、テストめいたものを書く。
//!
//! ### 簡単な例
//!
//! TODO: 簡単な doubly-linked list めいたものを書く。
//!
//! ### それ以外
//!
//! TODO: variance を気にするべき例を書く。
//!
//! TODO: Stacked Borrows のルールを陽に書いた方がいい？
//!
//! ## References
//! - [The Rustonomicon](https://doc.rust-lang.org/nomicon/)
//! - [Learn Rust With Entirely Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/)
//! - [`alloc::collections::btree::node`](https://doc.rust-lang.org/src/alloc/collections/btree/node.rs.html)
//! - [rust-lang / **unsafe-code-guidelines** :: /wip/**stacked-borrows.md**](https://github.com/rust-lang/unsafe-code-guidelines/blob/master/wip/stacked-borrows.md)
//! - [rust-lang / **miri**](https://github.com/rust-lang/miri)

pub mod variance;

use std::{
    marker::PhantomData,
    ptr::{self, NonNull},
};

pub struct Foo {
    key: u32,
    val: u32,
}
pub struct FooRef<BorrowType> {
    foo: NonNull<Foo>,
    _marker: PhantomData<BorrowType>,
}

pub mod marker {
    use std::marker::PhantomData;

    pub enum Owned {}
    pub enum Dying {}
    pub enum DormantMut {}
    pub struct Immut<'a>(PhantomData<&'a ()>);
    pub struct Mut<'a>(PhantomData<&'a ()>);
    pub struct ValMut<'a>(PhantomData<&'a ()>);
}

impl Foo {
    pub fn new() -> Box<Self> { Box::new(Self { key: 0, val: 0 }) }
}

impl FooRef<marker::Owned> {
    pub fn new_foo() -> Self {
        Self {
            foo: NonNull::from(Box::leak(Foo::new())),
            _marker: PhantomData,
        }
    }
    pub fn borrow_mut(&mut self) -> FooRef<marker::Mut<'_>> {
        FooRef { foo: self.foo, _marker: PhantomData }
    }
    pub fn borrow_valmut(&mut self) -> FooRef<marker::ValMut<'_>> {
        FooRef { foo: self.foo, _marker: PhantomData }
    }
    pub fn into_dying(self) -> FooRef<marker::Dying> {
        FooRef { foo: self.foo, _marker: PhantomData }
    }
}

impl<BorrowType> FooRef<BorrowType> {
    pub fn reborrow(&self) -> FooRef<marker::Immut<'_>> {
        FooRef { foo: self.foo, _marker: PhantomData }
    }
    fn as_ptr(this: &Self) -> *mut Foo { this.foo.as_ptr() }
}

impl Copy for FooRef<marker::Immut<'_>> {}
impl Clone for FooRef<marker::Immut<'_>> {
    fn clone(&self) -> Self { *self }
}

impl<'a> FooRef<marker::Mut<'a>> {
    pub unsafe fn reborrow_mut(&mut self) -> FooRef<marker::Mut<'_>> {
        FooRef { foo: self.foo, _marker: PhantomData }
    }
    pub fn dormant(&self) -> FooRef<marker::DormantMut> {
        FooRef { foo: self.foo, _marker: PhantomData }
    }
    fn as_mut(&mut self) -> &mut Foo {
        let ptr = Self::as_ptr(self);
        unsafe { &mut *ptr }
    }
    pub fn key_mut(&mut self) -> &mut u32 { &mut self.as_mut().key }
    pub fn val_mut(&mut self) -> &mut u32 { &mut self.as_mut().val }
}

impl<'a> FooRef<marker::ValMut<'a>> {
    pub fn into_key_valmut(mut self) -> (&'a u32, &'a mut u32) {
        let ptr = Self::as_ptr(&mut self);
        let key = unsafe { &*ptr::addr_of!((*ptr).key) };
        let val = unsafe { &mut *ptr::addr_of_mut!((*ptr).val) };
        (key, val)
    }
}

impl FooRef<marker::DormantMut> {
    pub unsafe fn awaken<'a>(self) -> FooRef<marker::Mut<'a>> {
        FooRef { foo: self.foo, _marker: PhantomData }
    }
}

impl FooRef<marker::Dying> {
    pub fn drop(self) { unsafe { drop(Box::from_raw(Self::as_ptr(&self))) } }
}

#[test]
fn test_foo_ref() {
    let mut foo_ref = FooRef::new_foo();

    let mut foo_ref_mut_1 = foo_ref.borrow_mut();
    let key_mut = foo_ref_mut_1.key_mut();
    assert_eq!(*key_mut, 0);
    *key_mut += 10;
    assert_eq!(*key_mut, 10);
    let val_mut = foo_ref_mut_1.val_mut();
    assert_eq!(*val_mut, 0);
    *val_mut += 100;
    assert_eq!(*val_mut, 100);

    let mut foo_ref_mut_2 = unsafe { foo_ref_mut_1.reborrow_mut() };
    let key_mut = foo_ref_mut_2.key_mut();
    assert_eq!(*key_mut, 10);
    *key_mut += 10;
    assert_eq!(*key_mut, 20);
    let val_mut = foo_ref_mut_2.val_mut();
    assert_eq!(*val_mut, 100);
    *val_mut += 100;
    assert_eq!(*val_mut, 200);

    let foo_dormant = foo_ref_mut_2.dormant();

    let foo_ref_kvm = foo_ref.borrow_valmut();
    let (key, val) = foo_ref_kvm.into_key_valmut();
    assert_eq!(*key, 20);
    assert_eq!(*val, 200);
    *val += 1;
    assert_eq!(*key, 20);
    assert_eq!(*val, 201);

    let mut foo_ref_mut_3 = unsafe { foo_dormant.awaken() };
    let key_mut = foo_ref_mut_3.key_mut();
    assert_eq!(*key_mut, 20);
    *key_mut += 10;
    assert_eq!(*key_mut, 30);
    let val_mut = foo_ref_mut_3.val_mut();
    assert_eq!(*val_mut, 201);
    *val_mut += 100;
    assert_eq!(*val_mut, 301);

    // *val += 1; // UB

    foo_ref.into_dying().drop();
}

#[test]
fn ptr_read() {
    struct Foo(u32, u32);

    let mut foo = Foo(0, 0);

    // {
    //     let foo_ptr_fst: *mut _ = &mut foo;
    //     let foo_ptr_snd: *mut _ = &mut foo;
    //     // unsafe { (*foo_ptr_fst).0 = 10 }; // UB
    // }

    // {
    //     let mut foo_nonnull = NonNull::from(&mut foo);
    //     let foo_mut_fst = unsafe { &mut foo_nonnull.as_mut().0 };
    //     let foo_mut_snd = unsafe { &mut foo_nonnull.as_mut().1 };
    //     *foo_mut_fst = 10; // UB
    // }

    {
        let foo_nonnull = NonNull::from(&mut foo);
        let foo_ptr_1 = foo_nonnull.as_ptr();
        let foo_ptr_2 = foo_nonnull.as_ptr();
        unsafe { (*foo_ptr_1).0 += 1 };
        unsafe { (*foo_ptr_1).1 += 2 };
        unsafe { (*foo_ptr_2).0 += 10 };
        unsafe { (*foo_ptr_2).1 += 20 };
        unsafe { (*foo_ptr_1).0 += 10 };
        unsafe { (*foo_ptr_1).1 += 10 };
        unsafe { (*foo_ptr_2).0 -= 10 };
        unsafe { (*foo_ptr_2).1 -= 10 };
        assert_eq!((foo.0, foo.1), (11, 22)); // ok

        unsafe { (*foo_ptr_1).0 }; // ok
        foo.0 += 100; // invalidates `foo_ptr_1` and `foo_nonnull`
        // unsafe { (*foo_ptr_1).0 }; // UB
    }

    {
        let foo_nonnull = NonNull::from(&mut foo);
        let foo_ptr_1 = foo_nonnull.as_ptr();
        unsafe { foo_ptr_1.read().0 };
        let _ptr = NonNull::from(&mut foo); // invalidates them
        // unsafe { foo_ptr_1.read().0 }; // UB
    }
}
