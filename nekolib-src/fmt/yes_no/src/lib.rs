use std::fmt;

pub struct YesNo(pub bool);

impl fmt::Display for YesNo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (if self.0 { "Yes" } else { "No" }).fmt(f)
    }
}

#[test]
fn sanity_check() {
    assert_eq!(format!("{}", YesNo(true)), "Yes");
    assert_eq!(format!("{}", YesNo(false)), "No");
}
