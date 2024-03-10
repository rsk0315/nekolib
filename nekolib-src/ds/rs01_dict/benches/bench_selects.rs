#![allow(long_running_const_eval)]

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
use rs01dict_runtime::Rs01DictRuntime;
use rs01dict_tree::Rs01DictTree;

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
    // let len = 1 << 20;
    let len = 10_usize.pow(7);
    let p = 1.0e-3;
    // let p = 5.0e-1;
    let dist = Bernoulli::new(p).unwrap();
    let a: Vec<_> = (0..len).map(|_| dist.sample(&mut rng)).collect();

    // let rs = Rs01Dict::new(&a);
    let rs_nlc = Rs01DictNlC::new(&a);
    let rs_nll = Rs01DictNLl::new(&a);
    let rs_rt = Rs01DictRuntime::new(&a);
    let rs_t = Rs01DictTree::new(&a);

    let expected_select0 = || (0..a.len()).filter(|&i| !a[i]);
    let count0 = expected_select0().count();
    eprintln!("count0: {count0}");

    let expected_select1 = || (0..a.len()).filter(|&i| a[i]);
    let count1 = expected_select1().count();
    eprintln!("count1: {count1}");

    // assert!((0..count0).map(|i| rs.select0(i)).eq(expected_select0()));
    // assert!((0..count0).map(|i| rs_nlc.select0(i)).eq(expected_select0()));
    // assert!((0..count0).map(|i| rs_nll.select0(i)).eq(expected_select0()));
    // assert!((0..count0).map(|i| rs_rt.select0(i)).eq(expected_select0()));
    // assert!((0..count0).map(|i| rs_t.select0(i)).eq(expected_select0()));

    // assert!((0..count1).map(|i| rs.select1(i)).eq(expected_select1()));
    // assert!((0..count1).map(|i| rs_nlc.select1(i)).eq(expected_select1()));
    // assert!((0..count1).map(|i| rs_nll.select1(i)).eq(expected_select1()));
    // assert!((0..count1).map(|i| rs_rt.select1(i)).eq(expected_select1()));
    // assert!((0..count1).map(|i| rs_t.select1(i)).eq(expected_select1()));

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

    // assert!((0..a.len()).map(|i| rs.rank0(i)).eq(expected_rank0()));
    // assert!((0..a.len()).map(|i| rs_nlc.rank0(i)).eq(expected_rank0()));
    // assert!((0..a.len()).map(|i| rs_nll.rank0(i)).eq(expected_rank0()));
    // assert!(
    //     (0..a.len() - 1)
    //         .map(|i| rs_rt.rank0(i + 1))
    //         .eq(expected_rank0().take(a.len() - 1))
    // );
    // assert!(
    //     (0..a.len() - 1)
    //         .map(|i| rs_t.rank0(i + 1))
    //         .eq(expected_rank0().take(a.len() - 1))
    // );

    // assert!((0..a.len()).map(|i| rs.rank1(i)).eq(expected_rank1()));
    // assert!((0..a.len()).map(|i| rs_nlc.rank1(i)).eq(expected_rank1()));
    // assert!((0..a.len()).map(|i| rs_nll.rank1(i)).eq(expected_rank1()));
    // assert!(
    //     (0..a.len() - 1)
    //         .map(|i| rs_rt.rank1(i + 1))
    //         .eq(expected_rank1().take(a.len() - 1))
    // );
    // assert!(
    //     (0..a.len() - 1)
    //         .map(|i| rs_t.rank1(i + 1))
    //         .eq(expected_rank1().take(a.len() - 1))
    // );

    let rank_query = rand_seq(0..a.len(), &mut rng);
    let select0_query = rand_seq(0..count0, &mut rng);
    let select1_query = rand_seq(0..count1, &mut rng);

    group
        // .bench_function(BenchmarkId::new("succinct", "preprocess"), |b| {
        //     b.iter(|| black_box(Rs01Dict::new(&a)))
        // })
        // .bench_function(BenchmarkId::new("naive", "preprocess"), |b| {
        //     b.iter(|| black_box(Rs01DictNlC::new(&a)))
        // })
        // .bench_function(BenchmarkId::new("compact", "preprocess"), |b| {
        //     b.iter(|| black_box(Rs01DictNLl::new(&a)))
        // })
        // .bench_function(BenchmarkId::new("runtime", "preprocess"), |b| {
        //     b.iter(|| black_box(Rs01DictRuntime::new(&a)))
        // })
        .bench_function(BenchmarkId::new("tree", "preprocess"), |b| {
            b.iter(|| black_box(Rs01DictTree::new(&a)))
        });

    macro_rules! bench_fn {
        ($g:ident, $name:ident, $fn:ident, $fst:literal, $snd:literal, $iter:expr) => {
            $g.bench_function(BenchmarkId::new($fst, $snd), |b| {
                b.iter(|| {
                    for i in $iter {
                        black_box($name.$fn(i));
                    }
                })
            });
        };
    }

    // bench_fn! { group, rs, rank0, "succinct", "rank0-seq", 0..a.len() }
    // bench_fn! { group, rs, rank1, "succinct", "rank1-seq", 0..a.len() }
    // bench_fn! { group, rs, rank0, "succinct", "rank0-rand", rank_query.iter().copied() }
    // bench_fn! { group, rs, rank1, "succinct", "rank1-rand", rank_query.iter().copied() }
    // bench_fn! { group, rs, select0, "succinct", "select0-seq", 0..count0 }
    // bench_fn! { group, rs, select1, "succinct", "select1-seq", 0..count1 }
    // bench_fn! { group, rs, select0, "succinct", "select0-rand", select0_query.iter().copied() }
    // bench_fn! { group, rs, select1, "succinct", "select1-rand", select1_query.iter().copied() }

    // bench_fn! { group, rs_nll, rank0, "compact", "rank0-seq", 0..a.len() }
    // bench_fn! { group, rs_nll, rank1, "compact", "rank1-seq", 0..a.len() }
    bench_fn! { group, rs_nll, rank0, "compact", "rank0-rand", rank_query.iter().copied() }
    bench_fn! { group, rs_nll, rank1, "compact", "rank1-rand", rank_query.iter().copied() }
    // bench_fn! { group, rs_nll, select0, "compact", "select0-seq", 0..count0 }
    // bench_fn! { group, rs_nll, select1, "compact", "select1-seq", 0..count1 }
    bench_fn! { group, rs_nll, select0, "compact", "select0-rand", select0_query.iter().copied() }
    bench_fn! { group, rs_nll, select1, "compact", "select1-rand", select1_query.iter().copied() }

    // bench_fn! { group, rs_nlc, rank0, "naive", "rank0-seq", 0..a.len() }
    // bench_fn! { group, rs_nlc, rank1, "naive", "rank1-seq", 0..a.len() }
    bench_fn! { group, rs_nlc, rank0, "naive", "rank0-rand", rank_query.iter().copied() }
    bench_fn! { group, rs_nlc, rank1, "naive", "rank1-rand", rank_query.iter().copied() }
    // bench_fn! { group, rs_nlc, select0, "naive", "select0-seq", 0..count0 }
    // bench_fn! { group, rs_nlc, select1, "naive", "select1-seq", 0..count1 }
    bench_fn! { group, rs_nlc, select0, "naive", "select0-rand", select0_query.iter().copied() }
    bench_fn! { group, rs_nlc, select1, "naive", "select1-rand", select1_query.iter().copied() }

    // bench_fn! { group, rs_rt, rank0, "runtime", "rank0-seq", 0..a.len() }
    // bench_fn! { group, rs_rt, rank1, "runtime", "rank1-seq", 0..a.len() }
    // bench_fn! { group, rs_rt, rank0, "runtime", "rank0-rand", rank_query.iter().copied() }
    // bench_fn! { group, rs_rt, rank1, "runtime", "rank1-rand", rank_query.iter().copied() }
    // bench_fn! { group, rs_rt, select0, "runtime", "select0-seq", 0..count0 }
    // bench_fn! { group, rs_rt, select1, "runtime", "select1-seq", 0..count1 }
    // bench_fn! { group, rs_rt, select0, "runtime", "select0-rand", select0_query.iter().copied() }
    // bench_fn! { group, rs_rt, select1, "runtime", "select1-rand", select1_query.iter().copied() }

    // bench_fn! { group, rs_t, rank0, "tree", "rank0-seq", 0..a.len() }
    // bench_fn! { group, rs_t, rank1, "tree", "rank1-seq", 0..a.len() }
    bench_fn! { group, rs_t, rank0, "tree", "rank0-rand", rank_query.iter().copied() }
    bench_fn! { group, rs_t, rank1, "tree", "rank1-rand", rank_query.iter().copied() }
    // bench_fn! { group, rs_t, select0, "tree", "select0-seq", 0..count0 }
    // bench_fn! { group, rs_t, select1, "tree", "select1-seq", 0..count1 }
    bench_fn! { group, rs_t, select0, "tree", "select0-rand", select0_query.iter().copied() }
    bench_fn! { group, rs_t, select1, "tree", "select1-rand", select1_query.iter().copied() }

    group.finish();
}

criterion_group!(benches, bench_selects);
criterion_main!(benches);
