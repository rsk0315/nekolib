//! 動的計画法。
//!
//! ## Notations
//!
//! $`\Lambda_n = \{0, 1, \dots, n-1\}`$ とする。
//! $`\gdef\M{\mathsf{M}}`$
//! $`\gdef\MM{\mathsf{MM}}`$
//!
//! 積が $`n`$ 次になる多項式乗算の時間計算量を $`\M(n)`$ とする。$`\M(n)\in O(n^2)\cap\Omega(n)`$ とする。$`\M(n_1+n_2) \ge \M(n_1)+\M(n_2)`$ が成り立つ。
//!
//! $`n`$ 次正方行列同士の乗算の時間計算量を $`\MM(n)`$ とする。$`\MM(n)\in O(n^3)\cap\Omega(n^2)`$ とする。
//! 古典的には $`\MM(n) = \Theta(n^3)`$ や $`\MM(n) = \Theta(n^{\log_2(7)})`$ などである。
//! 最近は $`\MM(n) = O(n^{2.371177})`$ が達成されているらしい？
//!
//! ## Idea
//!
//! ### Variants
//!
//! #### 区間での分割全体からなる集合
//!
//! 長さ $`n`$ の配列をいくつかの非空な区間に分割したものを考える。
//! すなわち、$`[0\lldot n) = [i_0\lldot i_1)\sqcup[i_1\lldot i_2)\sqcup\dots\sqcup[i_{k-1}\lldot i_k)`$
//! とする。ただし、$`(i_0, i_k) = (0, n)`$ である。
//! この分割全体からなる集合 $`\dp[n]`$ を考えたい。
//!
//! $`\dp[0] = \{\emptyset\}`$ である。$`i\ge 1`$ に対して下記が成り立つ。
//! ```math
//! \dp[n] = \{\{[0\lldot n)\}\}\sqcup\bigsqcup_{i\in\Lambda_n} \{S\sqcup\{[n-1\lldot n)\} \mid S\in\dp[n-1]\}.
//! ```
//!
//! 分割 $`[l\lldot r)`$ に関する値 $`f(l, r)`$ に対し、$`f(i_0, i_1)\circ\dots\circ f(i_{k-1}, i_k)`$
//! のすべての分割における $`\ast`$-fold を求めるのに使える場合がある ([ABC 224 F])。
//!
//! [ABC 224 F]: https://atcoder.jp/contests/abc224/tasks/abc224_f
//!
//! ### Speed-up
//!
//! #### 行列累乗
//!
//! $`\dp[i][j]`$ ($`i, j \in \Lambda_n`$) を考える。
//! $`\dp[0][j]`$ ($`j\in\Lambda_n`$) は与えられているとし、$`i\ge 0`$ について下記が成り立つとする。
//! ```math
//! \dp[i+1][j] = \sum_{t=0}^{n-1} a_{j, t}\cdot \dp[i][t].
//! ```
//! すなわち、下記が成り立つ。
//! ```math
//! \left(\begin{matrix}
//! \dp[i+1][0] \\
//! \dp[i+1][1] \\
//! \vdots \\
//! \dp[i+1][n-1]
//! \end{matrix}\right)
//! =
//! \left(\begin{matrix}
//! a_{0, 0} & a_{0, 1} & \cdots & a_{0, n-1} \\
//! a_{1, 0} & a_{1, 1} & \cdots & a_{1, n-1} \\
//! \vdots & \vdots & \ddots & \vdots \\
//! a_{n-1, 0} & a_{n-1, 1} & \cdots & a_{n-1, n-1}
//! \end{matrix}\right)
//! \times
//! \left(\begin{matrix}
//! \dp[i][0] \\
//! \dp[i][1] \\
//! \vdots \\
//! \dp[i][n-1]
//! \end{matrix}\right).
//! ```
//! このとき、下記が成り立つ。
//! ```math
//! \left(\begin{matrix}
//! \dp[m][0] \\
//! \dp[m][1] \\
//! \vdots \\
//! \dp[m][n-1]
//! \end{matrix}\right)
//! =
//! \left(\begin{matrix}
//! a_{0, 0} & a_{0, 1} & \cdots & a_{0, n-1} \\
//! a_{1, 0} & a_{1, 1} & \cdots & a_{1, n-1} \\
//! \vdots & \vdots & \ddots & \vdots \\
//! a_{n-1, 0} & a_{n-1, 1} & \cdots & a_{n-1, n-1}
//! \end{matrix}\right)^m
//! \times
//! \left(\begin{matrix}
//! \dp[0][0] \\
//! \dp[0][1] \\
//! \vdots \\
//! \dp[0][n-1]
//! \end{matrix}\right).
//! ```
//! よって、各 $`j\in\Lambda_n`$ について $`\dp[m][j]`$ を $`O(\MM(n)\log(m)+m)`$ 時間で求められる。
//!
//! 特に、$`\dp[n]`$ が $`(\dp[0], \dp[1], \dots, \dp[n-1])^{\top}`$ の線形和で表せるときは、
//! $`(\dp'[i][0], \dp'[i][1], \dots, \dp'[i][n-1]) = (\dp[i], \dp[i+1], \dots, \dp[i+n-1])`$
//! と見なすことで前述の形に帰着できる。
//!
//! 下記のような形で表される DP も、適切な行列を考えることで上記の形式で扱える
//! ([ref](https://twitter.com/rsk0315_h4x/status/1634874645588643840))。
//! ```math
//! \dp[n+1] = d\cdot\dp[n] + \sum_{j=0}^k c_j \sum_{i=1}^{n-1} i^j\cdot \dp[n-i] + \sum_{j=0}^m q_jn^j + r\cdot a^n.
//! ```
//!
//! #### 多項式積
//!
//! $`\dp[i][j]`$ ($`i\in\Lambda_n`$, $`j\in\Lambda_m`$) を考える。
//! $`\dp[0][0] = 1`$ かつ $`\dp[0][j] = 0`$ ($`j\gt 0`$) とし、$`i\ge 0`$
//! について下記が成り立つとする。
//! ```math
//! \dp[i+1][j] = \sum_{t=0}^{d_i} a_{i, t}\cdot \dp[i][j-t]
//! ```
//! ただし、$`j'\lt 0`$ のとき $`\dp[i][j'] = 0`$ とする。
//! 各 $`i\in\Lambda_{n+1}`$ について
//! ```math
//! f_i(x) = \sum_{j=0}^{\infty} \dp[i][j]\cdot x^j,
//! ```
//! 各 $`i\in\Lambda_n`$ について
//! ```math
//! a_i(x) = \sum_{t=0}^{d_i} a_{i, t}\cdot x^t
//! ```
//! とすると、
//! ```math
//! f_{i+1}(x) = a_i(x) \cdot f_i(x)
//! ```
//! であり、
//! ```math
//! f_n(x) = \prod_{i=0}^{n-1} a_i(x)
//! ```
//! が成り立つ。
//! $`d = \sum_{i=0}^{n-1} d_i`$ として $`O(\M(d)\log(n))`$ 時間で $`f_n(x)`$ を求められる。
//!
//! *Proof*.
//!
//! 簡単のため $`n`$ は $`2`$ べきとする。
//! $`n/2`$ 個のペアを作り、それぞれの積を求めることを考える。
//!
//! ```math
//! \begin{aligned}
//! &\phantom{{}={}} \M(d_0+d_1) + \M(d_2+d_3) + \dots + \M(d_{n-2}+d_{n-1}) \\
//! &\le \M((d_0+d_1) + (d_2+d_3) + \dots + (d_{n-2}+d_{n-1})) \\
//! &= \M(d)
//! \end{aligned}
//! ```
//! が成り立つ。これにより、$`n`$ を $`n/2`$ で置き換えた問題に $`O(\M(d))`$ 時間で帰着できる。
//! これを $`\log_2(n)`$ 回繰り返すことで所望の $`d`$ 次式が得られるため、全体で $`O(\M(d)\log(n))`$ 時間となる。$`\qed`$
//!
//! ## See also
//!
//! - <https://rsk0315.hatenablog.com/entry/2023/09/10/225138>
