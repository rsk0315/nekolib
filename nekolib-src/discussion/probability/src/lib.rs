//! 確率。
//!
//! ## Definitions
//!
//! TODO: 書く。
//! $`\gdef\P{\mathbb{P}}`$
//! $`\gdef\F{\mathcal{F}}`$
//! $`\gdef\E{\mathbb{E}}`$
// $`\gdef\Pr{\mathrm{Pr}}`$
//! $`\gdef\Pr{\P}`$
//!
//! $`(\Omega, \F, \P)`$, $`\P(\Omega) = 1`$
//!
//! $`\E[X]`$
//!
//! ## Lemmata
//!
//! TODO: 書く。
//!
//! ## Examples
//!
//! ### Markov chain
//!
//! 有限個の状態からなる集合 $`Q = \{q_0, q_1, \dots, q_{n-1}\}`$ を考える。
//! 時刻 $`t`$ に状態 $`q_i`$ にいるとき、時刻 $`t+1`$ に確率 $`p_{i, j}`$ で
//! $`q_j`$ に遷移する。各 $`i`$ に対して $`\sum_j p_{i, j} = 1`$ が成り立つ。
//! 各遷移は独立に行われるものとする。
//!
//! 状態 $`q_0`$ にいる状態から開始し、初めて $`q_{n-1}`$ に到達する時刻の期待値を求めよ、という問題を考える。
//!
//! まず、$`j \le i \implies p_{i, j} = 0`$ の問題設定について考える。
//! 時刻 $`0`$ に状態 $`q_i`$ にいるとき、初めて状態 $`q_{n-1}`$ に到達する時刻を $`X_i`$
//! とする。最終的な答えは $`\E[X_0]`$ である。
//!
//! 定義から明らかに $`\E[X_{n-1}] = 0`$ である。時刻 $`0`$ に状態 $`q_i`$
//! にいるとし、時刻 $`1`$ に各状態にいる確率を考えることで
//! ```math
//! \begin{aligned}
//! \E[X_i] &= 1 + \sum_{j=0}^{n-1} p_{i, j}\cdot \E[X_j] \\
//! &= 1 + \sum_{j=0}^i 0\cdot \E[X_j] + \sum_{j=i+1}^{n-1} p_{i, j}\cdot \E[X_j] \\
//! &= 1 + \sum_{j=i+1}^{n-1} p_{i, j}\cdot \E[X_j] \\
//! \end{aligned}
//! ```
//! を得る。すなわち、$`\E[X_i]`$ の計算のために必要な値は $`i \lt j`$ なる $`j`$ における
//! $`\E[X_j]`$ と $`p_{i, j}`$ のみであるため、$`i`$ の降順に計算することで $`\E[X_0]`$
//! を求められる。
//!
//! 続いて、$`j \lt i \implies p_{i, j} = 0`$ かつ $`i\lt n-1\implies p_{i, i} \in[0\lldot 1)`$
//! の問題設定について考える。$`X_i`$ の定義は先ほどと同様とする。
//! ```math
//! \begin{aligned}
//! \E[X_i] &= 1 + \sum_{j=0}^{n-1} p_{i, j}\cdot \E[X_j] \\
//! &= 1 + \sum_{j=0}^{i-1} 0\cdot \E[X_j] + \sum_{j=i}^{n-1} p_{i, j}\cdot \E[X_j] \\
//! &= 1 + p_{i, i}\cdot \E[X_i] + \sum_{j=i+1}^{n-1} p_{i, j}\cdot \E[X_j], \\
//! (1-p_{i, i})\cdot \E[X_i] &= 1 + \sum_{j=i+1}^{n-1} p_{i, j}\cdot \E[X_j], \\
//! \E[X_i] &= \frac1{1-p_{i, i}}\cdot\left(1 + \sum_{j=i+1}^{n-1} p_{i, j}\cdot \E[X_j]\right) \\
//! \end{aligned}
//! ```
//! よって、先ほどと同様に $`i`$ の降順に計算して $`E[X_0]`$ を求められる。
//!
//! 続いて、一般の場合について考える。ただし、任意の $`i`$ について、状態 $`q_i`$
//! から開始して有限時間で状態 $`q_{n-1}`$ に遷移できるような遷移の仕方が存在するものとする。各
//! $`i\lt n-1`$ についての $`\E[X_i] = 1 + \sum_j p_{i, j}\cdot \E[X_j]`$ を線形方程式系と見なせるので
//! ```math
//! \begin{aligned}
//! \left(\begin{matrix}
//! 1-p_{0, 0} & -p_{0, 1} & \cdots & -p_{0, n-2} & -p_{0, n-1} \\
//! -p_{1, 0} & 1-p_{1, 1} & \cdots & -p_{1, n-2} & -p_{1, n-1} \\
//! \vdots & \vdots & \ddots & \vdots & \vdots \\
//! -p_{n-2, 0} & -p_{n-2, 1} & \cdots & 1-p_{n-2,n-2} & -p_{n-2,n-1}
//! \end{matrix}\right)
//! \times
//! \left(\begin{matrix}
//! \E[X_0] \\
//! \E[X_1] \\
//! \vdots \\
//! \E[X_{n-2}] \\
//! 0
//! \end{matrix}\right)
//! =
//! \left(\begin{matrix}
//! \;0\; \\
//! 0 \\
//! \vdots \\
//! 0
//! \end{matrix}\right),
//! \end{aligned}
//! ```
//! すなわち
//! ```math
//! \begin{aligned}
//! \left(\begin{matrix}
//! 1-p_{0, 0} & -p_{0, 1} & \cdots & -p_{0, n-2} \\
//! -p_{1, 0} & 1-p_{1, 1} & \cdots & -p_{1, n-2} \\
//! \vdots & \vdots & \ddots & \vdots \\
//! -p_{n-2, 0} & -p_{n-2, 1} & \cdots & 1-p_{n-2,n-2}
//! \end{matrix}\right)
//! \times
//! \left(\begin{matrix}
//! \E[X_0] \\
//! \E[X_1] \\
//! \vdots \\
//! \E[X_{n-2}]
//! \end{matrix}\right)
//! =
//! \left(\begin{matrix}
//! \;0\; \\
//! 0 \\
//! \vdots \\
//! 0
//! \end{matrix}\right)
//! \end{aligned}
//! ```
//! を解けば（あるいは、解けないことを示せば）よい。
//! 対角成分が正、それ以外が非正であることからなにかが言える？
//!
//! 行列が特殊な形をしている場合は、掃き出し法などを介さずとも前述のように解くことができる。
//! See also [ABC 189 F](https://atcoder.jp/contests/abc189/tasks/abc189_f).
//!
//! ## Techniques
//!
//! ### Probability
//!
//! 何らかの条件を満たすまで、ある操作を繰り返すことを考える。操作を完了するまでの回数の期待値を求めたい。
//!
//! $`i`$ 回目の操作が行われるとき $`X_i = 1`$、そうでないとき $`X_i = 0`$ なる確率変数 $`X_i`$
//! を考える。高々 $`n`$ 回で完了する確率を $`p(n)`$ とすると、求める期待値は
//! ```math
//! \begin{aligned}
//! \E{\left[\sum_{i=1}^{\infty} X_i\right]}
//! &= \E{\left[\sum_{i=0}^{\infty} {(1-p(i))}\right]}
//! \end{aligned}
//! ```
//! である。
//!
//! See also [ABC 331 G](https://atcoder.jp/contests/abc331/tasks/abc331_g).
//!
//! また、非負整数の値を取る確率変数 $`X`$ に対して、
//! ```math
//! \begin{aligned}
//! \E[X] &= \sum_{i=0}^{\infty} i\cdot\Pr(X = i) \\
// &= \sum_{i=1}^{\infty} \sum_{j=i}^{\infty} \Pr(X=j) \\
//! &= \sum_{i=0}^{\infty} i\cdot(\Pr(X\ge i) - \Pr(X\gt i)) \\
//! &= \sum_{i=0}^{\infty} i\cdot\Pr(X\ge i) - \sum_{i=0}^{\infty} i\cdot\Pr(X\ge i+1) \\
//! &= \sum_{i=1}^{\infty} i\cdot\Pr(X\ge i) - \sum_{i=1}^{\infty} {(i-1)\cdot\Pr(X\ge i)} \\
//! &= \sum_{i=1}^{\infty} \Pr(X\ge i)
//! \end{aligned}
//! ```
//! が成り立ち、$`\Pr(X = i)`$ を求める問題から $`\Pr(x\ge i)`$ を求める問題に帰着できる。
//!
//! ### Powers
//!
//! 各 $`i\in\Lambda_n`$ に対して $`X_i \in \{0, 1\}`$ となるような確率変数 $`X_i`$
//! を考える。ただし、各 $`X_i`$ 同士は独立とは限らないとする。これに対し、
//! ```math
//! \E{\left[\left(\sum_{i=0}^{n-1} X_i\right)^2\right]}
//! ```
//! を求めたいとする。
//! ```math
//! \begin{aligned}
//! \E{\left[\left(\sum_{i=0}^{n-1} X_i\right)^2\right]}
//! &= \E{\left[\sum_{i=0}^{n-1} X_i^2 + 2 \sum_{i=0}^{n-1} \sum_{j=i+1}^{n-1} X_i X_j\right]} \\
//! &= \sum_{i=0}^{n-1} \E{\left[X_i^2\right]} + 2 \sum_{i=0}^{n-1} \sum_{j=i+1}^{n-1} {\E[X_i X_j]} \\
//! &= \sum_{0\le i\lt n} \Pr(X_i = 1) + 2 \sum_{0\le i\lt j\lt n} \Pr(X_i = X_j = 1)
//! \end{aligned}
//! ```
//! より、$`\sum_{0\le i\lt n} \Pr(X_i = 1)`$ や $`\sum_{0\le i\lt j\lt n} \Pr(X_i = X_j = 1)`$
//! を求める問題に帰着できる。より一般には、$`\E[(\sum_{i=0}^{n-1} X_i)^M]`$ は、各
//! $`1\le m\le M`$ に対して
//! ```math
//! \sum_{(i_0, \dots, i_{m-1}) \in \binom{\Lambda_n}{m}} \Pr{\left(\bigwedge_{j=0}^{m-1} X_{i_j} = 1\right)}
//! ```
//! を求める問題に帰着できる。係数は Stirling partition number になる。
//!
//! DP などで、任意の $`0\le i_0\lt i_1\lt \dots \lt i_{m-1}\lt n`$ にわたる
//! $`\Pr(X_{i_0} = X_{i_1} = \dots = X_{i_{m-1}} = 1)`$ の総和を求められる場合は、これを利用するのがよいであろう。
//!
//! See also [ABC 277 G](https://atcoder.jp/contests/abc277/tasks/abc277_g).
