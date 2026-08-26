pub struct IncrementalFnTable<T, Fi, Fj> {
    step: usize,
    table: Vec<Vec<T>>,
    inc_i: Fi,
    inc_j: Fj,
}

impl<T, Fi, Fj> IncrementalFnTable<T, Fi, Fj>
where
    T: Clone,
    Fi: Fn((usize, usize), &T) -> T,
    Fj: Fn((usize, usize), &T) -> T,
{
    pub fn new(n: usize, step: usize, f_00: T, inc_i: Fi, inc_j: Fj) -> Self {
        let len = n / step;
        let mut table = vec![vec![f_00; len + 1]; len + 1];
        let mut f_i0 = table[0][0].clone();
        for il in 0..=len {
            let mut f_ij = f_i0.clone();
            table[il][0] = f_ij.clone();
            for jl in 0..len {
                for js in 0..step {
                    f_ij = inc_j((il * step, jl * step + js), &f_ij);
                }
                table[il][jl + 1] = f_ij.clone();
            }
            if il == len {
                break;
            }
            for is in 0..step {
                f_i0 = inc_i((il * step + is, 0), &f_i0);
            }
        }

        Self { step, table, inc_i, inc_j }
    }

    pub fn get(&self, i: usize, j: usize) -> T {
        let il = i / self.step;
        let jl = j / self.step;
        let mut f_ij = self.table[il][jl].clone();
        for i in il * self.step..i {
            f_ij = (self.inc_i)((i, jl * self.step), &f_ij);
        }
        for j in jl * self.step..j {
            f_ij = (self.inc_j)((i, j), &f_ij);
        }
        f_ij
    }
}

#[cfg(test)]
fn test_binom_acc(n: usize, step: usize) {
    let binom = {
        let mut dp = vec![vec![0_u128; n + 1]; n + 1];
        dp[0][0] = 1;
        for i in 1..=n {
            dp[i][0] = 1;
            for j in 1..=i {
                dp[i][j] = dp[i - 1][j - 1] + dp[i - 1][j];
            }
        }
        dp
    };

    let binom_acc = {
        let mut dp = binom.clone();
        for i in 0..=n {
            for j in 0..n {
                dp[i][j + 1] += dp[i][j];
            }
        }
        dp
    };

    let inc_i = |(i, j): (usize, usize), f_ij: &u128| 2 * f_ij - binom[i][j];
    let inc_j = |(i, j): (usize, usize), f_ij: &u128| f_ij + binom[i][j + 1];

    let ift = IncrementalFnTable::new(n, step, binom[0][0], inc_i, inc_j);

    for i in 0..=n {
        for j in 0..=n {
            assert_eq!(ift.get(i, j), binom_acc[i][j]);
        }
    }
}

#[cfg(test)]
fn test_binom_x_acc(n: usize, step: usize) {
    let binom = {
        let mut dp = vec![vec![0_u128; n + 1]; n + 1];
        dp[0][0] = 1;
        for i in 1..=n {
            dp[i][0] = 1;
            for j in 1..=i {
                dp[i][j] = dp[i - 1][j - 1] + dp[i - 1][j];
            }
        }
        dp
    };

    let binom_x = {
        let mut dp = binom.clone();
        for i in 0..=n {
            for j in 0..=i {
                dp[i][j] *= j as u128;
            }
        }
        dp
    };

    let binom_x_acc = {
        let mut dp = binom_x.clone();
        for i in 0..=n {
            for j in 0..n {
                dp[i][j + 1] += dp[i][j];
            }
        }
        dp
    };

    let inc_i = |(i, j): (usize, usize), f_ij: &(u128, u128)| {
        (
            2 * f_ij.0 - binom[i][j],
            2 * f_ij.1 + f_ij.0 - binom[i][j] * (j + 1) as u128,
        )
    };
    let inc_j = |(i, j): (usize, usize), f_ij: &(u128, u128)| {
        (f_ij.0 + binom[i][j + 1], f_ij.1 + binom[i][j + 1] * (j + 1) as u128)
    };

    let ift = IncrementalFnTable::new(n, step, (1, 0), inc_i, inc_j);

    for i in 0..=n {
        for j in 0..=n {
            assert_eq!(ift.get(i, j).1, binom_x_acc[i][j]);
        }
    }
}

#[test]
fn sanity_check() {
    for n in 1..=100 {
        for step in 1..=n {
            test_binom_acc(n, step);
            test_binom_x_acc(n, step);
        }
    }
}
