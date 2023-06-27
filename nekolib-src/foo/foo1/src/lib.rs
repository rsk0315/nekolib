use bar1::bar1_fn;

/// ```
/// # use foo1::foo1_fn;
/// assert_eq!(foo1_fn(), ());
/// ```
pub fn foo1_fn() { bar1_fn() }
