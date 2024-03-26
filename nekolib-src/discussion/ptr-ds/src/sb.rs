//! Stacked Borrows。
//!
//! ## See also
//!
//! - [Learning Rust With Entirely Too Many Linked Lists, Attempting To Understand Stacked Borrows](https://rust-unofficial.github.io/too-many-lists/fifth-stacked-borrows.html)
//! - [rust-lang / **miri**, Stacked Borrows Implementation](https://github.com/rust-lang/miri/blob/367082342f6287ffe209d956a33115f3d1d024e7/src/borrow_tracker/stacked_borrows/mod.rs#L166)
//! - [rust-lang / **unsafe-code-guidelines**, Stacked Borrows](https://github.com/rust-lang/unsafe-code-guidelines/blob/master/wip/stacked-borrows.md)
//! - [paper on Stacked Borrows](https://plv.mpi-sws.org/rustbelt/stacked-borrows/)

#[test]
fn stack() {
    use std::ptr::NonNull;

    let mut foo = 1;
    let mut ptr1 = NonNull::from(&mut foo);

    let ptr2 = ptr1.as_ptr(); // does not invalidate
    assert_eq!(unsafe { *ptr1.as_ptr() }, 1);
    assert_eq!(unsafe { *ptr2 }, 1);
    unsafe {
        *ptr1.as_ptr() += 10;
        *ptr2 -= 10;
    } // also ok

    // -Zmiri-tree-borrows allows following (?)

    let ptr2 = unsafe { &mut *ptr1.as_ptr() }; // invalidates
    assert_eq!(unsafe { *ptr1.as_ptr() }, 1);
    assert_eq!(*ptr2, 1);

    let ptr2 = unsafe { ptr1.as_mut() }; // invalidates
    assert_eq!(unsafe { *ptr1.as_ptr() }, 1);
    assert_eq!(*ptr2, 1);
}

#[test]
fn sb2() {
    unsafe {
        let x = &mut 0;
        let raw1 = x as *mut _;
        let tmp = &mut *raw1;
        let raw2 = tmp as *mut _;
        *raw1 = 1;
        let _val = *raw2;
    }
}

#[cfg(test)]
mod treebor {
    #[test]
    fn const_ptr() {
        let a = 1_u32;
        let const_ptr = &a as *const u32;
        let mut_ref =
            Box::leak(unsafe { Box::from_raw(const_ptr as *mut u32) });
        let mut_ptr = mut_ref as *mut u32;
        assert_eq!(unsafe { *mut_ptr }, 1);
        unsafe { *mut_ptr += 1 }; // UB
    }

    #[test]
    fn frozen() {
        let mut a = 1_u32;
        let mut_a = &mut a;
        let mut_p = mut_a as *mut u32;
        unsafe { *mut_p += 1 };
        let shr_a = &a;
        let shr_p = shr_a as *const u32;
        assert_eq!(unsafe { *shr_p }, 2);
        assert_eq!(unsafe { *mut_p }, 2);
    }
}
