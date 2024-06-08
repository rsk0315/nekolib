use std::collections::BTreeSet;

// order-preserving minimal perfect hashing
pub trait Opmph<B> {
    fn opmph(&self) -> B;
}

impl<I, T, B> Opmph<B> for I
where
    for<'a> &'a I: IntoIterator<Item = &'a T>,
    T: Clone + Ord,
    B: FromIterator<(T, usize)>,
{
    fn opmph(&self) -> B {
        let seen: BTreeSet<_> = self.into_iter().cloned().collect();
        seen.into_iter().zip(0..).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use maplit::btreemap;

    use super::*;

    #[test]
    fn sanity_check() {
        let a = vec![3, 5, 1, 2, 5];
        let enc: BTreeMap<_, _> = a.opmph();

        assert_eq!(enc, btreemap! { 1 => 0, 2 => 1, 3 => 2, 5 => 3 });
    }
}
