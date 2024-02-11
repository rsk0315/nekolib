use std::{collections::BinaryHeap, ops::Add};

pub struct DijkstraSssp<V, W, I> {
    cost: Vec<Option<W>>,
    prev: Vec<Option<V>>,
    index: I,
    src: V,
}

#[derive(Eq, PartialEq)]
struct RevFst<F, S>(F, S);

impl<F: Ord, S: Eq> Ord for RevFst<F, S> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

impl<F: Ord, S: Eq> PartialOrd for RevFst<F, S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<V, W, I> DijkstraSssp<V, W, I>
where
    V: Eq + Clone,
    W: Add<Output = W> + Ord + Clone,
    I: Fn(&V) -> usize,
{
    pub fn new<D, J>(src: V, len: usize, zero: W, index: I, delta: D) -> Self
    where
        D: Fn(&V) -> J,
        J: Iterator<Item = (V, W)>,
    {
        let mut cost = vec![None; len];
        let mut prev = vec![None; len];
        let mut heap = BinaryHeap::new();
        cost[index(&src)] = Some(zero.clone());
        heap.push(RevFst(zero, src.clone()));
        while let Some(RevFst(w, v)) = heap.pop() {
            if let Some(cur_w) = &cost[index(&v)] {
                if cur_w > &w {
                    continue;
                }
            }
            for (nv, dw) in delta(&v) {
                let nw = w.clone() + dw;
                let ni = index(&nv);
                match &cost[ni] {
                    Some(cur_w) if cur_w <= &nw => {}
                    _ => {
                        cost[ni] = Some(nw.clone());
                        prev[ni] = Some(v.clone());
                        heap.push(RevFst(nw, nv));
                    }
                }
            }
        }

        Self { src, cost, prev, index }
    }
    pub fn cost(&self, dst: &V) -> Option<W> {
        self.cost[(self.index)(dst)].clone()
    }
    pub fn path(&self, dst: &V) -> Option<std::vec::IntoIter<V>> {
        let mut i = (self.index)(dst);
        if self.prev[i].is_none() {
            return (&self.src == dst).then(|| vec![dst.clone()].into_iter());
        }

        let mut res = vec![dst.clone()];
        while let Some(v) = &self.prev[i] {
            i = (self.index)(v);
            res.push(v.clone());
        }
        res.reverse();
        Some(res.into_iter())
    }
}
