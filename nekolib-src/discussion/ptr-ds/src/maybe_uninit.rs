//! 未初期化の値。
//!
//! ## Preliminaries
//!
//! 実行時の所望のタイミングで初期化されるような値を作りたいことがある。
//! 次のような Safe Rust のコードはコンパイルエラーとなる。
//!
//! ```compile_fail
//! let a = 123;
//! let mut b;
//! if a % 5 <= 3 { b = "foo"; }
//! if a % 7 <= 4 { b = "bar"; }
//! let _invalid = b; // CE
//! ```
//!
//! [`MaybeUninit`][`std::mem::MaybeUninit`] を使うことで、下記のように書くことができる。
//!
//! ```
//! use std::mem::MaybeUninit;
//!
//! let a = 123;
//! let mut b_uninit = MaybeUninit::<&str>::uninit();
//! let ptr = b_uninit.as_mut_ptr();
//! if a % 5 <= 3 { unsafe { ptr.write("foo") } };
//! if a % 7 <= 4 { unsafe { ptr.write("bar") } };
//! let b = unsafe { b_uninit.assume_init() };
//! assert_eq!(b, "bar");
//! ```
//!
//! `.assume_init()` を呼んだ時点で実際には未初期化だった場合、当然未定義動作となる。
//!
//! ```ignore
//! use std::mem::MaybeUninit;
//!
//! let a = MaybeUninit::<i32>::uninit();
//! let _invalid = unsafe { a.assume_init() };
//! ```
//!
//! ```text
//!  --> src/lib.rs:6:25
//!   |
//! 6 | let _invalid = unsafe { a.assume_init() };
//!   |                         ^^^^^^^^^^^^^^^ using uninitialized data, but this operation requires initialized memory
//! ```
//!
//! `MaybeUninit<T>` をメンバに持つ型 `U` に対する `MaybeUninit<U>` や、`[MaybeUninit<T>; N]`
//! や、`Box<MaybeUninit<T>>` などの例を見てみる。
//!
//! ```
//! use std::{mem::MaybeUninit, ptr};
//!
//! struct Foo {
//!     vector: Vec<i32>,
//!     string: MaybeUninit<String>,
//! }
//!
//! let mut foo = unsafe {
//!     let mut foo_uninit = MaybeUninit::<Foo>::uninit();
//!     let ptr = foo_uninit.as_mut_ptr();
//!     // (*ptr).vector = vec![]; // UB
//!     ptr::addr_of_mut!((*ptr).vector).write(vec![0]);
//!     foo_uninit.assume_init()
//! };
//! assert_eq!(foo.vector, [0]);
//!
//! let s_ptr = foo.string.as_mut_ptr();
//! unsafe {
//!     s_ptr.write("init".to_owned());
//!     assert_eq!(foo.string.assume_init_ref(), "init");
//!     drop(foo.string.assume_init_drop());
//! }
//! ```
//!
//! ```
//! use std::mem::MaybeUninit;
//!
//! // let mut foo_uninit = [MaybeUninit::<String>::uninit(); 3]; // !Copy
//! let mut foo_uninit = unsafe {
//!     MaybeUninit::<[MaybeUninit<String>; 3]>::uninit().assume_init()
//! };
//! unsafe {
//!     foo_uninit[0].write("uno".to_owned());
//!     foo_uninit[1].write("dos".to_owned());
//!
//!     let foo = {
//!         &*(&foo_uninit[..2] as *const [MaybeUninit<_>] as *const [String])
//!     };
//!     assert_eq!(foo[0], "uno");
//!     assert_eq!(foo[1], "dos");
//!
//!     foo_uninit[0].assume_init_drop();
//!     foo_uninit[1].assume_init_drop();
//! }
//! ```
//!
//! ```
//! use std::{mem::MaybeUninit, ptr};
//!
//! struct Foo {
//!     vector: Vec<i32>,
//!     string: String,
//! }
//!
//! let mut foo_uninit = Box::new(MaybeUninit::<Foo>::uninit());
//! let foo: Box<Foo> = unsafe {
//!     let ptr = foo_uninit.as_mut_ptr();
//!     ptr::addr_of_mut!((*ptr).vector).write(vec![0]);
//!     ptr::addr_of_mut!((*ptr).string).write("init".to_owned());
//!     Box::from_raw(Box::leak(foo_uninit).as_mut_ptr())
//! };
//! assert_eq!(foo.vector, [0]);
//! assert_eq!(foo.string, "init");
//! ```
//!
//! レイアウトに関して、`T` と `MaybeUninit<T>` は同様の size および alignment を持つことは保証されている ([ref](https://doc.rust-lang.org/nightly/core/mem/union.MaybeUninit.html#layout-1))。
//! `T` を含む型と `MaybeUninit<T>` を含む型が同様のレイアウトを持つとは限らないが、一旦は忘れておく。
//!
//! 未初期化な領域を含む状態で参照 `&mut T` を取得すると未定義動作となる。`ptr::addr_of_mut!`
//! などを用いて回避する必要がある。
//!
//! ```
//! use std::{mem::MaybeUninit, ptr};
//!
//! struct Foo {
//!     vector: Vec<i32>,
//!     string: String,
//! }
//!
//! let mut foo = unsafe {
//!     let mut foo_uninit = MaybeUninit::<Foo>::uninit();
//!     let ptr = foo_uninit.as_mut_ptr();
//!     // (*ptr).vector = vec![0]; // UB
//!     ptr::addr_of_mut!((*ptr).vector).write(vec![0]);
//!     // (*ptr).string = "init".to_owned(); // UB
//!     ptr::addr_of_mut!((*ptr).string).write("init".to_owned());
//!     (*ptr).string = "re-init".to_owned(); // OK
//!     
//!     foo_uninit.assume_init()
//! };
//! ```
//!
//! 初期化済みの領域に `.write()` するとメモリリークが起きる。
//! あるいは、未初期化の領域を `=` で書き込もうとすると、`*mut T` の dereference なり
//! `&mut T` の取得なりをする必要があるため未定義動作となる。
//!
//! ```
//! use std::mem::MaybeUninit;
//!
//! struct Foo(MaybeUninit<String>);
//!
//! let mut foo = Foo(MaybeUninit::uninit());
//! // unsafe { *foo.0.as_mut_ptr() = "init".to_owned() } // UB
//! // unsafe { *foo.0.assume_init_mut() = "init".to_owned() } // UB
//! foo.0.write("init once".to_owned());
//!
//! unsafe { *foo.0.as_mut_ptr() = "init twice".to_owned() } // OK
//! unsafe { *foo.0.assume_init_mut() = "init thrice".to_owned() } // OK
//! // foo.0.write("init".to_owned()); // memory leak
//!
//! unsafe { foo.0.assume_init_drop() }
//! ```
//!
//! `*mut ()` であれば、未初期化 (dangling) の領域を dereference しても大丈夫。
//!
//! ```
//! use std::mem::MaybeUninit;
//!
//! let unit_uninit = MaybeUninit::<()>::uninit();
//! unsafe {
//!     assert_eq!(unit_uninit.assume_init(), ());
//! }
//! ```

