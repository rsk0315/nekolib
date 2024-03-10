use rand::{
    distributions::{Bernoulli, Distribution},
    SeedableRng,
};
use rand_chacha::ChaCha20Rng;
use rs01dict_tree::Rs01DictTree;

fn rng() -> ChaCha20Rng {
    ChaCha20Rng::from_seed([
        0x55, 0xEF, 0xE0, 0x3C, 0x71, 0xDA, 0xFC, 0xAB, 0x5C, 0x1A, 0x9F, 0xEB,
        0xA4, 0x9E, 0x61, 0xE6, 0x1E, 0x7E, 0x29, 0x77, 0x38, 0x9A, 0xF5, 0x67,
        0xF5, 0xDD, 0x07, 0x06, 0xAE, 0xE4, 0x5A, 0xDC,
    ])
}

fn test_rank_internal(len: usize, p: f64) {
    let mut rng = rng();
    let dist = Bernoulli::new(p).unwrap();
    let a: Vec<_> = (0..len).map(|_| dist.sample(&mut rng)).collect();
    let naive: Vec<_> = a
        .iter()
        .map(|&x| x as usize)
        .scan(0, |acc, x| Some(std::mem::replace(acc, *acc + x)))
        .collect();
    let dict = Rs01DictTree::new(&a);
    for i in 0..len {
        assert_eq!(dict.rank1(i), naive[i], "i: {}", i);
        assert_eq!(dict.rank0(i), i - naive[i], "i: {}", i);
    }
    if p == 1.0 {
        eprintln!("---");
        eprintln!("a.len(): {}", a.len());
    }
}

fn test_select_internal(len: usize, p: f64) {
    eprintln!("{:?}", (len, p));
    let mut rng = rng();
    let dist = Bernoulli::new(p).unwrap();
    let a: Vec<_> = (0..len).map(|_| dist.sample(&mut rng)).collect();
    let naive: (Vec<_>, _) = (0..len).partition(|&i| !a[i]);
    let dict = Rs01DictTree::new(&a);

    for i in 0..naive.0.len() {
        assert_eq!(dict.select0(i), naive.0[i], "i: {}", i);
    }
    for i in 0..naive.1.len() {
        assert_eq!(dict.select1(i), naive.1[i], "i: {}", i);
    }
    if p == 1.0 {
        eprintln!("---");
        eprintln!("a.len(): {}", a.len());
    }
}

fn main() {
    for len in Some(0).into_iter().chain((0..=5).map(|e| 10_usize.pow(e))) {
        for &p in &[1.0, 0.999, 0.9, 0.5, 0.1, 1.0e-3, 0.0] {
            test_rank_internal(len, p);
        }
    }

    for len in Some(0).into_iter().chain((0..=5).map(|e| 10_usize.pow(e))) {
        for &p in &[1.0, 0.999, 0.9, 0.5, 0.1, 1.0e-3, 0.0] {
            test_select_internal(len, p);
        }
    }
}
