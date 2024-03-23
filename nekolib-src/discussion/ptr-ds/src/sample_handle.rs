//! 例：handle。
//!
//! ## Samples
//!
//! 以下のような API を提供するクラスを考える。
//!
//! - $`S \gets \varnothing`$ で初期化。
//! - $`i = |S|`$ として $`S \gets S\cup (i, i)`$ で更新。
//! - $`(i, j)`$ を受け取る。$`(i, j')\in S`$ として $`S \gets (S\smallsetminus (i, j'))\cup (i, j)`$ で更新。
//! - $`i`$ を受け取る。$`\angled{i, f(i), f(f(i)), \dots}`$ を返す。
//!     - ただし、$`{}^{\exists!}j.\: (i, j)\in S`$ のとき $`f(i) = j`$ とする。
//!
//! 当然 `Vec<usize>` で実現できるが、ここではポインタを扱う練習として、$`i`$ 番目のノードから
//! $`j`$ 番目のノードに対するポインタを持つような実装を考える。
//!
//! ```
//! use std::ptr::NonNull;
//!
//! // --- implementations ---
//! pub struct Node {
//!     val: usize,
//!     mapsto: NonNull<Node>,
//! }
//! #[derive(Clone, Copy)]
//! pub struct Handle {
//!     node: NonNull<Node>,
//! }
//! pub struct Graph {
//!     nodes: Vec<NonNull<Node>>,
//! }
//!
//! impl Node {
//!     pub fn new(val: usize) -> NonNull<Self> {
//!         let node = Box::new(Node { val, mapsto: NonNull::dangling() });
//!         let ptr = NonNull::from(Box::leak(node));
//!         unsafe { (*ptr.as_ptr()).mapsto = ptr };
//!         ptr
//!     }
//! }
//!
//! impl Handle {
//!     fn new(node: NonNull<Node>) -> Self { Self { node } }
//!     pub fn mapsto(&mut self, other: Self) {
//!         unsafe { (*self.node.as_ptr()).mapsto = other.node }
//!     }
//!     pub fn walk(&self) -> impl Iterator<Item = usize> {
//!         std::iter::successors(Some(self.node), |&ptr| unsafe {
//!             Some((*ptr.as_ptr()).mapsto)
//!         })
//!         .map(|ptr| unsafe { (*ptr.as_ptr()).val })
//!     }
//! }
//!
//! impl Graph {
//!     pub fn new() -> Self { Self { nodes: vec![] } }
//!     pub fn push(&mut self) -> Handle {
//!         let val = self.nodes.len();
//!         let ptr = Node::new(val);
//!         self.nodes.push(ptr);
//!         Handle::new(ptr)
//!     }
//! }
//!
//! impl Drop for Graph {
//!     fn drop(&mut self) {
//!         for ptr in self.nodes.drain(..) {
//!             unsafe { drop(Box::from_raw(ptr.as_ptr())) };
//!         }
//!     }
//! }
//!
//! // --- tests ---
//! let mut graph = Graph::new();
//! let mut x0 = graph.push();
//! let mut x1 = graph.push();
//! let mut x2 = graph.push();
//! let mut x3 = graph.push();
//! let mut x4 = graph.push();
//!
//! x0.mapsto(x1);
//! x1.mapsto(x3);
//! x2.mapsto(x1);
//! x3.mapsto(x1);
//! x4.mapsto(x2);
//!
//! // 0 -> 1 <> 3
//! //      ^
//! //      2 <- 4
//!
//! let a: Vec<_> = x0.walk().take(10).collect();
//! assert_eq!(a, [0, 1, 3, 1, 3, 1, 3, 1, 3, 1]);
//!
//! x3.mapsto(x4);
//! x4.mapsto(x2);
//!
//! // 0 -> 1 -> 3
//! //      ^    v
//! //      2 <- 4
//!
//! let a: Vec<_> = x0.walk().take(10).collect();
//! assert_eq!(a, [0, 1, 3, 4, 2, 1, 3, 4, 2, 1]);
//!
//! x0.mapsto(x0); // self loop
//!
//! let a: Vec<_> = x0.walk().take(10).collect();
//! assert_eq!(a, [0; 10]);
//! ```
//!
//! ## Notes
//!
//! 「node を作成し、handle を返す」「handle を渡し、構造を更新する」が基本的な方針になると思われる。
//! より複雑な例においては「更新の際に、別の handle（が持っている pointer が指す node）を無効化する」
//! ということも起きうるので、注意する必要がある。
//!
//! これが問題なく書けるのであれば、Fibonacci heap なども書けそうな気がするが果たして？
//!
//! おそらくは次のような構造で書けると思う (cf. [CS166 (1186), Fibonacci Heaps](https://web.stanford.edu/class/archive/cs/cs166/cs166.1186/lectures/09/Slides09.pdf#page=208))。
//!
//! ```
//! use std::ptr::NonNull;
//!
//! pub struct FibHeap<T: Ord> {
//!     roots: Vec<FibHeapNode<T>>,
//!     max: Option<T>,
//! }
//! struct FibHeapNode<T> {
//!     val: T,
//!     parent: Option<NonNull<FibHeapNode<T>>>,
//!     neighbor: (NonNull<FibHeapNode<T>>, NonNull<FibHeapNode<T>>),
//!     any_child: Option<NonNull<FibHeapNode<T>>>,
//!     order: usize,
//!     cut: bool,
//! }
//! pub struct FibHeapHandle<T> {
//!     node: NonNull<FibHeapNode<T>>,
//! }
//!
//! impl<T: Ord> FibHeap<T> {
//!     pub fn new() -> Self { todo!() }
//!     pub fn push(&mut self, elt: T) -> FibHeapHandle<T> { todo!() }
//!     pub fn pop(&mut self) -> Option<T> { todo!() }
//!     pub fn meld(&mut self, other: Self) { todo!() }
//! }
//! impl<T: Ord> FibHeapHandle<T> {
//!     pub unsafe fn urge(&mut self, elt: T) -> bool { todo!() }
//! }
//! ```
//!
//! 優先度を変えるインターフェースとしては [`std::collections::binary_heap::PeekMut`]
//! のようなものも存在しているが、Fibonacci heap を使いたい状況では、最大値に限らない要素を（検索含め）
//! 償却定数時間で行える必要があり、handle 経由で行う必要があると思われる。当然、deterministic
//! に行いたいので hash map は使いたくない。
//!
//!
