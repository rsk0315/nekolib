use std::ops::{Range, RangeBounds};

use usize_bounds::UsizeBounds;

pub struct SqrtBucket<T, S, Ff, Fr, Fq> {
    buf: Vec<T>,
    summary: Vec<S>,
    force: Ff,
    reduce: Fr,
    query: Fq,
    bucket_size: usize,
}

pub enum BucketBorrow<'a, T, S> {
    Slice(&'a mut [T]),
    Summary(&'a mut S),
}

impl<T, S, Ff, Fr, Fq> SqrtBucket<T, S, Ff, Fr, Fq>
where
    Fr: FnMut(&[T]) -> S,
    Ff: FnMut(&S, &mut [T]),
{
    pub fn new(buf: Vec<T>, force: Ff, reduce: Fr, query: Fq) -> Self {
        Self::new_with_bucket_size(buf, force, reduce, query, 384)
    }

    pub fn new_with_bucket_size(
        buf: Vec<T>,
        force: Ff,
        mut reduce: Fr,
        query: Fq,
        bucket_size: usize,
    ) -> Self {
        let summary: Vec<S> =
            buf.chunks(bucket_size).map(&mut reduce).collect();
        Self { buf, summary, force, reduce, query, bucket_size }
    }

    pub fn query<Q, B, R>(&mut self, range: B, args: Q) -> R
    where
        B: RangeBounds<usize>,
        Fq: for<'a> FnMut(&mut [BucketBorrow<'a, T, S>], Q) -> R,
    {
        let n = self.buf.len();
        let b = self.bucket_size;
        let Range { start, end } = range.to_range(n);

        let mut borrowed = vec![];
        let mut affected = vec![];
        for ((l, chunk_i), summary_i) in
            (0..n).step_by(b).zip(self.buf.chunks_mut(b)).zip(&mut self.summary)
        {
            let i = l / b;
            let r = n.min(l + b);
            if r <= start {
                continue;
            }
            if end <= l {
                break;
            }
            if start <= l && r <= end {
                borrowed.push(BucketBorrow::Summary(summary_i));
            } else {
                (self.force)(summary_i, chunk_i);
                affected.push((i, (l, r)));
                let jl = l.max(start) - l;
                let jr = r.min(end) - l;
                borrowed.push(BucketBorrow::Slice(&mut chunk_i[jl..jr]));
            }
        }
        let res = (self.query)(&mut borrowed, args);
        for (i, (l, r)) in affected {
            self.summary[i] = (self.reduce)(&mut self.buf[l..r]);
        }
        res
    }
}

// TODO: doc (cf. <https://atcoder.jp/contests/abc322/submissions/53952026>)
