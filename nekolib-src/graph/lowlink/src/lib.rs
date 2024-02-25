use std::marker::PhantomData;

pub struct Lowlink<V, I, D> {
    low: Vec<usize>,
    ord: Vec<usize>,
    par_ord: Vec<usize>,
    index: I,
    delta: D,
    _phd: PhantomData<(fn(&V) -> I, fn(&V) -> D)>,
}

impl<V, I, D, J> Lowlink<V, I, D>
where
    V: Eq + Clone,
    I: Fn(&V) -> usize + Copy,
    D: Fn(&V) -> J + Copy,
    J: Iterator<Item = V>,
{
    pub fn new(
        vertices: impl Iterator<Item = V>,
        len: usize,
        index: I,
        delta: D,
    ) -> Self
    where
        D: Fn(&V) -> J + Copy,
        I: Fn(&V) -> usize + Copy,
        J: Iterator<Item = V>,
    {
        struct State {
            ord: Vec<usize>,
            low: Vec<usize>,
            par_ord: Vec<usize>,
            is_ancestor: Vec<bool>,
            index: usize,
        }

        fn dfs<V, I, D, J>(v: &V, index: I, delta: D, state: &mut State)
        where
            D: Fn(&V) -> J + Copy,
            I: Fn(&V) -> usize + Copy,
            J: Iterator<Item = V>,
        {
            let n = state.ord.len();
            let vi = index(v);
            state.ord[vi] = state.index;
            state.low[vi] = state.index;
            state.index += 1;
            state.is_ancestor[vi] = true;
            for nv in delta(v) {
                let nvi = index(&nv);
                if state.ord[nvi] == n {
                    dfs(&nv, index, delta, state);
                    state.par_ord[nvi] = vi;
                    state.low[vi] = state.low[vi].min(state.low[nvi]);
                } else if state.is_ancestor[nvi] {
                    state.low[vi] = state.low[vi].min(state.ord[nvi]);
                }
            }
            state.is_ancestor[vi] = false;
        }

        let mut state = State {
            ord: vec![len; len],
            low: vec![len; len],
            par_ord: vec![len; len],
            index: 0,
            is_ancestor: vec![false; len],
        };

        for v in vertices {
            if state.ord[index(&v)] == len {
                dfs(&v, index, delta, &mut state);
            }
        }

        let State { ord, low, par_ord, .. } = state;
        Self {
            ord,
            low,
            par_ord,
            index,
            delta,
            _phd: PhantomData,
        }
    }

    pub fn low(&self, v: &V) -> usize { self.low[(self.index)(v)] }
    pub fn ord(&self, v: &V) -> usize { self.ord[(self.index)(v)] }
    pub fn par_ord(&self, v: &V) -> usize { self.par_ord[(self.index)(v)] }

    // the difference of the number of connected components on the removal of the vertex
    pub fn cc_rm_v(&self, v: &V) -> isize {
        let len = self.ord.len();
        let vi = (self.index)(v);
        let count = if self.par_ord[vi] == len {
            // (!) multiple edges?
            (self.delta)(v)
                .map(|nv| (self.index)(&nv))
                .filter(|&nvi| self.par_ord[nvi] == vi)
                .count()
        } else {
            (self.delta)(v)
                .map(|nv| (self.index)(&nv))
                .filter(|&nvi| self.par_ord[nvi] == vi)
                .filter(|&nvi| self.ord[vi] <= self.low[nvi])
                .count()
                + 1
        };
        count as isize - 1
    }
}
