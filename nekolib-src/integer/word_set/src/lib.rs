pub trait WordSet: Sized {
    fn subset(self) -> impl Iterator<Item = Self>;
}

macro_rules! impl_uint {
    ( $($ty:ty)* ) => { $(
        impl WordSet for $ty {
            fn subset(self) -> impl Iterator<Item = Self> {
                let n = self;
                std::iter::successors(Some(0), move |&i| {
                    let d = n ^ i;
                    (d > 0).then(|| {
                        let d_neg = d.wrapping_neg();
                        (i & (d | d_neg)) + (d & d_neg)
                    })
                })
            }
        }
    )* };
}

impl_uint! { u8 u16 u32 u64 u128 usize }

#[test]
fn sanity_check() {
    assert!(0_usize.subset().eq([0]));
    eprintln!("{:?}", 0b1101_usize.subset().collect::<Vec<_>>());
    assert!(
        0b1101_usize.subset().eq([
            0b0000, 0b0001, 0b0100, 0b0101, 0b1000, 0b1001, 0b1100, 0b1101
        ])
    );
}
