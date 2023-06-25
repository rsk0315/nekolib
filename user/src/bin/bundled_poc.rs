use nekolib_bundled::{foo::foo1_fn, macros::qux};

fn main() {
    println!("Hello, world!");
    qux!();
    foo1_fn()
}

// <bundled>
mod nekolib_bundled {
    pub mod macros {
        // #[macro_export]
        macro_rules! qux {
            () => {
                println!("qux!")
            };
        }
        // https://stackoverflow.com/questions/26731243
        pub(crate) use qux;
    }
    pub mod foo {
        pub fn foo1_fn() {}
    }
}
// </bundled>
