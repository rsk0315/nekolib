use std::cmp::Ordering::{Equal, Greater, Less};

use scc::Scc;

pub fn twosat(
    len: usize,
    cnf: impl IntoIterator<Item = [(usize, bool); 2]>,
) -> Option<Vec<bool>> {
    let g = {
        let mut g = vec![vec![]; 2 * len];
        let index = |(index, not): (usize, bool)| 2 * index + not as usize;
        for [(index_x, not_x), (index_y, not_y)] in cnf {
            g[index((index_x, !not_x))].push(index((index_y, not_y)));
            g[index((index_y, !not_y))].push(index((index_x, not_x)));
        }
        g
    };

    let index = |&v: &_| v;
    let delta = |&v: &usize| g[v].iter().copied();
    let scc = Scc::new(0..2 * len, 2 * len, index, delta);
    let mut cert = vec![false; len];
    for x in 0..len {
        let x_true = 2 * x;
        let x_false = 2 * x + 1;
        match scc.comp_id(&x_true).cmp(&scc.comp_id(&x_false)) {
            Less => cert[x] = false,
            Equal => return None,
            Greater => cert[x] = true,
        }
    }
    Some(cert)
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn sanity_check_sat() {
        let cnf = vec![
            [(0, true), (1, false)],
            [(1, true), (2, true)],
            [(2, false), (3, false)],
            [(3, true), (0, false)],
        ];

        let res = twosat(4, cnf.clone());
        assert!(res.is_some());
        let cert = res.unwrap();
        assert!(
            cnf.iter()
                .all(|&[(ix, vx), (iy, vy)]| cert[ix] == vx || cert[iy] == vy)
        );
    }

    #[test]
    fn sanity_check_unsat() {
        let cnf = vec![
            [(0, true), (1, true)],
            [(0, true), (1, false)],
            [(0, false), (1, true)],
            [(0, false), (1, false)],
        ];

        let res = twosat(2, cnf);
        assert!(res.is_none());
    }
}
