pub struct N1Rmq<T> {
    base: Vec<T>,
    large: SparseTable<T>,
    small: Vec<u8>,
    types: Vec<usize>,
    b: usize,
}

impl<T: Clone + Ord> From<Vec<T>> for N1Rmq<T> {
    fn from(base: Vec<T>) -> Self {
        let n = base.len();
        let lg_n = n.next_power_of_two().trailing_zeros();
        let b = 1.max(lg_n / 4) as usize;

        let mut large = vec![];
        let mut small = vec![0; (b * b) << (2 * b)];
        let mut types = vec![];
        let mut seen = vec![false; 1 << (2 * b)];
        for ch in base.chunks(b) {
            large.push(ch.iter().min().unwrap().clone());
            let ty = enc(ch);
            types.push(ty);
            if !seen[ty] {
                for l in 0..ch.len() {
                    let mut j = l;
                    for r in l..ch.len() {
                        if ch[j] > ch[r] {
                            j = r;
                        }
                        let i = (ty * b + l) * b + r;
                        small[i] = j as _;
                    }
                }
                seen[ty] = true;
            }
        }
        let large: SparseTable<_> = large.into();
        Self { base, large, small, types, b }
    }
}

impl<T: Clone + Ord> N1Rmq<T> {
    fn index(&self, bucket: usize, start: usize, end: usize) -> usize {
        let b = self.b;
        (self.types[bucket] * b + start) * b + end
    }

    pub fn min(&self, l: usize, r: usize) -> &T {
        assert!(l < r);
        let b = self.b;
        let lb = l / b;
        let rb = (r - 1) / b;
        if lb == rb {
            let j = self.small[self.index(lb, l % b, (r - 1) % b)] as usize;
            return &self.base[lb * b + j];
        }
        let mut res = if l % b == 0 {
            self.large.min(lb, lb + 1)
        } else {
            let j = self.small[self.index(lb, l % b, b - 1)] as usize;
            &self.base[lb * b + j]
        };
        res = res.min(if r % b == 0 {
            self.large.min(rb, rb + 1)
        } else {
            let j = self.small[self.index(rb, 0, (r - 1) % b)] as usize;
            &self.base[rb * b + j]
        });

        if lb + 1 < rb {
            res = res.min(self.large.min(lb + 1, rb));
        }
        res
    }
}

fn enc<T: Ord>(a: &[T]) -> usize {
    let mut stack = vec![];
    let mut res = 0;
    for ai in a {
        while let Some(&last) = stack.last() {
            if last > ai {
                stack.pop();
                res = res << 1 | 1;
            } else {
                break;
            }
        }
        stack.push(ai);
        res = res << 1 | 0;
    }
    ((res + 1) << stack.len()) - 1
}

struct SparseTable<T> {
    base: Vec<T>,
    table: Vec<Vec<usize>>,
}

impl<T: Ord> From<Vec<T>> for SparseTable<T> {
    fn from(base: Vec<T>) -> Self {
        let mut table = vec![];
        let n = base.len();
        table.push((0..n).collect::<Vec<_>>());
        for sh in 1.. {
            let last = table.last().unwrap();
            let len = 1 << sh;
            if len >= n {
                break;
            }
            let mut cur = vec![0; n - len + 1];
            for i in len..=n {
                let (il, ir) = (last[i - len], last[i - len + (1 << (sh - 1))]);
                cur[i - len] = if base[il] < base[ir] { il } else { ir };
            }
            table.push(cur);
        }
        Self { base, table }
    }
}

impl<T: Ord> SparseTable<T> {
    pub fn min(&self, i: usize, j: usize) -> &T {
        let len = j - i;
        if len <= 1 {
            return &self.base[i];
        }
        let sh = len.next_power_of_two().trailing_zeros() as usize - 1;
        let [il, ir] = [self.table[sh][i], self.table[sh][j - (1 << sh)]];
        (&self.base[il]).min(&self.base[ir])
    }
}

#[test]
fn test() {
    let n = 20000;
    let it = std::iter::successors(Some(3_usize), |x| Some(3 * x % 46337));
    let a: Vec<_> = it.take(n).collect();
    let rmq: N1Rmq<_> = a.clone().into();

    for l in 0..n {
        let mut min = a[l];
        for r in l..n - 1 {
            min = min.min(a[r]);
            assert_eq!(rmq.min(l, r + 1), &min);
        }
    }
}