#[cfg(test)]
mod playground {
    use std::{mem::MaybeUninit, ptr};

    #[test]
    fn nested() {
        struct Foo {
            i: i32,
            s: String,
            a: [MaybeUninit<i32>; 3],
        }

        let mut foo = unsafe {
            let mut foo_uninit = MaybeUninit::<Foo>::uninit();
            let ptr = foo_uninit.as_mut_ptr();
            ptr::addr_of_mut!((*ptr).i).write(1);
            // (*foo.as_mut_ptr()).s = "s".to_owned(); // UB
            ptr::addr_of_mut!((*ptr).s).write("s".to_owned());
            foo_uninit.assume_init()
        };

        foo.a[0].write(1);
        assert_eq!(unsafe { foo.a[0].assume_init() }, 1);
    }

    #[test]
    fn array() {
        struct Foo<T, const N: usize> {
            a: [MaybeUninit<T>; N],
        }

        let mut foo =
            unsafe { MaybeUninit::<Foo<String, 3>>::uninit().assume_init() };

        foo.a[0].write("one".to_owned());
        // unsafe { *(foo.a[1].as_mut_ptr()) = "_".to_owned() }; // bad
        // unsafe { *foo.a[1].assume_init_mut() = "_".to_owned() }; // also bad
        foo.a[1].write("two".to_owned());

        assert_eq!(unsafe { foo.a[0].assume_init_ref() }, "one");
        assert_eq!(unsafe { foo.a[1].assume_init_ref() }, "two");

        // foo.a[1].write("_".to_owned()); // bad
        unsafe { *(foo.a[1].as_mut_ptr()) = "zwei".to_owned() }; // ok
        unsafe { *foo.a[1].assume_init_mut() = "deux".to_owned() }; // also ok

        let a = unsafe {
            &mut *(&mut foo.a[..2] as *mut [MaybeUninit<_>] as *mut [String])
        };
        a[0] = "uno".to_owned();

        let a = unsafe {
            &*(&foo.a[..2] as *const [MaybeUninit<_>] as *const [String])
        };
        assert_eq!(a, ["uno", "deux"]);

        unsafe { foo.a[0].assume_init_drop() };
        unsafe { foo.a[1].assume_init_drop() };
    }

    #[test]
    fn boxed() {
        struct Foo {
            a: String,
        }

        let mut foo_uninit = Box::new(MaybeUninit::<Foo>::uninit());
        let foo = unsafe {
            let ptr = foo_uninit.as_mut_ptr();
            ptr::addr_of_mut!((*ptr).a).write("a".to_owned());
            Box::from_raw(Box::leak(foo_uninit).as_mut_ptr())
        };

        assert_eq!(foo.a, "a");
    }
}
