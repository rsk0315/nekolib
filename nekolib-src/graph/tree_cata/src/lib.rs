use std::collections::VecDeque;

pub struct TreeCata<T> {
    par: Vec<Option<(usize, usize, T)>>,
    order: Vec<(usize, usize)>,
    child: Vec<Vec<(usize, usize, T)>>,
}

impl<T> From<Vec<Vec<(usize, T)>>> for TreeCata<T> {
    fn from(mut g: Vec<Vec<(usize, T)>>) -> Self {
        let n = g.len();
        let mut par: Vec<_> = (0..n).map(|_| None).collect();
        let mut q: VecDeque<_> = vec![(0, n)].into();
        let mut order = vec![];
        let mut child: Vec<_> = (0..n).map(|_| vec![]).collect();

        while let Some((v, vi)) = q.pop_front() {
            order.push((v, vi));
            let gv = std::mem::take(&mut g[v]);
            for (i, (nv, w)) in (0..).zip(gv) {
                if nv == 0 || par[nv].is_some() {
                    par[v] = Some((nv, i, w));
                } else {
                    child[v].push((nv, i, w));
                    q.push_back((nv, i));
                }
            }
        }

        Self { par, order, child }
    }
}

impl<T> TreeCata<T> {
    pub fn cata<U: Clone>(
        &self,
        empty: U,
        mut map: impl FnMut(&U, &T) -> U,
        mut fold: impl FnMut(&U, &U) -> U,
    ) -> (Vec<U>, Vec<Vec<U>>) {
        let n = self.child.len();
        if n == 0 {
            return (vec![], vec![]);
        }

        // dp_fold[v] := 0 を根としたときの v 以下の部分木の fold
        // dp_map[v][i] := v を根としたときの i 番目の map
        // accl[v][i] := v を根としたときの ..i 番目の子の fold
        // accr[v][i] := v を根としたときの i.. 番目の子の fold

        let len = |v: usize| self.child[v].len() + if v == 0 { 0 } else { 1 };

        let empty = || empty.clone();
        let mut dp_fold = vec![empty(); n];
        let mut dp_map: Vec<_> =
            (0..n).map(|v| vec![empty(); len(v)]).collect();
        for &(v, vi) in self.order[1..].iter().rev() {
            dp_fold[v] = dp_map[v].iter().fold(empty(), |x, y| fold(&x, y));
            if let Some((pv, _, ref w)) = self.par[v] {
                dp_map[pv][vi] = map(&dp_fold[v], &w);
            }
        }

        let mut accl: Vec<_> =
            (0..n).map(|v| vec![empty(); len(v) + 1]).collect();
        let mut accr = accl.clone();

        for &(v, _) in &self.order {
            let children = &self.child[v];
            for i in 0..len(v) {
                accl[v][i + 1] = fold(&accl[v][i], &dp_map[v][i]);
            }
            for i in (0..len(v)).rev() {
                accr[v][i] = fold(&dp_map[v][i], &accr[v][i + 1]);
            }

            for &(u, i, ref w) in children {
                let j = self.par[u].as_ref().unwrap().1;
                let tmp = fold(&accl[v][i], &accr[v][i + 1]);
                dp_map[u][j] = map(&tmp, w);
            }
        }

        // dp_fold[v] := v を根としたときの部分木の fold
        let mut dp_fold = vec![empty(); n];
        for v in 0..n {
            dp_fold[v] = accr[v][0].clone();
        }

        (dp_fold, dp_map)
    }
}

#[test]
fn sanity_check() {
    let g = vec![
        vec![(1, 0), (2, 0)],
        vec![(0, 1), (3, 1), (4, 1), (5, 1)],
        vec![(0, 2)],
        vec![(1, 3)],
        vec![(1, 4)],
        vec![(1, 5)],
    ];

    let tc: TreeCata<_> = g.into();

    let empty = "".to_owned();
    let map = |x: &String, c: &usize| format!("({} {} )", x, c);
    let fold = |x: &String, y: &String| format!("{}{}", x, y);

    let (dp_fold, dp_map) = tc.cata(empty, map, fold);
    eprintln!("{dp_fold:?}");

    for i in 0..6 {
        eprintln!("{i}: {:?}", dp_map[i]);
    }
}
