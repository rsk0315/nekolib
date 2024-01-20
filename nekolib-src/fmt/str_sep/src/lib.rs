use std::fmt;

pub struct SpaceSep<I>(pub I);
pub struct PerLine<I>(pub I);
pub struct StrSep<'a, I>(pub I, pub &'a str);

pub struct SpaceSepUsize1<I>(pub I);
pub struct PerLineUsize1<I>(pub I);
pub struct StrSepUsize1<'a, I>(pub I, pub &'a str);

macro_rules! impl_fmt {
    ( $( $fmt:ident )* ) => { $(
        #[allow(non_snake_case)]
        fn $fmt<I, T: fmt::$fmt>(iter: I, sep: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result
        where
            I: IntoIterator<Item = T>,
        {
            let mut iter = iter.into_iter();
            if let Some(first) = iter.by_ref().next() {
                first.fmt(f)?;
            }
            iter.map(|rest| { write!(f, "{}", sep)?; rest.fmt(f) }).collect()
        }

        impl<I, T: fmt::$fmt> fmt::$fmt for SpaceSep<I>
        where
            I: IntoIterator<Item = T> + Clone,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                $fmt(self.0.clone(), " ", f)
            }
        }
        impl<I, T: fmt::$fmt> fmt::$fmt for PerLine<I>
        where
            I: IntoIterator<Item = T> + Clone,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                $fmt(self.0.clone(), "\n", f)
            }
        }
        impl<I, T: fmt::$fmt> fmt::$fmt for StrSep<'_, I>
        where
            I: IntoIterator<Item = T> + Clone,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                $fmt(self.0.clone(), self.1, f)
            }
        }
    )* }
}

macro_rules! impl_fmt_usize1 {
    ( $( $fmt:ident )* ) => { $(
        impl<I, T> fmt::$fmt for SpaceSepUsize1<I>
        where
            I: IntoIterator<Item = T> + Clone,
            T: std::ops::Add<usize, Output = usize>,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                $fmt(self.0.clone().into_iter().map(|u| u + 1), " ", f)
            }
        }
        impl<I, T> fmt::$fmt for PerLineUsize1<I>
        where
            I: IntoIterator<Item = T> + Clone,
            T: std::ops::Add<usize, Output = usize>,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                $fmt(self.0.clone().into_iter().map(|u| u + 1), "\n", f)
            }
        }
        impl<I, T> fmt::$fmt for StrSepUsize1<'_, I>
        where
            I: IntoIterator<Item = T> + Clone,
            T: std::ops::Add<usize, Output = usize>,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                $fmt(self.0.clone().into_iter().map(|u| u + 1), self.1, f)
            }
        }
)* }
}

impl_fmt! { Binary Debug Display LowerExp LowerHex Octal Pointer UpperExp UpperHex }
impl_fmt_usize1! { Debug Display LowerHex Octal UpperHex }

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
fn iter() {
    assert_eq!(format!("{}", SpaceSep(0..5)), "0 1 2 3 4");
    assert_eq!(
        format!("{}", SpaceSep([0, 1, 2].iter().map(|x| x * 100))),
        "0 100 200"
    );
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

#[test]
fn usize1() {
    assert_eq!(format!("{}", SpaceSepUsize1([0, 3, 1, 4, 2])), "1 4 2 5 3");
    let a = vec![0, 3, 1, 4, 2];
    assert_eq!(format!("{}", SpaceSepUsize1(&a)), "1 4 2 5 3");
    assert_eq!(format!("{}", SpaceSepUsize1(a)), "1 4 2 5 3");
}
