pub struct Scc<V, I> {
    comp_id: Vec<usize>,
    comp: Vec<Vec<V>>,
    index: I,
}

impl<V, I> Scc<V, I>
where
    V: Eq + Clone,
    I: Fn(&V) -> usize + Copy,
{
    pub fn new<D, J>(
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
            scc: Vec<Vec<usize>>,
            num: Vec<usize>,
            low: Vec<usize>,
            stack: Vec<usize>,
            is_ancestor: Vec<bool>,
            index: usize,
        }

        fn dfs<V, D, I, J>(v: &V, index: I, delta: D, state: &mut State)
        where
            D: Fn(&V) -> J + Copy,
            I: Fn(&V) -> usize + Copy,
            J: Iterator<Item = V>,
        {
            state.index += 1;
            let vi = index(v);
            state.low[vi] = state.index;
            state.num[vi] = state.index;
            state.stack.push(vi);
            state.is_ancestor[vi] = true;
            for nv in delta(v) {
                let nvi = index(&nv);
                if state.num[nvi] == 0 {
                    dfs(&nv, index, delta, state);
                    state.low[vi] = state.low[vi].min(state.low[nvi]);
                } else if state.is_ancestor[nvi] {
                    state.low[vi] = state.low[vi].min(state.num[nvi]);
                }
            }
            if state.low[vi] == state.num[vi] {
                let mut tmp = vec![];
                loop {
                    let nvi = state.stack.pop().unwrap();
                    state.is_ancestor[nvi] = false;
                    tmp.push(nvi);
                    if vi == nvi {
                        break;
                    }
                }
                state.scc.push(tmp);
            }
        }

        let mut state = State {
            scc: vec![],
            num: vec![0; len],
            low: vec![0; len],
            stack: vec![],
            is_ancestor: vec![false; len],
            index: 0,
        };

        let mut vs = vec![];
        for v in vertices {
            if state.num[index(&v)] == 0 {
                dfs(&v, index, delta, &mut state);
            }
            vs.push(v);
        }

        let mut comp_id = vec![0; len];
        for i in 0..state.scc.len() {
            for &c in &state.scc[i] {
                comp_id[c] = state.scc.len() - i - 1;
            }
        }

        let mut comp: Vec<_> = (0..state.scc.len()).map(|_| vec![]).collect();
        for v in vs {
            comp[comp_id[index(&v)]].push(v);
        }

        Self { comp_id, comp, index }
    }

    pub fn comp_id(&self, v: &V) -> usize { self.comp_id[(self.index)(v)] }
    pub fn comp(&self, i: usize) -> &[V] { &self.comp[i] }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn sanity_check() {
        // 4 -> 0 <> 1 -> 5
        //      |    ^
        //      v    |
        //      3 -> 2
        //      ^
        //      |
        //      6
        let g = vec![
            vec![1, 3], // 0
            vec![0, 5], // 1
            vec![0],    // 2
            vec![2],    // 3
            vec![0],    // 4
            vec![],     // 5
            vec![3],    // 6
        ];
        let len = g.len();
        let index = |&v: &usize| v;
        let delta = |&v: &usize| g[v].iter().copied();
        let scc = Scc::new(0..len, len, index, delta);

        assert!(scc.comp_id(&0) == scc.comp_id(&1));
        assert!(scc.comp_id(&0) == scc.comp_id(&2));
        assert!(scc.comp_id(&0) == scc.comp_id(&3));
        assert!(scc.comp_id(&0) > scc.comp_id(&4));
        assert!(scc.comp_id(&0) > scc.comp_id(&6));
        assert!(scc.comp_id(&0) < scc.comp_id(&5));
    }
}
