use nekolib::{foo::foo1_fn, macros::qux1};

fn main() {
    println!("Hello, world!");
    qux1!();
    foo1_fn()
}
