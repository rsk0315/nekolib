use monoid::def_monoid_generics;

def_monoid_generics! {
    pub OpConcat[T, B] where [
        T: Clone,
        B: Clone + IntoIterator<Item = T> + FromIterator<T>,
    ] = (
        B,
        |x: &B, y: &B| x.clone().into_iter().chain(y.clone()).collect(),
        || None.into_iter().collect(),
    ),
}

#[cfg(test)]
mod tests {
    use monoid::{BinaryOp, Identity};

    use crate::*;

    #[test]
    fn sanity_check() {
        let concat = OpConcat::<i32, Vec<_>>::new();
        assert_eq!(concat.op(&vec![], &vec![]), vec![]);
        assert_eq!(concat.op(&vec![], &vec![3, 4]), vec![3, 4]);
        assert_eq!(concat.op(&vec![1, 2], &vec![]), vec![1, 2]);
        assert_eq!(concat.op(&vec![1, 2], &vec![3, 4]), vec![1, 2, 3, 4]);
        assert_eq!(concat.id(), vec![]);
    }
}
