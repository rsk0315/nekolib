use bit_vector::{Rs01DictNLl, Rs01DictNlC};
use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use rand::{
    distributions::{Bernoulli, Distribution},
    seq::SliceRandom,
    Rng, SeedableRng,
};
use rand_chacha::ChaCha20Rng;
use rs01_dict::Rs01Dict;

// % bc <<< "obase=16; ibase=2; $(gshuf -re {0,1}{0,1}{0,1}{0,1} -n$((1024*16)))" \
//       | tr -d \\n | fold -w4 | paste -sd _ - | fold -w20 | sed 's/^/0x_/; s/_$/,/'

fn rand_seq<T, I, R>(iter: I, rng: &mut R) -> Vec<T>
where
    I: IntoIterator<Item = T>,
    R: Rng + ?Sized,
{
    let mut res: Vec<_> = iter.into_iter().collect();
    res.shuffle(rng);
    res
}

fn bench_selects(c: &mut Criterion) {
    let mut group = c.benchmark_group("rs01dict");

    // bc <<< "obase=16; ibase=2; $(shuf -re {0,1}{0,1}{0,1}{0,1}{0,1}{0,1}{0,1}{0,1} -n 32)" \
    //     | sed '/../!s/^/0/; s/^/0x/' | paste -sd , -
    let mut rng = ChaCha20Rng::from_seed([
        0x55, 0xEF, 0xE0, 0x3C, 0x71, 0xDA, 0xFC, 0xAB, 0x5C, 0x1A, 0x9F, 0xEB,
        0xA4, 0x9E, 0x61, 0xE6, 0x1E, 0x7E, 0x29, 0x77, 0x38, 0x9A, 0xF5, 0x67,
        0xF5, 0xDD, 0x07, 0x06, 0xAE, 0xE4, 0x5A, 0xDC,
    ]);
    let len = 1 << 20;
    let dist = Bernoulli::new(1.0e-3).unwrap();
    let a: Vec<_> = (0..len).map(|_| dist.sample(&mut rng)).collect();

    let rs = Rs01Dict::new(&a);
    let rs_nlc = Rs01DictNlC::new(&a);
    let rs_nll = Rs01DictNLl::new(&a);

    let expected_select0 = || (0..a.len()).filter(|&i| !a[i]);
    let count0 = expected_select0().count();
    eprintln!("count0: {count0}");

    let expected_select1 = || (0..a.len()).filter(|&i| a[i]);
    let count1 = expected_select1().count();
    eprintln!("count1: {count1}");

    assert!((0..count0).map(|i| rs.select0(i)).eq(expected_select0()));
    assert!((0..count0).map(|i| rs_nlc.select0(i)).eq(expected_select0()));
    assert!((0..count0).map(|i| rs_nll.select0(i)).eq(expected_select0()));

    assert!((0..count1).map(|i| rs.select1(i)).eq(expected_select1()));
    assert!((0..count1).map(|i| rs_nlc.select1(i)).eq(expected_select1()));
    assert!((0..count1).map(|i| rs_nll.select1(i)).eq(expected_select1()));

    assert!((0..count0).map(|i| rs.select0(i)).eq(expected_select0()));
    assert!((0..count0).map(|i| rs_nlc.select0(i)).eq(expected_select0()));
    assert!((0..count0).map(|i| rs_nll.select0(i)).eq(expected_select0()));

    assert!((0..count1).map(|i| rs.select1(i)).eq(expected_select1()));
    assert!((0..count1).map(|i| rs_nlc.select1(i)).eq(expected_select1()));
    assert!((0..count1).map(|i| rs_nll.select1(i)).eq(expected_select1()));

    let expected_rank0 = || {
        (0..a.len()).map(|i| !a[i] as usize).scan(0, |acc, x| {
            *acc += x;
            Some(*acc)
        })
    };
    let expected_rank1 = || {
        (0..a.len()).map(|i| a[i] as usize).scan(0, |acc, x| {
            *acc += x;
            Some(*acc)
        })
    };

    assert!((0..a.len()).map(|i| rs.rank0(i)).eq(expected_rank0()));
    assert!((0..a.len()).map(|i| rs_nlc.rank0(i)).eq(expected_rank0()));
    assert!((0..a.len()).map(|i| rs_nll.rank0(i)).eq(expected_rank0()));

    assert!((0..a.len()).map(|i| rs.rank1(i)).eq(expected_rank1()));
    assert!((0..a.len()).map(|i| rs_nlc.rank1(i)).eq(expected_rank1()));
    assert!((0..a.len()).map(|i| rs_nll.rank1(i)).eq(expected_rank1()));

    let rank_query = rand_seq(0..a.len(), &mut rng);
    let select0_query = rand_seq(0..count0, &mut rng);
    let select1_query = rand_seq(0..count1, &mut rng);

    let rep = 1;

    group
        .bench_function(BenchmarkId::new("succinct", "preprocess"), |b| {
            b.iter(|| black_box(Rs01Dict::new(&a)))
        })
        .bench_function(BenchmarkId::new("naive", "preprocess"), |b| {
            b.iter(|| black_box(Rs01DictNlC::new(&a)))
        })
        .bench_function(BenchmarkId::new("compact", "preprocess"), |b| {
            b.iter(|| black_box(Rs01DictNLl::new(&a)))
        })
        .bench_function(BenchmarkId::new("succinct", "rank-seq"), |b| {
            b.iter(|| {
                for _ in 0..rep {
                    for i in 0..a.len() {
                        black_box(rs.rank0(i));
                    }
                    for i in 0..a.len() {
                        black_box(rs.rank1(i));
                    }
                }
            })
        })
        .bench_function(BenchmarkId::new("naive", "rank-seq"), |b| {
            b.iter(|| {
                for _ in 0..rep {
                    for i in 0..a.len() {
                        black_box(rs_nlc.rank0(i));
                    }
                    for i in 0..a.len() {
                        black_box(rs_nlc.rank1(i));
                    }
                }
            })
        })
        .bench_function(BenchmarkId::new("compact", "rank-seq"), |b| {
            b.iter(|| {
                for _ in 0..rep {
                    for i in 0..a.len() {
                        black_box(rs_nll.rank0(i));
                    }
                    for i in 0..a.len() {
                        black_box(rs_nll.rank1(i));
                    }
                }
            })
        })
        .bench_function(BenchmarkId::new("succinct", "select-seq"), |b| {
            b.iter(|| {
                for _ in 0..rep {
                    for i in 0..count0 {
                        black_box(rs.select0(i));
                    }
                    for i in 0..count1 {
                        black_box(rs.select1(i));
                    }
                }
            })
        })
        .bench_function(BenchmarkId::new("naive", "select-seq"), |b| {
            b.iter(|| {
                for _ in 0..rep {
                    for i in 0..count0 {
                        black_box(rs_nlc.select0(i));
                    }
                    for i in 0..count1 {
                        black_box(rs_nlc.select1(i));
                    }
                }
            })
        })
        .bench_function(BenchmarkId::new("compact", "select-seq"), |b| {
            b.iter(|| {
                for _ in 0..rep {
                    for i in 0..count0 {
                        black_box(rs_nll.select0(i));
                    }
                    for i in 0..count1 {
                        black_box(rs_nll.select1(i));
                    }
                }
            })
        })
        .bench_function(BenchmarkId::new("succinct", "rank-rand"), |b| {
            b.iter(|| {
                for _ in 0..rep {
                    for &i in &rank_query {
                        black_box(rs.rank0(i));
                    }
                    for &i in &rank_query {
                        black_box(rs.rank1(i));
                    }
                }
            })
        })
        .bench_function(BenchmarkId::new("naive", "rank-rand"), |b| {
            b.iter(|| {
                for _ in 0..rep {
                    for &i in &rank_query {
                        black_box(rs_nlc.rank0(i));
                    }
                    for &i in &rank_query {
                        black_box(rs_nlc.rank1(i));
                    }
                }
            })
        })
        .bench_function(BenchmarkId::new("compact", "rank-rand"), |b| {
            b.iter(|| {
                for _ in 0..rep {
                    for &i in &rank_query {
                        black_box(rs_nll.rank0(i));
                    }
                    for &i in &rank_query {
                        black_box(rs_nll.rank1(i));
                    }
                }
            })
        })
        .bench_function(BenchmarkId::new("succinct", "select-rand"), |b| {
            b.iter(|| {
                for _ in 0..rep {
                    for &i in &select0_query {
                        black_box(rs.select0(i));
                    }
                    for &i in &select1_query {
                        black_box(rs.select1(i));
                    }
                }
            })
        })
        .bench_function(BenchmarkId::new("naive", "select-rand"), |b| {
            b.iter(|| {
                for _ in 0..rep {
                    for &i in &select0_query {
                        black_box(rs_nlc.select0(i));
                    }
                    for &i in &select1_query {
                        black_box(rs_nlc.select1(i));
                    }
                }
            })
        })
        .bench_function(BenchmarkId::new("compact", "select-rand"), |b| {
            b.iter(|| {
                for _ in 0..rep {
                    for &i in &select0_query {
                        black_box(rs_nll.select0(i));
                    }
                    for &i in &select1_query {
                        black_box(rs_nll.select1(i));
                    }
                }
            })
        });
}

criterion_group!(benches, bench_selects);
criterion_main!(benches);
