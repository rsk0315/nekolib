pub trait CycleMuLambda: Eq {
    fn cycle_mu_lambda<F: Fn(&Self) -> Self>(self, f: F) -> (usize, usize);
}

impl<T: Eq> CycleMuLambda for T {
    fn cycle_mu_lambda<F: Fn(&T) -> T>(self, f: F) -> (usize, usize) {
        let mut tor = f(&self);
        let mut har = f(&tor);

        while tor != har {
            tor = f(&tor);
            har = f(&f(&har));
        }

        let mut tor = self;
        let mut mu = 0;
        while tor != har {
            tor = f(&tor);
            har = f(&har);
            mu += 1;
        }

        let mut lambda = 1;
        har = f(&tor);
        while tor != har {
            har = f(&har);
            lambda += 1;
        }

        (mu, lambda)
    }
}

#[test]
fn sanity_check() {
    let x0 = 879_u32;
    let f = |&x: &u32| x % 104 * 10;
    // 879/104 = 8.451(923076...)
    assert_eq!(x0.cycle_mu_lambda(f), (4, 6));

    assert_eq!('.'.cycle_mu_lambda(|&x| x), (0, 1));
}
