//! 動的計画法。
//!
//! ## Notations
//!
//! $`\Lambda_n = \{0, 1, \dots, n-1\}`$ とする。
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
//!
//! ## See also
//!
//! - <https://rsk0315.hatenablog.com/entry/2023/09/10/225138>
