#![allow(unused_imports)]

use std::{collections::BTreeSet, ops::Range};

use rand::{
    distributions::{Distribution, Uniform},
    Rng, SeedableRng,
};
use rand_chacha::ChaCha20Rng;

pub trait Gen {
    type Output;
    fn generate<R: Rng>(&self, rng: &mut R) -> Self::Output;
}

pub struct StrictAsc<B> {
    bound: B,
    len: usize,
}

pub struct Asc<B> {
    bound: B,
    len: usize,
}

impl Gen for Range<i32> {
    type Output = i32;
    fn generate<R: Rng>(&self, rng: &mut R) -> Self::Output {
        let between = Uniform::from(self.clone());
        between.sample(rng)
    }
}

impl Gen for StrictAsc<Range<i32>> {
    type Output = Vec<i32>;
    fn generate<R: Rng>(&self, rng: &mut R) -> Self::Output {
        let Self { bound: Range { start, end }, len } = self;

        // n = end - start, k = len
        let dense = (2 * len) as i32 > (end - start) / 2;
        let count = if dense { (end - start) as usize - len } else { *len };

        let mut seen = BTreeSet::new();
        while seen.len() < count {
            seen.insert((*start..*end).generate(rng));
        }

        if dense {
            (*start..*end).filter(|x| !seen.contains(x)).collect()
        } else {
            seen.into_iter().collect()
        }
    }
}

impl Gen for Asc<Range<i32>> {
    type Output = Vec<i32>;
    fn generate<R: Rng>(&self, rng: &mut R) -> Self::Output {
        let Self { bound: Range { start, end }, len } = self;
        let mut strict =
            StrictAsc { bound: *start..end + *len as i32, len: *len }
                .generate(rng);
        for i in 0..*len {
            strict[i] -= i as i32;
        }
        strict
    }
}

#[test]
fn uniformity() {
    use std::collections::BTreeMap;

    let mut rng = ChaCha20Rng::from_seed([0; 32]);
    let n = 10_usize.pow(6);

    let mut map = BTreeMap::new();
    for _ in 0..n {
        let tmp = StrictAsc { bound: 0..4, len: 3 }.generate(&mut rng);
        *map.entry(tmp).or_insert(0) += 1;
    }
    let k = 4;
    assert_eq!(map.len(), k);
    for &v in map.values() {
        assert!(v >= (n / k) * 99 / 100);
        assert!(v <= (n / k) * 101 / 100);
    }

    let mut map = BTreeMap::new();
    for _ in 0..n {
        let tmp = Asc { bound: 0..4, len: 3 }.generate(&mut rng);
        *map.entry(tmp).or_insert(0) += 1;
    }
    let k = 35;
    assert_eq!(map.len(), k);
    for &v in map.values() {
        assert!(v >= (n / k) * 97 / 100);
        assert!(v <= (n / k) * 103 / 100);
    }
}

#[test]
#[cfg(ignore)]
fn macros() {
    rand_gen! {
        rng = _; // Default
        n in 1_usize..10;
        a in [0..10_i32.pow(9); n];
        b in Asc { bound: 1..=5, len: 3 };
        c in StrictAsc { bound: 1..=5, len: 3 };
    }
    let mut rng = ChaCha20Rng::from_seed([0; 32]);
    rand_gen! {
        rng = &mut rng;
        n in 1_usize..10;
        a in [0..10_i32.pow(9); n];
        b in Asc { bound: 1..=5, len: 3 };
        c in StrictAsc { bound: 1..=5, len: 3 };
    }
}
