//! [`take_mut`] と [`replace`] の定義。

use std::{process, ptr};

pub fn take_mut<T>(v: &mut T, change: impl FnOnce(T) -> T) {
    replace(v, |value| (change(value), ()))
}

pub fn replace<T, R>(v: &mut T, change: impl FnOnce(T) -> (T, R)) -> R {
    struct PanicGuard;
    impl Drop for PanicGuard {
        fn drop(&mut self) { process::abort() }
    }
    let guard = PanicGuard;
    let value = unsafe { ptr::read(v) };
    let (new_value, ret) = change(value);
    unsafe {
        ptr::write(v, new_value);
    }
    std::mem::forget(guard);
    ret
}
