//! 母関数を考える際に必要となる操作。
//!
//! ## Notations
//!
//! $`n`$ 次多項式 $`f(x) = \sum_{i=0}^n a_ix^i`$ に対し、$`[x^i]\, f(x) = a_i`$ とする。
//! $`\gdef\e{\mathrm e}`$
//! $`\gdef\d{\mathrm d}`$
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
//! $`\tfrac{\d}{\d x} \log(f(x)) = f'(x)\cdot f(x)^{-1}`$ や
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
//! $`\varphi(y) \equiv 0 \pmod{x^k}`$ なる $`y`$ を求める。
//! $`(y, k) \mapsto \varphi(y)\cdot \left(\tfrac{\d}{\d y}\varphi(y)\right)^{-1} \bmod x^k`$ の
//! oracle を用いる。$`\tfrac{\d x}{\d y} = 0`$ として考えるっぽい。
//!
//! ### First-order derivative equation
//!
//! $`y' \equiv f(y, x) \pmod{x^k}`$ を満たす $`y(x)`$ を求める。
//! $`(y, k) \mapsto (f(y, x) \bmod x^k, f'(y, x) \bmod x^k)`$ の oracle を用いる。
//!
//! ### Relaxed multiplication
//!
//! $`f(x)`$ と $`g(x)`$ に対し、$`(f\cdot g)(x) = f(x)\cdot g(x)`$ を求める。ただし、$`[x^i]\, f(x)`$
//! や $`[x^i]\, g(x)`$ は、各 $`0\le j\lt i`$ における $`[x^j]\, (f\cdot g)(x)`$
//! を用いて計算されるとする。
//!
//! たとえば、与えられた $`\varphi`$ に対して $`\e^{\varphi}`$ を求めるのは、下記の微分方程式を解くことに帰着できる。
//! ```math
//! \e^{\varphi} = \int \varphi' \e^{\varphi}.
//! ```
//! $`f = \varphi'`$ かつ $`g = \e^{\varphi}`$ として relaxed multiplication を行い、 $`i\gt 0`$
//! に対して $`[x^i]\, g(x) = i^{-1}\cdot [x^{i-1}]\, (f\cdot g)(x)`$ とすればよい。
//!
//! $`\mathrm C_n \mathrm H_{2n+1} \mathrm{OH}`$ の形で表されるアルコールの立体異性体の個数は
//! ```math
//! s(z) = 1 + z\cdot \frac{s(z)^3 + 2s(z^3)}3
//! ```
//! として $`[z^n]\, s(z)`$ として表せることが Pólya によって示されている。$`[z^0]\, s(z)^3 = 1`$
//! であり、$`i\gt 0`$ に対して
//! ```math
//! \begin{aligned}
//! [z^i]\, s(z) &= [z^{i-1}] \left(3^{-1}\cdot s(z)^3 + 2\cdot 3^{-1}\cdot s(z^3)\right) \\
//! &= 3^{-1}\cdot [z^{i-1}]\, s(z)^3 + 2\cdot 3^{-1}\cdot [z^{i-1}]\, s(z^3) \\
//! &= 3^{-1}\cdot [z^{i-1}]\, s(z)^3 + 2\cdot 3^{-1}\cdot [z^{(i-1)/3}]\, s(z) \\
//! \end{aligned}
//! ```
//! なので…？
//!
//! ```math
//! f(z) = z\cdot\left(1+f{\left(\frac z{1+z}\right)} - z^4 f'(z)^2\right)
//! ```
//! が $`O(n\log(n)^3\log(\log(n)))`$ 時間で解けるらしい。
//! ```math
//! f(z) = z + f(z^2+z^3)
//! ```
//! が $`\tilde O(n)`$ 時間？で解けるらしい。
//! ```math
//! f(z) = z + f(z f(z) + z^2 f'(z)) + z^4\exp(z f''(z))
//! ```
//! が $`O(n^{3/2}\log(n)^{5/2}\log(\log(n)))`$ 時間で解けるらしい。
//!
//! ### Power projection
//!
//! ### Composition
//!
//! ### Reversion
//!
//! ### Transposition principle
//!
//! ### General-order derivative equation
//!
//! ### General functional equation
//!
//! ### Lagrange inversion theorem
//!
//! ### Log-exp
//!
//! $`\log(1-f(x))`$ は下記として定義される（再掲）。
//! ```math
//! \log(1-f(x)) = -\sum_{n=1}^{\infty} \frac{f(x)^n}n.
//! ```
//!
//! $`\prod\varphi = \exp(\log(\prod\varphi)) = \exp(\sum \log(\varphi))`$
//! を用いて式変形できる場合がある。
//!
//! たとえば次のような例がある。
//! ```math
//! \begin{aligned}
//! &\phantom{{}={}} \prod_{i=1}^{n} {(1+x^i+x^{2i}+\dots+x^{i\cdot a_i})} \\
//! &= \prod_{i=1}^n {\left(\frac1{1-x^i}-\frac{x^{i\cdot(a_i+1)}}{1-x^i}\right)} \\
//! &= \prod_{i=1}^n {\frac{1-x^{i\cdot(a_i+1)}}{1-x^i}} \\
//! &= {\exp}{\left({\log}{\left(\prod_{i=1}^n {\frac{1-x^{i\cdot(a_i+1)}}{1-x^i}}\right)}\right)} \\
//! &= {\exp}{\left(\sum_{i=1}^n {{\log}{\left(\frac{1-x^{i\cdot(a_i+1)}}{1-x^i}\right)}}\right)} \\
//! &= {\exp}{\left(\sum_{i=1}^n {{\log}{\left(\frac{1-x^{i\cdot(a_i+1)}}{1-x^i}\right)}}\right)} \\
//! &= {\exp}{\left(\sum_{i=1}^n {{\log}{\left(1-x^{i\cdot(a_i+1)}\right)} - \sum_{i=1}^n {{\log}{\left({1-x^i}\right)}}}\right)} \\
//! &= {\exp}{\left(\sum_{i=1}^n {\left(-\sum_{j=1}^{\infty} \frac{x^{i\cdot(a_i+1)\cdot j}}j\right)} - \sum_{i=1}^n {\left(-\sum_{j=1}^{\infty} \frac{x^{i\cdot j}}j\right)}\right)} \\
//! &= {\exp}{\left(-\sum_{i=1}^n {\left(\sum_{j=1}^{\infty} \frac{x^{i\cdot(a_i+1)\cdot j}}j\right)} + \sum_{i=1}^n {\left(\sum_{j=1}^{\infty} \frac{x^{i\cdot j}}j\right)}\right)} \\
//! &= {\exp}{\left(\sum_{i=1}^n {\left(\sum_{j=1}^{\infty} \frac{x^{i\cdot j}}j\right)} -\sum_{i=1}^n {\left(\sum_{j=1}^{\infty} \frac{x^{i\cdot(a_i+1)\cdot j}}j\right)}\right)}, \\
//! &\phantom{{}={}} [x^n]\, {\exp}{\left(\sum_{i=1}^n {\left(\sum_{j=1}^{\infty} \frac{x^{i\cdot j}}j\right)} -\sum_{i=1}^n {\left(\sum_{j=1}^{\infty} \frac{x^{i\cdot(a_i+1)\cdot j}}j\right)}\right)} \\
//! &= [x^n]\, {\exp}{\left(\sum_{i=1}^n {\left(\sum_{j=1}^{\floor{n/i}} \frac{x^{i\cdot j}}j\right)} -\sum_{i=1}^n {\left(\sum_{j=1}^{\floor{n/i}} \frac{x^{i\cdot(a_i+1)\cdot j}}j\right)}\right)}.
//! \end{aligned}
//! ```
//! 調和級数に関する計算量解析により、$`\exp`$ の引数は $`O(n\log(n))`$ 時間で計算できる。See
//! also [FPS-24 N](https://atcoder.jp/contests/fps-24/tasks/fps_24_n).
//!
//! ### Sum of powers
//!
//! 与えられた $`a = (a_0, a_1, \dots, a_{m-1})`$ に対し、各 $`i\ge 0`$ について
//! $`\sum_{j=0}^{m-1} a_j^i`$ を求めたいとする。
//!
//! ```math
//! \begin{aligned}
//! &\phantom{{}={}} -\frac{\d}{\d x}{\log}{\left(\prod_{j=0}^{m-1} {(1-a_j x)}\right)} \\
//! &= -\frac{\d}{\d x} \sum_{j=0}^{m-1} {\log(1-a_j x)} \\
//! &= -\sum_{j=0}^{m-1} {\frac{\d}{\d x} \log(1-a_j x)} \\
//! &= -\sum_{j=0}^{m-1} {\frac{(1-a_j x)'}{1-a_j x}} \\
//! &= -\sum_{j=0}^{m-1} {-\frac{a_j}{1-a_j x}} \\
//! &= \sum_{j=0}^{m-1} {\frac{a_j}{1-a_j x}} \\
//! &= \sum_{j=0}^{m-1} \sum_{i=0}^{\infty} a_j^{i+1} x^i \\
//! &= \sum_{i=0}^{\infty} \sum_{j=0}^{m-1} a_j^{i+1} x^i \\
//! \end{aligned}
//! ```
//! より、
//! ```math
//! [x^n]\, \left(-\frac{\d}{\d x}{\log}{\left(\prod_{j=0}^{m-1} {(1-a_j x)}\right)}\right)
//! = \sum_{j=0}^{m-1} a_j^{i+1}
//! ```
//! や
//! ```math
//! \sum_{i=0}^{\infty} \sum_{j=0}^{m-1} a_j^i x^i
//! = m - x \cdot \left(\frac{\d}{\d x}{\log}{\left(\prod_{j=0}^{m-1} {(1-a_j x)}\right)}\right)
//! ```
//! が成り立つ。
//!
//! あるいは、
//! ```math
//! \frac{P_1(x)}{Q_1(x)} + \frac{P_2(x)}{Q_2(x)}
//! = \frac{(P_1\cdot Q_2 + P_2\cdot Q_1)(x)}{(Q_1\cdot Q_2)(x)}
//! ```
//! より、各 $`i`$ に対して $`P_i(x) = 1`$ かつ $`Q_i(x) = 1-a_i x`$
//! として（適切に分割統治を用いて）総和を求めることで、対応する有理式を $`O(n\log(n)^2)`$
//! 時間で求めることもできる。
//!
//! ## See also
//!
//! - Taylor shift ([ABC 215 G](https://atcoder.jp/contests/abc215/editorial/2529))
