use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use small_rank_select::{
    const_rank_table, const_select_table, rank, select, RankTable, SelectTable,
};

fn rand_seq<T, I, R>(iter: I, rng: &mut R) -> Vec<T>
where
    I: IntoIterator<Item = T>,
    R: Rng + ?Sized,
{
    let mut res: Vec<_> = iter.into_iter().collect();
    res.shuffle(rng);
    res
}

const RANK_TABLE: [[u8; 8]; 256] = const_rank_table::<8, 256>();
const SELECT_TABLE: [[u8; 8]; 256] = const_select_table::<8, 256>();

fn bench_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("small");

    let mut rng = ChaCha20Rng::from_seed([
        0x55, 0xEF, 0xE0, 0x3C, 0x71, 0xDA, 0xFC, 0xAB, 0x5C, 0x1A, 0x9F, 0xEB,
        0xA4, 0x9E, 0x61, 0xE6, 0x1E, 0x7E, 0x29, 0x77, 0x38, 0x9A, 0xF5, 0x67,
        0xF5, 0xDD, 0x07, 0x06, 0xAE, 0xE4, 0x5A, 0xDC,
    ]);

    let rank_table = RankTable::new();
    let select_table = SelectTable::new();

    let rank_query = {
        let mut q = vec![];
        for i in 0..=255 {
            for j in 0..8 {
                q.push((i, j));
            }
        }
        rand_seq(q.repeat(100), &mut rng)
    };
    let select_query = {
        let mut q = vec![];
        for i in 0_u8..=255 {
            for j in 0..i.count_ones() as u8 {
                q.push((i, j));
            }
        }
        rand_seq(q.repeat(200), &mut rng)
    };

    group
        .bench_function(BenchmarkId::new("lookup-const", "rank"), |b| {
            b.iter(|| {
                for &(w, i) in &rank_query {
                    black_box(RANK_TABLE[w as usize][i as usize]);
                }
            })
        })
        .bench_function(BenchmarkId::new("lookup-const", "select"), |b| {
            b.iter(|| {
                for &(w, i) in &select_query {
                    black_box(SELECT_TABLE[w as usize][i as usize]);
                }
            })
        })
        .bench_function(BenchmarkId::new("lookup-runtime", "rank"), |b| {
            b.iter(|| {
                for &(w, i) in &rank_query {
                    black_box(rank_table.rank(w as _, i as _));
                }
            })
        })
        .bench_function(BenchmarkId::new("lookup-runtime", "select"), |b| {
            b.iter(|| {
                for &(w, i) in &select_query {
                    black_box(select_table.select(w as _, i as _));
                }
            })
        })
        .bench_function(BenchmarkId::new("calculate", "rank"), |b| {
            b.iter(|| {
                for &(w, i) in &rank_query {
                    black_box(rank(w, i as _));
                }
            })
        })
        .bench_function(BenchmarkId::new("calculate", "select"), |b| {
            b.iter(|| {
                for &(w, i) in &select_query {
                    black_box(select(w, i as _));
                }
            })
        });

    // group
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
    // .bench_function(BenchmarkId::new("tree", "preprocess"), |b| {
    //     b.iter(|| black_box(Rs01DictTree::new(&a)))
    // })

    group.finish();
}

criterion_group!(benches, bench_small);
criterion_main!(benches);
