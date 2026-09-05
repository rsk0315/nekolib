//! 確率。
//!
//! ## Definitions
//!
//! TODO: 精査する。
//! $`\gdef\P{\mathbb{P}}`$
//! $`\gdef\F{\mathcal{F}}`$
//! $`\gdef\E{\mathbb{E}}`$
// $`\gdef\Pr{\mathrm{Pr}}`$
//! $`\gdef\Pr{\P}`$
//! $`\gdef\compl#1{#1^{\complement}}`$
//! $`\gdef\B{\mathcal B}`$
//! $`\gdef\L{\mathcal L}`$
//! $`\gdef\1{\mathbf 1}`$
//!
//! ### Probability spaces
//!
//! 集合 $`S`$ の部分集合族 $`\Sigma`$ が $`S`$ 上の **$`\sigma`$-代数** (*$`\sigma`$-algebra*)
//! であるとは、以下の 3 つの条件を満たすことをいう。
//! - $`S\in\Sigma`$,
//! - $`F\in\Sigma \implies \compl{F}\in\Sigma`$,
//! - 任意の $`I\subseteq\N`$ に対し、$`\{F_n\}_{n\in I}\subseteq\Sigma \implies \bigcup_{n\in I} F_n\in\Sigma`$.
//!
//! $`I\subseteq\N`$ を任意に取り、$`i\in I`$ に対して $`\F_i`$ を $`S`$ 上の $`\sigma`$-代数とする。このとき、$`\bigcup_{i\in I} \F_i`$ は $`\sigma`$-代数となる。$`\mathcal C`$ を $`S`$ の部分集合族とするとき、$`\sigma(\mathcal C)`$ で $`\mathcal C`$
//! を含む最小の $`\sigma`$-代数を表す。これは、そのようなすべての $`\sigma`$-代数の共通部分として定義される。
//! $`\R`$ のすべての開集合を含む最小の $`\sigma`$-代数を $`\B(\R)`$ と書き、*Borel $`\sigma`$-algebra* と呼ぶ。
//!
//! 関数 $`\mu\colon\Sigma\to[0\lldot+\infty]`$ が **$`\sigma`$-加法的** (*$`\sigma`$-additive*)
//! であるとは、次の 2 つの条件を満たすことをいう。
//! - $`\mu(\emptyset) = 0`$,
//! - $`(i, j)\in \binom I2 \implies F_i\cap F_j = \emptyset`$ かつ $`\bigcup_{n\in I} F_n \in \Sigma`$ なる任意の $`I\in\N`$ に対し、$`\mu(\bigcup_{n\in I} F_n) = \sum_{n\in I} \mu(F_n)`$.
//!
//! $`\Sigma`$ を $`S`$ 上の $`\sigma`$-代数とする。このとき $`(S, \Sigma)`$ を **可測空間**
//! (*measurable space*) という。$`\Sigma`$ 上の $`\sigma`$-加法的関数 $`\mu`$ を **測度**
//! (*measure*) といい、$`(S, \Sigma, \mu)`$ を **測度空間** (*measure space*) という。
//!
//! 測度空間 $`(\Omega, \F, \P)`$ が $`\P(\Omega) = 1`$ を満たすとき、$`\P`$ を **確率測度**
//! (*probability measure*) といい、$`(\Omega, \F, \P)`$, $`\P(\Omega)`$ を **確率空間**
//! (*probability space*) という。$`\F`$ に含まれる集合を **事象** (*event*) という。
//!
//! <span style="font-size: 75%">Caratheodory の拡張定理と、拡張の一意性を述べる必要がありそう。</span>
//!
//! ### Random variables
//!
//! $`(S, \Sigma, \mu)`$ を可測空間とし、$`\B = \B(\R)`$ とする。
//!
//! $`h\colon S\to\R`$ とし、$`A\subseteq \R`$ に対して $`h^{-1}(A) = \{s\in S\mid h(s)\in A\}`$
//! と定義する。任意の $`B\in\B`$ に対して $`h^{-1}(B)\in\Sigma`$ となるとき、$`h`$
//! は **$`\Sigma`$-可測** (*$`\Sigma`$-measurable*) であるという。$`\Sigma`$-可測な関数を
//! $`m\Sigma`$ と書く。そのうち、非負のものと有界のものを、それぞれ $`(m\Sigma)^+`$
//! および $`b\Sigma`$ と書く。
//!
//! 確率空間 $`(\Omega, \F, \P)`$ 上の可測関数<span style="font-size: 75%">（$`\F`$-可測な関数？）</span>を
//! **確率変数** (*random variable*) という。
//!
//! $`X\colon\Omega\to\R`$ を $`(\Omega, \F, \P)`$ 上の確率変数とする。*the law of $`X`$* は
//! $`\L_X = \P\circ X^{-1}`$ で、$`(\R, \B)`$ 上の確率測度である。$`\L_X`$ は、
//! **$`X`$ の分布関数** (*distribution function of $`X`$*) $`F_X(x) = \P[X\le x]`$
//! ($`x\in\R`$) で定められる。
//!
//! ### Independence
//!
//! $`(\Omega, \F, \P)`$ を確率空間とする。
//!
//! $`\F`$ の部分 $`\sigma`$-代数 $`\mathcal G_1, \mathcal G_2, \dots`$ が **独立**
//! (*independent*) であるとは、任意の相異なる $`i_1, i_2, \dots, i_n`$ に対して
//! ```math
//! \P[G_{i_1}\cap\dots\cap G_{i_n}] = \prod_{j=1}^n \P[G_{i_j}]
//! ```
//! が成り立つことをいう。ただし、任意の $`i\in\N`$ に対して $`G_i\in\mathcal G_i`$
//! とする。確率変数 $`X_1, X_2, \dots`$ が独立であるとは、$`\sigma`$-代数
//! $`\sigma(X_1), \sigma(X_2), \dots`$ が独立であることをいう。
//!
//! 事象 $`E_1, E_2, \dots`$ が独立であるとは、$`\mathcal E_i = \{\emptyset, E_i, \compl{E_i}, \Omega\}`$
//! で定義される $`\sigma`$-代数 $`\mathcal E_1, \mathcal E_2, \dots`$ が独立であることをいう。
//!
//! <span style="font-size: 75%">*independent and identically distributed* について触れる必要がありそう。</span>
//!
//! ### Expectation
//!
//! $`(S, \Sigma, \mu)`$ を可測空間とする。集合 $`A`$ 上の **特性関数**
//! (*characteristic function*) を
//! ```math
//! \1_A(x) = \begin{cases}
//! 1, & \text{if~}s\in A; \\
//! 0, & \text{if~}s\notin A
//! \end{cases}
//! ```
//! で定義する。**単関数** (*simple function*) とは、次の形で書ける関数のことをいう。
//! ```math
//! f = \sum_{k=1}^m a_k\cdot\1_{A_k}.
//! ```
//! ただし、各 $`k`$ に対し $`a_k\in[0\lldot+\infty]`$ かつ $`A_k\in\Sigma`$
//! とする。単関数全体からなる集合を $`\mathrm{SF}^+`$ と書く。$`f`$ の **積分** (*integral*) を
//! ```math
//! \mu(f) = \sum_{k=1}^m a_k\cdot\mu(A_k)
//! ```
//! で定義する。
//! <span style="font-size: 75%">$`\mu\colon \mathrm{SF}^+\ni f\mapsto \mu(f)\in\R`$ と、$`\mu\colon \Sigma\ni A_k\mapsto \mu(A_k)\in[0\lldot+\infty]`$ が混在していてややこしい。</span>
//!
//! 関数 $`f`$ に対し、$`f^+`$ と $`f^-`$ でそれぞれ $`f`$ の正・負の部分を表す。
//! ```math
//! f^+(s) = \max{\{f(s), 0\}}, \qquad f^-(s) = \max{\{-f(s), 0\}}.
//! ```
//! $`|f| = f^+ + f^-`$ に注意せよ。$`\mu(f^+) + \mu(f^-) \lt +\infty`$
//! であるとき、$`\mu(f) = \mu(f^+) - \mu(f^-)`$ と定義する。
//!
//! $`(\Omega, \F, \P)`$ を確率空間とする。非負の確率変数 $`X`$ に対し、$`P`$ 上の $`X`$
//! の積分を $`\E[X]`$ と書き $`X`$ の **期待値** (*expectation*)
//! という。一般に、$`\E[X^+] + \E[X^-] \lt +\infty`$ のとき
//! $`\E[X] = \E[X^+] - \E[X^-]`$ と定義する。そのような **積分可能な確率変数**
//! (*integrable random variables*) 全体からなる集合を $`L^1(\Omega, \F, \P)`$ と書く。
//!
//! $`X\colon\Omega\to\R_{\ge 0}`$ が
//! ```math
//! X(\omega) = \sum_{E\in\F} x_E\cdot \1_E(\omega)
//! ```
//! の形で書けるとして、
//! ```math
//! \E[X] = \P(X) = \sum_{E\in\F} x_E\cdot \P(E)
//! ```
//! と定義されるということ？
//!
//! ## Lemmata
//!
//! TODO: 書く。
//!
//! 確率変数 $`X_1, X_2 \in L^1(\Omega, \F, \P)`$ に対し、
//! ```math
//! a_1, a_2 \in \R \implies \E[a_1X_1+a_2X_2] = a_1\E[X_1] + a_2\E[X_2]
//! ```
//! が成り立つ。
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
//! 非負整数の値を取る確率変数 $`X`$ に対して、
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
//! が成り立ち、$`\Pr(X = i)`$ を求める問題から $`\Pr(X\ge i)`$ を求める問題に帰着できる。
//!
//! また、何らかの条件を満たすまで何らかの操作を繰り返すことを考える。操作を完了するまでの回数の期待値を求めたい。
//!
//! $`i`$ 回目の操作が行われるとき $`X_i = 1`$、そうでないとき $`X_i = 0`$ となる確率変数 $`X_i`$
//! を考える。高々 $`n`$ 回で完了する確率を $`p(n)`$ とすると、求める期待値は
//! ```math
//! \begin{aligned}
//! \E{\left[\sum_{i=1}^{\infty} X_i\right]}
// &= \E{\left[\sum_{i=0}^{\infty} {(1-p(i))}\right]}
//! &= \sum_{i=1}^{\infty} \E[X_i] \\
//! &= \sum_{i=1}^{\infty} {(0\cdot p(i-1) + 1\cdot(1-p(i-1)))} \\
//! &= \sum_{i=0}^{\infty} {(0\cdot p(i) + 1\cdot(1-p(i)))} \\
//! &= \sum_{i=0}^{\infty} {(1-p(i))} \\
//! \end{aligned}
//! ```
//! である。操作回数を表す確率変数を $`X`$ とすると $`\P(X\le i) = p(i)`$ なので、先の補題より
//! ```math
//! \begin{aligned}
//! \E[X] &= \sum_{i=1}^{\infty} \P(X\ge i) \\
//! &= \sum_{i=1}^{\infty} {(1-\P(X\lt i))} \\
//! &= \sum_{i=1}^{\infty} {(1-\P(X\le i-1))} \\
//! &= \sum_{i=0}^{\infty} {(1-\P(X\le i))} \\
//! &= \sum_{i=0}^{\infty} {(1-p(i))} \\
//! \end{aligned}
//! ```
//! とすることもできる。
//!
//! See also [ABC 331 G](https://atcoder.jp/contests/abc331/tasks/abc331_g).
//!
//! ### Powers
//!
//! 各 $`i\in\Lambda_n`$ に対して $`X_i \in \{0, 1\}`$ となるような確率変数 $`X_i`$
//! を考える。ただし、各 $`X_i`$ 同士は独立とは限らないとする。これに対し、
//! ```math
//! \E{\left[\left(\sum_{i=0}^{n-1} X_i\right)^2\,\right]}
//! ```
//! を求めたいとする。
//! ```math
//! \begin{aligned}
//! \E{\left[\left(\sum_{i=0}^{n-1} X_i\right)^2\,\right]}
//! &= \E{\left[\sum_{i=0}^{n-1} X_i^2 + 2 \sum_{i=0}^{n-1} \sum_{j=i+1}^{n-1} X_i X_j\right]} \\
//! &= \sum_{i=0}^{n-1} \E{\left[X_i^2\right]} + 2 \sum_{i=0}^{n-1} \sum_{j=i+1}^{n-1} {\E[X_i X_j]} \\
//! &= \sum_{0\le i\lt n} \Pr(X_i = 1) + 2 \sum_{0\le i\lt j\lt n} \Pr(X_i = X_j = 1)
//! \end{aligned}
//! ```
//! より、$`\sum_{0\le i\lt n} \Pr(X_i = 1)`$ や $`\sum_{0\le i\lt j\lt n} \Pr(X_i = X_j = 1)`$
//! を求める問題に帰着できる。より一般には、$`\E[(\sum_{i=0}^{n-1} X_i)^M]`$ は、各
//! $`1\le m\le M`$ に対する
//! ```math
//! \sum_{(i_0, \dots, i_{m-1}) \in \binom{\Lambda_n}{m}} \Pr{\left(\bigwedge_{j=0}^{m-1} X_{i_j} = 1\right)}
//! ```
//! の線形和を求める問題に帰着できる。係数は Stirling partition number になる。
//!
//! DP などで、任意の $`0\le i_0\lt i_1\lt \dots \lt i_{m-1}\lt n`$ にわたる
//! $`\Pr(X_{i_0} = X_{i_1} = \dots = X_{i_{m-1}} = 1)`$ の総和を求められる場合は、これを利用するのがよいであろう。
//!
//! See also [ABC 277 G](https://atcoder.jp/contests/abc277/tasks/abc277_g).
//!
//! ## References
//!
//! - Roch, Sébastien. *Modern Discrete Probability: An Essential Toolkit*. of *Cambridge Series in Statistical and Probabilistic Mathematics*. Cambridge: Cambridge University Press, 2024. <https://doi.org/10.1017/9781009305129>.
