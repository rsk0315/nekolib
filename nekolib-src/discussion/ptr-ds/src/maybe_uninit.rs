//! 未初期化の値。
//!

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
            // Is `_` allowed because of the invariance?
            &mut *(&mut foo.a[..2] as *mut [MaybeUninit<_>] as *mut [_])
        };
        a[0] = "uno".to_owned();

        let a = unsafe {
            &*(&foo.a[..2] as *const [MaybeUninit<String>] as *const [String])
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
