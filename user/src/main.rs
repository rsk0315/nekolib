use nekolib::{foo::foo1_fn, macros::qux};

fn main() {
    println!("Hello, world!");
    qux!();
    foo1_fn()
}
