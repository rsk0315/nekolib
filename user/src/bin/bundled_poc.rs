use nekolib_bundled::{foo::foo1_fn, macros::qux1};

fn main() {
    println!("Hello, world!");
    qux1!();
    foo1_fn()
}

// <bundled>
mod nekolib_bundled {
    pub mod macros {
        pub mod qux {
            // #[macro_export]
            macro_rules! qux1 {
                () => {
                    println!("qux!")
                };
            }
            // https://stackoverflow.com/questions/26731243
            pub(crate) use qux1;
        }
        #[allow(unused_imports)]
        pub use qux::*;
    }
    pub mod foo {
        pub mod foo1 {
            pub fn foo1_fn() {}
        }
        pub use foo1::*;
    }
}
// </bundled>
