//! 母関数を考える際に必要となる操作。
//!
//! ## Notations
//!
//! $`n`$ 次多項式 $`f(x) = \sum_{i=0}^n a_ix^i`$ に対し、$`[x^i]\, f(x) = a_i`$ とする。
//!
//! ## Basics
//!
//! $`a(x) = \sum_{i=0}^n a_ix^i`$ かつ $`b(x) = \sum_{i=0}^n b_ix^i`$ とする。
//! ```math
//! \begin{aligned}
//! [x^i]\, \frac{a(x)}{1-x} &= \sum_{j=0}^i a_j, \\
//! [x^i]\, \bigl(a(x)\cdot b(x)\bigr) &= \sum_{j=0}^i a_jb_{i-j}
//! \end{aligned}
//! ```
//!
//! すなわち、累積和や畳み込みは多項式の演算に対応すると見なせる。
//!
//! $`\sum`$ の添字の入れ替えを悩まずできると吉。
//!
//! ```math
//! \begin{aligned}
//! \sum_{i=L}^R f(i) &= \sum_{i=L+1}^{R+1} f(i-1)
//! = f((L+1)-1) + f((L+2)-1) + \dots + f((R+1)-1), \\
//! \sum_{i=L}^R f(i) &= \sum_{R-i=0}^{R-L} f(L+(R-i))
//! = f(L+0) + f(L+1) + \dots + f(L+(R-L)) \\
//! &= \sum_{R-i=0}^{R-L} f(R-(R-i))
//! = f(R-0) + f(R-1) + \dots + f(R-(R-L)).
//! \end{aligned}
//! ```
//!
//! ## Convolution-like
//!
//! $`a_i = a^{\leftarrow}_{n-i}`$ とする。
//!
//! ```math
//! \begin{aligned}
//! \sum_{i=0}^n \left(\sum_{j=i}^n a_jb_{j-i}\right)x^i
//! &= \sum_{i=0}^n \left(\sum_{j=i}^n a^{\leftarrow}_{n-j}b_{j-i}\right)x^i \\
//! &= \sum_{i=0}^n \left(\sum_{n-j=0}^{n-i} a^{\leftarrow}_{n-(i+n-j)}b_{(i+n-j)-i}) \right) x^i \\
//! &= \sum_{i=0}^n \left(\sum_{n-j=0}^{n-i} a^{\leftarrow}_{(n-i)-(n-j)}b_{n-j}) \right) x^i \\
//! &= \sum_{n-i=0}^n \left(\sum_{n-j=0}^{n-i} a^{\leftarrow}_{(n-i)-(n-j)}b_{n-j}) \right) x^i \\
//! &= \sum_{i'=0}^n \left(\sum_{j'=0}^{i'} a^{\leftarrow}_{i'-j'}b_{j'} \right) x^{n-i'} \\
//! &= \sum_{i=0}^n \left([x^i]\, \bigl(a^{\leftarrow}(x)\cdot b(x)\bigr) \right) x^{n-i}. \\
//! \end{aligned}
//! ```
//!
//! ## Operations
//!
//! ### Reciprocal
//!
//! $`f(x)\cdot g(x) \bmod x^k = 1`$ なる $`g(x)`$ を求める。
//!
//! ### Square root
//!
//! $`g(x)^2 \bmod x^k = f(x)`$ なる $`g(x)`$ を求める。
//!
//! ### Exponential
//!
//! 下記で定義される $`\exp(f(x))`$ に対して、$`\exp(f(x))\bmod x^k`$ を求める。
//! ```math
//! \exp(f(x)) = \sum_{n=0}^{\infty} \frac{f(x)^n}{n!}.
//! ```
//!
//! ### Logarithm
//!
//! $`[x^0]\, f(x) = 1`$ なる $`f`$ に対し、$`\log(f(x)) \bmod x^k`$ を求める。
//! ```math
//! \log(1-f(x)) = -\sum_{n=1}^{\infty} \frac{f(x)^n}n.
//! ```
//!
//! $`\tfrac{\mathrm d}{\mathrm dx} \log(f(x)) = f'(x)\cdot f(x)^{-1}`$ や
//! $`\log(f(x)\cdot g(x)) = \log(f(x)) + \log(g(x))`$ などが成り立つ。
//!
//! ### Power
//!
//! $`f(x)`$ と $`k`$ に対して $`f(x)^k`$ を求める。
//! $`f(x)^0 = 1`$ および、$`k\ne 0`$ に対して $`0^k = 0`$ が成り立つ。
//!
//! 以下、$`f(x)\ne 0`$ かつ $`k\ne 0`$ とする。
//! ある $`l`$ と $`g(x)`$ に対して $`f(x) = a_l x^l\cdot(1+g(x)\cdot x)`$ が成り立つので、
//! ```math
//! \begin{aligned}
//! f(x)^k
//! &= \bigl(a_lx^l\cdot (1+g(x)\cdot x)\bigr)^k \\
//! &= a_l^k x^{lk} \cdot (1+g(x)\cdot x)^k \\
//! &= a_l^k x^{lk} \cdot \exp(k\cdot \log(1+g(x)\cdot x))
//! \end{aligned}
//! ```
//! により求められる。
//!
//! ### Circular
//!
//! $`\cos(f(x))`$ および $`\sin(f(x))`$ を求める。
//!
//! ### Polynomial equation
//!
//! $`f(y) \equiv 0 \pmod{x^k}`$ なる $`y`$ を求める。
//! $`(y, k) \mapsto f(y)\cdot f'(y)^{-1} \bmod x^k`$ の oracle を用いる。
//!
//! ### First-order derivative equation
//!
//! $`y' \equiv f(y, x) \pmod{x^k}`$ を満たす $`y(x)`$ を求める。
//! $`(y, k) \mapsto (f(y, x) \bmod x^k, f'(y, x) \bmod x^k)`$ の oracle を用いる。
//!
//! ## See also
//!
//! - Taylor shift ([ABC 215 G](https://atcoder.jp/contests/abc215/editorial/2529))
