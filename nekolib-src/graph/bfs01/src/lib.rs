use std::collections::VecDeque;

pub struct Cert<V>(Vec<Option<V>>);
pub struct NoCert;

pub struct Bfs01Sssp<V, I, C> {
    cost: Vec<usize>,
    prev: C,
    index: I,
    src: V,
}

impl<V, I> Bfs01Sssp<V, I, Cert<V>>
where
    V: Eq + Clone,
    I: Fn(&V) -> usize,
{
    pub fn new_cert<D, J>(src: V, len: usize, index: I, delta: D) -> Self
    where
        D: Fn(&V) -> J,
        J: Iterator<Item = (V, usize)>,
    {
        let mut cost = vec![len; len];
        let mut prev = vec![None; len];
        let mut deque = VecDeque::new();
        cost[index(&src)] = 0;
        deque.push_front((0, src.clone()));
        while let Some((w, v)) = deque.pop_front() {
            for (nv, dw) in delta(&v) {
                let nw = w + dw;
                let ni = index(&nv);
                if cost[ni] > nw {
                    cost[ni] = nw;
                    prev[ni] = Some(v.clone());
                    if dw == 0 {
                        deque.push_front((nw, nv));
                    } else {
                        deque.push_back((nw, nv));
                    }
                }
            }
        }

        Self { src, cost, prev: Cert(prev), index }
    }
    pub fn path(&self, dst: &V) -> Option<std::vec::IntoIter<V>> {
        let mut i = (self.index)(dst);
        if self.prev.0[i].is_none() {
            return (&self.src == dst).then(|| vec![dst.clone()].into_iter());
        }

        let mut res = vec![dst.clone()];
        while let Some(v) = &self.prev.0[i] {
            i = (self.index)(v);
            res.push(v.clone());
        }
        res.reverse();
        Some(res.into_iter())
    }
}
impl<V, I> Bfs01Sssp<V, I, NoCert>
where
    V: Eq + Clone,
    I: Fn(&V) -> usize,
{
    pub fn new<D, J>(src: V, len: usize, index: I, delta: D) -> Self
    where
        D: Fn(&V) -> J,
        J: Iterator<Item = (V, usize)>,
    {
        let mut cost = vec![len; len];
        let mut deque = VecDeque::new();
        cost[index(&src)] = 0;
        deque.push_front((0, src.clone()));
        while let Some((w, v)) = deque.pop_front() {
            for (nv, dw) in delta(&v) {
                let nw = w + dw;
                let ni = index(&nv);
                if cost[ni] > nw {
                    cost[ni] = nw;
                    if dw == 0 {
                        deque.push_front((nw, nv));
                    } else {
                        deque.push_back((nw, nv));
                    }
                }
            }
        }

        Self { src, cost, prev: NoCert, index }
    }
}
impl<V, I, C> Bfs01Sssp<V, I, C>
where
    V: Eq + Clone,
    I: Fn(&V) -> usize,
{
    pub fn cost(&self, dst: &V) -> Option<usize> {
        let tmp = self.cost[(self.index)(dst)].clone();
        (tmp < self.cost.len()).then_some(tmp)
    }
}
