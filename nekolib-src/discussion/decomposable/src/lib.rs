//! 分解可能なクエリ。
//!
//! ## Preliminaries
//!
//! 集合 $`S\subseteq U`$ と要素 $`x\in U`$ に関する問題 $`Q_S(x)`$ を考える。
//! $`S\cup T`$ に対する問題を解くにあたって、何らかの二項演算 $`\square`$ が存在して
//! ```math
//! Q_{S\cup T}(x) = Q_S(x) \mathop{\square} Q_T(x)
//! ```
//! と書けるとき、問題 $`Q`$ は *decomposable* であるという。
//!
//! ## Theorems
//!
//! 二種類のクエリを考える。
//!
//! - $`x`$ が与えられ、$`S \xgets{\cup} \{x\}`$ で更新する。
//! - $`x`$ が与えられ、$`Q_S(x)`$ を出力する。
//!
//! $`Q_S(x)`$ が $`\angled{f(|S|), g(|S|)}`$ 時間で解けるとき、$`n`$ 個のクエリを $`O((f(n) + n\cdot g(n))\log(n))`$
//! 時間でオンライン処理できる。
//! ただし、$`f(n)\in n^{\Omega(1)}`$, $`g(n)\in n^{o(1)}`$ とし、$`\square`$ の計算も $`O(g(n))`$ 時間でできるとする。
//!
//! たとえば、$`Q_S(x)`$ が $`\angled{O(|S|\log(|S|)), O(\log(|S|))}`$ 時間で解けるなら、$`n`$ 個のクエリを $`O(n\log(n)^2)`$ 時間で処理できる。
//!
//! ## Ideas
//!
//! $`i`$ 番目の更新クエリの値を $`x_i`$ として、次のようなセグ木をイメージする。
//!
//! - $`i`$ 番目の葉に $`\{x_i\}`$ を持つ。
//! - 子が $`S_L`$, $`S_R`$ を持っている頂点は $`S_L\cup S_R`$ を持つ。
//! - $`S`$ を持つ頂点では $`Q_S`$ のためのデータ構造を保持する。
//!
//! 更新クエリを $`k`$ 個処理している状態では、$`[0\lldot k)`$ をカバーする区間にあるデータ構造を用いて各々を処理し、$`\square`$ で合成する。
//!
//! 実際にはこのセグ木の各ノードは更新せず、また各ノードを陽に持っておく必要もない。
//! 次の擬似コードに示すようにして、高々 $`\log_2(n)`$ 個のデータ構造を保持しておけばよい。
//!
//! `meld` の際には、`other` の各要素を直接 `self` に追加してもよいし、一旦 `.into_iter()`
//! のようなことをしてから再構築してもよい。
//!
//! ```
//! enum Query<T> {
//!     Union(T),
//!     Search(T),
//! }
//! use Query::*;
//! let qs = Vec::<Query<()>>::new();
//!
//! struct Ds<T> {
//!     // ...
//! #   _phd: std::marker::PhantomData<T>,
//! }
//! impl<T> Ds<T> {
//!     pub fn singleton(elt: T) -> Self {
//!         // ...
//! #       Self { _phd: std::marker::PhantomData }
//!     }
//!     pub fn search<U>(&self, elt: &T) -> U {
//!         // ...
//! #       unimplemented!()
//!     }
//!     pub fn meld(&mut self, other: Self) {
//!         // ...
//!     }
//! }
//! # let id = ();
//! # fn compose(_: (), _: ()) {}
//!
//! let mut clx = vec![];
//! let mut count = 0_usize;
//! for q in qs {
//!     match q {
//!         Union(x) => {
//!             count += 1;
//!             clx.push(Ds::singleton(x));
//!             for _ in 0..count.trailing_zeros() {
//!                 let tmp = clx.pop().unwrap();
//!                 clx.last_mut().unwrap().meld(tmp);
//!             }
//!         }
//!         Search(x) => {
//!             let _ = clx.iter().map(|si| si.search(&x)).fold(id, compose);
//!         }
//!     }
//! }
//! ```
//!
//! ## References
//!
//! - <https://erikdemaine.org/papers/Retroactive_TALG/paper.pdf>
//!     - 命名を参考にした。
//! - <https://atcoder.jp/contests/abc244/submissions/30254752>
//!     - 実装を参考にした。
