#[macro_export]
macro_rules! qux1 {
    () => {
        println!("qux!");
    };
}

#[test]
macro_rules! qux_test {
    () => {};
}

macro_rules! qux_internal {
    ( $($fn:ident,)* ) => { $(
        fn $fn() -> i32 {
            123
        }
    )* }
}

#[macro_export]
macro_rules! qux_long {
    ( ($fn:ident, $val:expr) ) => {
        fn $fn() -> i32 {
            $val
        }
    };
    ( $( ($fn:ident, $val:expr), )* ) => { $(
        $crate::qux_long! { ($fn, $val) }
    )* };
    ( $( ($fn:ident, $val:expr) ),* ) => { $(
        $crate::qux_long! { ($fn, $val) }
    )* };
}
