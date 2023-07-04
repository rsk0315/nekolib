pub struct StrSep<'a, D: ?Sized>(pub &'a D, pub &'a str);
pub struct SpaceSep<'a, D: ?Sized>(pub &'a D);
pub struct PerLine<'a, D: ?Sized>(pub &'a D);

use std::fmt;

macro_rules! impl_fmt {
    ( $( ($fmt:ident, $fn:ident), )* ) => { $(
        fn $fn<I>(it: I, sep: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result
        where
            I: IntoIterator,
            <I as IntoIterator>::Item: fmt::$fmt,
        {
            let mut it = it.into_iter();
            if let Some(x) = it.by_ref().next() {
                fmt::$fmt::fmt(&x, f)?;
                for x in it {
                    write!(f, "{}", sep)?;
                    fmt::$fmt::fmt(&x, f)?;
                }
            }
            Ok(())
        }
        impl<'a, D: 'a> fmt::$fmt for StrSep<'a, D>
        where
            D: ?Sized,
            &'a D: IntoIterator,
            <&'a D as IntoIterator>::Item: fmt::$fmt,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                $fn(self.0, self.1, f)
            }
        }
        impl<'a, D: 'a> fmt::$fmt for SpaceSep<'a, D>
        where
            D: ?Sized,
            &'a D: IntoIterator,
            <&'a D as IntoIterator>::Item: fmt::$fmt,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                $fn(self.0, " ", f)
            }
        }
        impl<'a, D: 'a> fmt::$fmt for PerLine<'a, D>
        where
            D: ?Sized,
            &'a D: IntoIterator,
            <&'a D as IntoIterator>::Item: fmt::$fmt,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                $fn(self.0, "\n", f)
            }
        }
    )* }
}

impl_fmt! {
    (Display, join_display),
    (Debug, join_debug),
    (Octal, join_octal),
    (LowerHex, join_lower_hex),
    (UpperHex, join_upper_hex),
    (Pointer, join_pointer),
    (Binary, join_binary),
    (LowerExp, join_lower_exp),
    (UpperExp, join_upper_exp),
}

#[test]
fn sanity_check() {
    let a = [0, 1, 2];
    assert_eq!(format!("{}", StrSep(&a[..0], "_")), "");
    assert_eq!(format!("{}", StrSep(&a[..1], "_")), "0");
    assert_eq!(format!("{}", StrSep(&a[..2], "_")), "0_1");
    assert_eq!(format!("{}", StrSep(&a, "_")), "0_1_2");

    assert_eq!(format!("{}", SpaceSep(&a[..0])), "");
    assert_eq!(format!("{}", SpaceSep(&a[..1])), "0");
    assert_eq!(format!("{}", SpaceSep(&a[..2])), "0 1");
    assert_eq!(format!("{}", SpaceSep(&a)), "0 1 2");

    assert_eq!(format!("{}", PerLine(&a[..0])), "");
    assert_eq!(format!("{}", PerLine(&a[..1])), "0");
    assert_eq!(format!("{}", PerLine(&a[..2])), "0\n1");
    assert_eq!(format!("{}", PerLine(&a)), "0\n1\n2");
}

#[test]
fn formatting() {
    let int = [3, 14, 1, 59];
    assert_eq!(format!("({})", SpaceSep(&int)), "(3 14 1 59)");
    assert_eq!(format!("{:2}", SpaceSep(&int)), " 3 14  1 59");
    assert_eq!(format!("{:<2}", SpaceSep(&int)), "3  14 1  59");
    assert_eq!(format!("{:o}", SpaceSep(&int)), "3 16 1 73");
    assert_eq!(format!("{:x}", SpaceSep(&int)), "3 e 1 3b");
    assert_eq!(format!("{:#04x}", SpaceSep(&int)), "0x03 0x0e 0x01 0x3b");
    assert_eq!(
        format!("{:08b}", SpaceSep(&int)),
        "00000011 00001110 00000001 00111011"
    );

    let ch = ['a', '\0', '\n', char::MAX];
    assert_eq!(format!("{}", SpaceSep(&ch)), "a \0 \n \u{10ffff}");
    assert_eq!(format!("{:?}", SpaceSep(&ch)), r"'a' '\0' '\n' '\u{10ffff}'");
    assert_eq!(format!("{:#?}", SpaceSep(&ch)), r"'a' '\0' '\n' '\u{10ffff}'");

    let nested = [[1, 2], [3, 4]];
    assert_eq!(
        format!("\n{:?}", PerLine(&nested)),
        r"
[1, 2]
[3, 4]"
    );
    assert_eq!(
        format!("\n{:#?}", PerLine(&nested)),
        r"
[
    1,
    2,
]
[
    3,
    4,
]"
    );

    let float = [0.1, 2.34, f64::INFINITY, f64::NAN];
    assert_eq!(format!("{:4}", SpaceSep(&float)), " 0.1 2.34  inf  NaN");
    assert_eq!(format!("{:0.2}", SpaceSep(&float)), "0.10 2.34 inf NaN");
}
