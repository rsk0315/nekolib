//! B-tree のノード。
//!
//! 理想的には下記の概念を扱う。
//!
//! ```compile_fail
//! struct BTreeMap<K, V> {
//!     height: usize,
//!     root: Option<Box<Node<K, V, height>>>
//! }
//!
//! struct Node<K, V, height: usize> {
//!     keys: [K; 2 * B - 1],
//!     vals: [V; 2 * B - 1],
//!     edges: [if height > 0 { Box<Node<K, V, height - 1>> } else { () }; 2 * B],
//!     parent: Option<(NonNull<Node<K, V, height + 1>>, u16)>,
//!     len: u16,
//! }
//! ```
//!
//! Rust には依存型と多相再帰がないため、`unsafe` を用いてこれを実現する。
//!
//! このモジュールにおいては、B-tree の不変条件のうち、いくつかのみを管理する。
//!
//! - 高さが一様である。すなわち、根から葉までのパスは、いずれも同じ長さを持つ。
//! - 長さが $`n`$ のノードは $`n`$ 個の key-value pair を持ち、$`n + 1`$ 本の辺を持つ。
//!     - 空のノードであっても少なくとも一本の辺を持つ。
//!     - 葉ノードにおける辺は、値を持たないため、位置の管理にのみ用いる。
//!     - 内部ノードにおける辺は、位置の管理および子ノードでのポインタを持つ。
//!
//! # Explanations
//!
//! [`LeafNode`] および [`InternalNode`] では生成のメソッドのみを定義し、具体的な処理は
//! [`NodeRef`] を介して行うようになっている。[`NodeRef`] のメソッドには、mutability
//! の変換や生ポインタの取得、[`Handle`] の取得などを行う boilerplate が多々ある。
//!
//! ## Root updatings
//!
//! まず、[`marker::Owned`] に記述がある通り、[`NodeRef<Owned, ..>`]
//! は根ノードへの参照を表す。根ノードに関する重要なメソッドとして、[`push_internal_level`],
//! [`pop_internal_level`] がある。
//!
//! [`push_internal_level`] は次の処理を行う。
//!
//! - 既存の [`NodeRef<Owned, ..>`] が指しているノードのみを子に持つノードを作成する。
//! - 新しく作った根ノードを参照するように更新する。
//! - 新しい根ノードへの参照 [`NodeRef<Mut<'_>, ..>`] を返す。
//!
//! [`pop_internal_level`] は逆で、次の処理を行う。
//! ただし、既存の根ノードは子をちょうど一つ持つものとする。
//!
//! - 既存の [`NodeRef<Owned, ..>`] の最左の子を新たな根ノードとなるように更新する。
//! - 既存の根ノードを解放する。
//!
//! [`NodeRef<Owned, ..>`]: NodeRef
//! [`NodeRef<Mut<'_>, ..>`]: NodeRef
//! [`push_internal_level`]: NodeRef::push_internal_level
//! [`pop_internal_level`]: NodeRef::pop_internal_level
//!
//! ## Insertions
//!
//! B-tree に対する挿入は葉ノードから再帰的に行うのが基本であり、公開する部分としては
//! [`Handle::<NodeRef<Mut<'_>, .., Leaf>, Edge>::insert_recursing(..)`]
//! となる。根ノードの更新が必要になる場合があるものの、receiver が
//! [`Handle<Mut<'_>, ..>`] であり [`push_internal_level`]
//! を直接呼べないため工夫が必要になる。
//!
//! 呼び出し元から [`insert_recursing`] に [`SplitResult`] を受け取る callback
//! を渡しておき、ノードの分割が（既存の）根ノードでも起きた場合は、その
//! [`SplitResult`] に対して callback を呼び出すようになっている。callback
//! 内では [`push_internal_level`] を呼ぶのが想定されていると思われる。
//!
//! 概ね、次のような使い方となる。
//!
//! ```ignore
//! struct BTreeMap<'a, K, V> {
//!     root: Option<NodeRef<marker::Owned, K, V, marker::LeafOrInternal>>,
//!     len: usize,
//!     _marker: PhantomData<&'a mut (K, V)>,
//! }
//! let mut map = BTreeMap { root: Some(NodeRef::new()), len: 0, _marker: PhantomData };
//!
//! for i in (0..CAPACITY + 1).rev() {
//!     let (map, mut dormant_map) = DormantMutRef::new(&mut map);
//!     let mut_root = map.root.as_mut().unwrap().borrow_mut();
//!     let mut handle = mut_root.first_leaf_edge();
//!     let (key, val) = (i, ());
//!     handle.insert_recursing(key, val, |ins| {
//!         let SplitResult { left: _, kv: (k, v), right } = ins;
//!         let mut map = unsafe { dormant_map.reborrow() };
//!         let root = map.root.as_mut().unwrap();
//!         root.push_internal_level().push(k, v, right)
//!     });
//!     let mut map = unsafe { dormant_map.awaken() };
//!     map.len += 1;
//! }
//! ```
//!
//! [`DormantMutRef`] を使うことで、下記のエラーを回避している。
//!
//! ```text
//! error[E0500]: closure requires unique access to `map.root` but it is already borrowed
//! ```
//!
//! また、[`insert_recursing`] を呼んだ後、`dormant_map.awaken()`
//! から取得した参照ではなく `DormantMutRef::new(..)` から取得した参照を使うと、Stacked
//! Borrows のルールに違反することになる。
//!
//! [`DormantMutRef`]: super::borrow::DormantMutRef
//! [`Handle<Mut<'_>, ..>`]: Handle
//! [`insert_recursing`]: Handle::insert_recursing
//! [`Handle::<NodeRef<Mut<'_>, .., Leaf>, Edge>::insert_recursing(..)`]: Handle::insert_recursing

use std::{
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::{self, NonNull},
    slice::SliceIndex,
};

/// 各定数の値を定めるためのパラメータ。
const B: usize = 6;
/// 各ノードの最大要素数。
pub const CAPACITY: usize = 2 * B - 1;
/// ノードの分割後に保証されるノードの要素数。
pub const MIN_LEN_AFTER_SPLIT: usize = B - 1;

const KV_IDX_CENTER: usize = B - 1;
const EDGE_IDX_LEFT_OF_CENTER: usize = B - 1;
const EDGE_IDX_RIGHT_OF_CENTER: usize = B;

/// 葉ノードの表現。
///
/// 内部ノード ([`InternalNode`]) の表現の一部としても用いる。
struct LeafNode<K, V> {
    /// `K` および `V` について covariant にしたい。
    parent: Option<NonNull<InternalNode<K, V>>>,

    /// 親ノードの `edges` における添字を表す。
    /// すなわち、`*node.parent.edges[node.parent_idx]` が `node`
    /// と同じものを指すようにする。`parent` が `Some(_)` のときのみ、初期化が保証される。
    parent_idx: MaybeUninit<u16>,

    /// 管理している key-value pair の個数。
    len: u16,

    /// 管理している key の列。先頭から `len` 個の要素のみが初期化済み・有効である。
    keys: [MaybeUninit<K>; CAPACITY],
    /// 管理している value の列。不変条件は `keys` と同じ。
    vals: [MaybeUninit<V>; CAPACITY],
}

impl<K, V> LeafNode<K, V> {
    /// [`LeafNode`] を in-place に初期化する。
    unsafe fn init(this: *mut Self) {
        unsafe {
            ptr::addr_of_mut!((*this).parent).write(None);
            ptr::addr_of_mut!((*this).len).write(0);
        }
    }
    /// [`Box`] に入った [`LeafNode`] を作成する。
    fn new() -> Box<Self> {
        unsafe {
            let mut leaf = MaybeUninit::<LeafNode<K, V>>::uninit();
            LeafNode::init(leaf.as_mut_ptr());
            Box::new(leaf.assume_init())
        }
    }
}

/// 内部ノードの表現。
///
/// 未初期化の key-value pair が drop されるのを防ぐため、[`BoxedNode`]
/// の内部に隠した状態で扱う。これは [`LeafNode`] についても同様である。
///
/// `data` を先頭で定義して `#[repr(C)]` を適用しているため、[`InternalNode`]
/// へのポインタから [`LeafNode`] へのポインタにキャストしたとき、`data`
/// を指すポインタとして使うことができる。
/// すなわち、実際にポインタが指しているのが葉ノード・内部ノードのどちらを指すかを気にすることなく、葉ノードと共通で持っているフィールドを扱うことができる。
///
/// ```ignore
/// let internal = todo!();
/// let as_leaf = internal as *mut LeafNode<K, V>;
/// assert_eq!(unsafe { (*as_leaf).len }, 0);
/// ```
#[repr(C)]
struct InternalNode<K, V> {
    /// 葉ノードと共通しているフィールドを [`LeafNode`] として持っている。
    data: LeafNode<K, V>,

    /// 子ノードへのポインタの列。先頭から `len + 1`
    /// 個の要素のみが初期化済み・有効である。[`marker::Dying`] として borrow
    /// された後は、それらの中にも dangling なものが含まれうる。
    edges: [MaybeUninit<BoxedNode<K, V>>; 2 * B],
}

impl<K, V> InternalNode<K, V> {
    /// [`Box`] に入った [`InternalNode`] を作成する。
    unsafe fn new() -> Box<Self> {
        unsafe {
            let mut node = MaybeUninit::<InternalNode<K, V>>::uninit();
            LeafNode::init(ptr::addr_of_mut!((*node.as_mut_ptr()).data));
            Box::new(node.assume_init())
        }
    }
}

/// ノードへの null でないポインタ。
///
/// [`LeafNode<K, V>`] または [`InternalNode<K, V>`]
/// のいずれかを所有しているポインタであるが、どちらを持っているかの情報は持っていない。
/// 別々の型にせず destructor を持たない一つの理由として、そのことが挙げられる。
type BoxedNode<K, V> = NonNull<LeafNode<K, V>>;

/// ノードへの参照。
///
/// パラメータに応じて、どのように振る舞うかが変わる。
///
/// `BorrowType` は、借用の種類や lifetime を管理する。
///
/// - [`Immut<'a>`] のとき、`&'a Node` のように振る舞う。
/// - [`ValMut<'a>`] のとき、木の構造と key に関しては `&'a Node`
/// のように振る舞いつつ、同じ木に属する value たちに対する可変参照は共存できる。
/// - [`Mut<'a>`] のとき、`&'a mut Node` のように振る舞う。insert
/// のメソッドにより、value たちに対する複数個の可変参照が共存しうる。
/// - [`Owned`] のとき、`Box<Node>` のように振る舞うが、destructor
/// を持たないため手動で解放する必要がある。
/// - [`Dying`] のとき、`Box<Node>` にように振る舞うが、木を破棄するためのメソッドを持つ。
/// `unsafe` がついていない場合でも、不適切に呼ぶと未定義動作になりうる。
///
/// `K` および `V` は、ノードで管理している key-value pair の型を表す。
///
/// `Type` は、指しているノードの種類を表す。`NodeRef` の外から扱う場合は
/// `NodeType` と呼称するものとする。
///
/// - [`Leaf`] のとき、葉ノードを指す。
/// - [`Internal`] のとき、内部ノードを指す。
/// - [`LeafOrInternal`] のとき、どちらも指しうる。
///
/// たとえば、あるノードの親を返す関数や子を返す関数の返り値を考える。前者の `NodeType`
/// は [`Internal`] だが、後者は [`LeafOrInternal`] となる。
///
/// [`Immut<'a>`]: marker/struct.Immut.html
/// [`ValMut<'a>`]: marker/struct.ValMut.html
/// [`Mut<'a>`]: marker/struct.Mut.html
/// [`Owned`]: marker/enum.Owned.html
/// [`Dying`]: marker/enum.Dying.html
/// [`Leaf`]: marker/enum.Leaf.html
/// [`Internal`]: marker/enum.Internal.html
/// [`LeafOrInternal`]: marker/enum.LeafOrInternal.html
///
/// TODO
pub struct NodeRef<BorrowType, K, V, Type> {
    /// そのノードの高さ。`Type` では記述できず、`node`
    /// もその情報は持っていない。根ノードの高さのみを持ち、探索の過程で差分から管理する。
    /// `Type` が `Leaf` のときは `0`、`Internal` のときは `1` 以上である必要がある。
    height: usize,

    /// 葉ノードまたは内部ノードへのポインタ。内部ノードを指す場合は、`data` の部分を指す。
    node: NonNull<LeafNode<K, V>>,
    _marker: PhantomData<(BorrowType, Type)>,
}

/// 所有している木の根ノードへの参照。
///
/// destructor を持っておらず、手動で解放する必要がある。
pub type Root<K, V> = NodeRef<marker::Owned, K, V, marker::LeafOrInternal>;

impl<'a, K: 'a, V: 'a, Type> Copy for NodeRef<marker::Immut<'a>, K, V, Type> {}
impl<'a, K: 'a, V: 'a, Type> Clone for NodeRef<marker::Immut<'a>, K, V, Type> {
    fn clone(&self) -> Self { *self }
}

unsafe impl<BorrowType, K: Sync, V: Sync, Type> Sync
    for NodeRef<BorrowType, K, V, Type>
{
}

unsafe impl<K: Sync, V: Sync, Type> Send
    for NodeRef<marker::Immut<'_>, K, V, Type>
{
}
unsafe impl<K: Send, V: Send, Type> Send
    for NodeRef<marker::Mut<'_>, K, V, Type>
{
}
unsafe impl<K: Send, V: Send, Type> Send
    for NodeRef<marker::ValMut<'_>, K, V, Type>
{
}
unsafe impl<K: Send, V: Send, Type> Send
    for NodeRef<marker::Owned, K, V, Type>
{
}
unsafe impl<K: Send, V: Send, Type> Send
    for NodeRef<marker::Dying, K, V, Type>
{
}

impl<K, V> NodeRef<marker::Owned, K, V, marker::Leaf> {
    /// 空な葉ノードを作り、それへの参照を返す。
    pub fn new_leaf() -> Self { Self::from_new_leaf(LeafNode::new()) }

    /// `Box<Node>` を leak し、それへの参照を返す。
    fn from_new_leaf(leaf: Box<LeafNode<K, V>>) -> Self {
        NodeRef {
            height: 0,
            node: NonNull::from(Box::leak(leaf)),
            _marker: PhantomData,
        }
    }
}

impl<K, V> NodeRef<marker::Owned, K, V, marker::Internal> {
    /// 子として `child` のみを持つ内部ノードを作り、それへの参照を返す。
    fn new_internal(child: Root<K, V>) -> Self {
        let mut new_node = unsafe { InternalNode::new() };
        new_node.edges[0].write(child.node);
        unsafe { NodeRef::from_new_internal(new_node, child.height + 1) }
    }

    /// `Box<Node>` を leak し、それへの参照を返す。
    ///
    /// # Safety
    /// `height > 0` である必要がある。
    unsafe fn from_new_internal(
        internal: Box<InternalNode<K, V>>,
        height: usize,
    ) -> Self {
        debug_assert!(height > 0);
        let node = NonNull::from(Box::leak(internal)).cast();
        let mut this = NodeRef { height, node, _marker: PhantomData };
        this.borrow_mut().correct_all_childrens_parent_links();
        this
    }
}

impl<BorrowType, K, V> NodeRef<BorrowType, K, V, marker::Internal> {
    /// 高さが既知の内部ノードへの参照を返す。
    fn from_internal(node: NonNull<InternalNode<K, V>>, height: usize) -> Self {
        debug_assert!(height > 0);
        NodeRef { height, node: node.cast(), _marker: PhantomData }
    }

    /// 内部ノードの生ポインタを返す。
    ///
    /// このノードへの他の参照を無効化するのを避けるため、生ポインタで扱う。
    fn as_internal_ptr(this: &Self) -> *mut InternalNode<K, V> {
        this.node.as_ptr() as *mut InternalNode<K, V>
    }
}

impl<'a, K, V> NodeRef<marker::Mut<'a>, K, V, marker::Internal> {
    /// 内部ノードへの排他的なアクセスを借用する。
    fn as_internal_mut(&mut self) -> &mut InternalNode<K, V> {
        let ptr = Self::as_internal_ptr(self);
        unsafe { &mut *ptr }
    }
}

impl<BorrowType, K, V, Type> NodeRef<BorrowType, K, V, Type> {
    /// ノードが管理する値の個数を返す。辺は `len() + 1` 本となる。
    ///
    /// この関数により、可変参照が無効化されることに留意する必要がある。
    ///
    /// 内部実装に関して、`len` のフィールドのみにアクセスすることで、`ValMut<'_>`
    /// の参照を無効化しないように気をつけている。
    pub fn len(&self) -> usize {
        unsafe { usize::from((*Self::as_leaf_ptr(self)).len) }
    }

    /// 参照しているノードの高さを返す。
    pub fn height(&self) -> usize { self.height }

    /// 参照しているノードへの不変参照を返す。
    pub fn reborrow(&self) -> NodeRef<marker::Immut<'_>, K, V, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }

    /// 葉ノードへの生ポインタを返す。
    ///
    /// 参照しているノードが内部ノードだった場合は、返り値は `data`
    /// フィールドを指すポインタである。
    ///
    /// このノードへの他の参照を無効化するのを避けるため、生ポインタで扱う。
    fn as_leaf_ptr(this: &Self) -> *mut LeafNode<K, V> { this.node.as_ptr() }
}

impl<BorrowType: marker::BorrowType, K, V, Type>
    NodeRef<BorrowType, K, V, Type>
{
    /// 参照しているノードの親を返す。
    ///
    /// 親が存在する場合は `Ok(handle)` を返し、そうでない場合は `Err(self)`
    /// を返す。ここで、`handle` は親ノードの `self` を指す辺である。
    ///
    /// メソッドの命名は、根ノードを上に描くことを前提としている。
    ///
    /// `edge.descend().ascend().unwrap()` と `node.ascend().unwrap().descend()`
    /// は、unwrap が成功する前提では、何もしないのと同じになる。
    pub fn ascend(
        self,
    ) -> Result<
        Handle<NodeRef<BorrowType, K, V, marker::Internal>, marker::Edge>,
        Self,
    > {
        // const { assert!(BorrowType::TRAVERSAL_PERMIT) }

        let leaf_ptr: *const _ = Self::as_leaf_ptr(&self);
        unsafe { (*leaf_ptr).parent }
            .as_ref()
            .map(|parent| Handle {
                node: NodeRef::from_internal(*parent, self.height + 1),
                idx: unsafe {
                    usize::from((*leaf_ptr).parent_idx.assume_init())
                },
                _marker: PhantomData,
            })
            .ok_or(self)
    }

    /// 最初の辺を返す。
    pub fn first_edge(self) -> Handle<Self, marker::Edge> {
        unsafe { Handle::new_edge(self, 0) }
    }
    /// 最後の辺を返す。
    pub fn last_edge(self) -> Handle<Self, marker::Edge> {
        let len = self.len();
        unsafe { Handle::new_edge(self, len) }
    }
    /// 最初の key-value pair を返す。
    ///
    /// ## Panics
    /// `self` が空のとき panic する。
    pub fn first_kv(self) -> Handle<Self, marker::KV> {
        let len = self.len();
        assert!(len > 0);
        unsafe { Handle::new_kv(self, 0) }
    }
    /// 最後の key-value pair を返す。
    ///
    /// ## Panics
    /// `self` が空のとき panic する。
    pub fn last_kv(self) -> Handle<Self, marker::KV> {
        let len = self.len();
        assert!(len > 0);
        unsafe { Handle::new_kv(self, len - 1) }
    }
}

impl<BorrowType, K, V, Type> NodeRef<BorrowType, K, V, Type> {
    /// 等価判定を行う。
    ///
    /// [`PartialEq`] の pub な実装になりうるものの、モジュールの中でのみ使用する。
    fn eq(&self, other: &Self) -> bool {
        let Self { node, height, .. } = self;
        if node.eq(&other.node) {
            debug_assert_eq!(*height, other.height);
            true
        } else {
            false
        }
    }
}

impl<'a, K: 'a, V: 'a, Type> NodeRef<marker::Immut<'a>, K, V, Type> {
    /// 不変な木のノードへの参照に対して、葉ノードの領域への参照を返す。
    ///
    /// `NodeRef<.., InternalNode>` に対しては `data` を返すということ。
    fn into_leaf(self) -> &'a LeafNode<K, V> {
        let ptr = Self::as_leaf_ptr(&self);
        unsafe { &*ptr }
    }

    /// ノードに保持されている key の列への view を借用する。
    pub fn keys(&self) -> &[K] {
        let leaf = self.into_leaf();
        unsafe {
            let slice = leaf.keys.get_unchecked(..usize::from(leaf.len));
            &*(slice as *const [_] as *const [K])
        }
    }
}

impl<K, V> NodeRef<marker::Dying, K, V, marker::LeafOrInternal> {
    /// [`ascend`] と同様に親ノードを返しつつ、引数のノードを解放する。
    ///
    /// [`ascend`]: #method.ascend
    ///
    /// # Safety
    /// 呼び出し後、引数のノードにアクセスしてはいけない。
    pub unsafe fn deallocate_and_ascend(
        self,
    ) -> Option<
        Handle<NodeRef<marker::Dying, K, V, marker::Internal>, marker::Edge>,
    > {
        let Self { height, node, .. } = self;
        let ret = self.ascend().ok();
        unsafe {
            if height > 0 {
                drop(Box::from_raw(node.cast::<InternalNode<K, V>>().as_ptr()));
            } else {
                drop(Box::from_raw(node.as_ptr()));
            }
        }
        ret
    }
}

impl<'a, K, V, Type> NodeRef<marker::Mut<'a>, K, V, Type> {
    /// 参照しているノードに対する新しい可変参照を返す。
    ///
    /// 可変参照は木のどこへでも移動でき、ここで返した参照は既存の参照を容易に無効化させうる。
    /// 無効化は、dangling や out-of-bounds や、Stacked Borrows
    /// のルールに基づくものを含む。
    ///
    /// `BorrowType` に `!TRAVERSAL_PERMIT` な `Mut<'_>`
    /// の亜種を追加することで、この `unsafe`-ty を回避することが可能？
    unsafe fn reborrow_mut(&mut self) -> NodeRef<marker::Mut<'_>, K, V, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }

    /// 葉ノードの領域への排他的なアクセスを借用する。
    fn as_leaf_mut(&mut self) -> &mut LeafNode<K, V> {
        let ptr = Self::as_leaf_ptr(self);
        unsafe { &mut *ptr }
    }

    /// 葉ノードの領域への排他的なアクセスを返す。
    fn into_leaf_mut(mut self) -> &'a mut LeafNode<K, V> {
        let ptr = Self::as_leaf_ptr(&mut self);
        unsafe { &mut *ptr }
    }

    /// 参照しているノードの lifetime を消去したコピーを返す。
    pub fn dormant(&self) -> NodeRef<marker::DormantMut, K, V, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<K, V, Type> NodeRef<marker::DormantMut, K, V, Type> {
    /// 元々持っていた排他的な借用を復帰させる。
    ///
    /// # Safety
    ///
    /// reborrow を終了させている必要がある。すなわち、`NodeRef<DormantMut>`
    /// およびそこから作った参照は、この呼び出し後に使ってはいけない。
    pub unsafe fn awaken<'a>(self) -> NodeRef<marker::Mut<'a>, K, V, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<K, V, Type> NodeRef<marker::Dying, K, V, Type> {
    /// 葉ノードの領域への排他的なアクセスを借用する。
    fn as_leaf_dying(&mut self) -> &mut LeafNode<K, V> {
        let ptr = Self::as_leaf_ptr(self);
        unsafe { &mut *ptr }
    }
}

impl<'a, K: 'a, V: 'a, Type> NodeRef<marker::Mut<'a>, K, V, Type> {
    /// key の列への排他的なアクセスを借用する。
    ///
    /// # Safety
    /// `index` は `0..CAPACITY` に収まる必要がある。
    unsafe fn key_area_mut<I, Output: ?Sized>(
        &mut self,
        index: I,
    ) -> &mut Output
    where
        I: SliceIndex<[MaybeUninit<K>], Output = Output>,
    {
        unsafe {
            self.as_leaf_mut().keys.as_mut_slice().get_unchecked_mut(index)
        }
    }

    /// value の列への排他的なアクセスを借用する。
    ///
    /// # Safety
    /// `index` は `0..CAPACITY` に収まる必要がある。
    unsafe fn val_area_mut<I, Output: ?Sized>(
        &mut self,
        index: I,
    ) -> &mut Output
    where
        I: SliceIndex<[MaybeUninit<V>], Output = Output>,
    {
        unsafe {
            self.as_leaf_mut().vals.as_mut_slice().get_unchecked_mut(index)
        }
    }
}

impl<'a, K: 'a, V: 'a> NodeRef<marker::Mut<'a>, K, V, marker::Internal> {
    /// 辺の列への排他的なアクセスを借用する。
    ///
    /// # Safety
    /// `index` は `0..CAPACITY + 1` に収まる必要がある。
    unsafe fn edge_area_mut<I, Output: ?Sized>(
        &mut self,
        index: I,
    ) -> &mut Output
    where
        I: SliceIndex<[MaybeUninit<BoxedNode<K, V>>], Output = Output>,
    {
        unsafe {
            self.as_internal_mut().edges.as_mut_slice().get_unchecked_mut(index)
        }
    }
}

impl<'a, K, V, Type> NodeRef<marker::ValMut<'a>, K, V, Type> {
    /// `idx` 番目の key-value pair への参照を返す。
    ///
    /// # Safety
    /// `idx + 1` 個以上の要素が初期化されている必要がある。
    unsafe fn into_key_val_mut_at(mut self, idx: usize) -> (&'a K, &'a mut V) {
        let leaf = Self::as_leaf_ptr(&mut self);
        let keys = unsafe { ptr::addr_of!((*leaf).keys) };
        let vals = unsafe { ptr::addr_of_mut!((*leaf).vals) };
        let keys: *const [_] = keys;
        let vals: *mut [_] = vals;
        // let key = unsafe { (&*keys.get_unchecked(idx)).assume_init_ref() };
        // let val =
        //     unsafe { (&mut *vals.get_unchecked_mut(idx)).assume_init_mut() };
        let key = unsafe { (*ptr::addr_of!((*keys)[idx])).assume_init_ref() };
        let val =
            unsafe { (*ptr::addr_of_mut!((*vals)[idx])).assume_init_mut() };
        (key, val)
    }
}

impl<'a, K: 'a, V: 'a, Type> NodeRef<marker::Mut<'a>, K, V, Type> {
    /// ノードの長さへの排他的なアクセスを借用する。
    pub fn len_mut(&mut self) -> &mut u16 { &mut self.as_leaf_mut().len }
}

impl<'a, K, V> NodeRef<marker::Mut<'a>, K, V, marker::Internal> {
    /// `range` の範囲内の親子間のリンクを修正する。
    ///
    /// # Safety
    /// `range` が返す値は、どれも有効な辺を表す必要がある。
    unsafe fn correct_childrens_parent_links<R: Iterator<Item = usize>>(
        &mut self,
        range: R,
    ) {
        for i in range {
            debug_assert!(i <= self.len());
            unsafe { Handle::new_edge(self.reborrow_mut(), i) }
                .correct_parent_link();
        }
    }

    /// 親子間のリンクを修正する。
    fn correct_all_childrens_parent_links(&mut self) {
        let len = self.len();
        unsafe { self.correct_childrens_parent_links(0..=len) };
    }
}

impl<'a, K: 'a, V: 'a> NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal> {
    /// 親へのリンクおよび添字を更新する。
    ///
    /// そのノードに対する他の参照の無効化は生じない。
    fn set_parent_link(
        &mut self,
        parent: NonNull<InternalNode<K, V>>,
        parent_idx: usize,
    ) {
        let leaf = Self::as_leaf_ptr(self);
        unsafe { (*leaf).parent = Some(parent) };
        unsafe { (*leaf).parent_idx.write(parent_idx as u16) };
    }
}

impl<K, V> NodeRef<marker::Owned, K, V, marker::LeafOrInternal> {
    /// 根ノードの親へのリンクを削除する。
    fn clear_parent_link(&mut self) {
        let mut root_node = self.borrow_mut();
        let leaf = root_node.as_leaf_mut();
        leaf.parent = None;
    }
}

impl<K, V> NodeRef<marker::Owned, K, V, marker::LeafOrInternal> {
    /// 新しく空な木を作り、それへの参照を返す。
    pub fn new() -> Self { NodeRef::new_leaf().forget_type() }

    pub fn push_internal_level(
        &mut self,
    ) -> NodeRef<marker::Mut<'_>, K, V, marker::Internal> {
        super::mem::take_mut(self, |old_root| {
            NodeRef::new_internal(old_root).forget_type()
        });
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }

    pub fn pop_internal_level(&mut self) {
        assert!(self.height > 0);

        let top = self.node;

        let internal_self =
            unsafe { self.borrow_mut().cast_to_internal_unchecked() };
        let internal_node =
            unsafe { &mut *NodeRef::as_internal_ptr(&internal_self) };
        self.node = unsafe { internal_node.edges[0].assume_init_read() };
        self.height -= 1;
        self.clear_parent_link();

        unsafe {
            drop(Box::from_raw(top.cast::<InternalNode<K, V>>().as_ptr()))
        };
    }
}

impl<K, V, Type> NodeRef<marker::Owned, K, V, Type> {
    pub fn borrow_mut(&mut self) -> NodeRef<marker::Mut<'_>, K, V, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }

    pub fn borrow_valmut(&mut self) -> NodeRef<marker::ValMut<'_>, K, V, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }

    pub fn into_dying(self) -> NodeRef<marker::Dying, K, V, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<'a, K: 'a, V: 'a> NodeRef<marker::Mut<'a>, K, V, marker::Leaf> {
    pub unsafe fn push_with_handle<'b>(
        &mut self,
        key: K,
        val: V,
    ) -> Handle<NodeRef<marker::Mut<'b>, K, V, marker::Leaf>, marker::KV> {
        let len = self.len_mut();
        let idx = usize::from(*len);
        assert!(idx < CAPACITY);
        *len += 1;
        unsafe {
            self.key_area_mut(idx).write(key);
            self.val_area_mut(idx).write(val);
            Handle::new_kv(
                NodeRef {
                    height: self.height,
                    node: self.node,
                    _marker: PhantomData,
                },
                idx,
            )
        }
    }

    pub fn push(&mut self, key: K, val: V) -> *mut V {
        unsafe { self.push_with_handle(key, val).into_val_mut() }
    }
}

impl<'a, K: 'a, V: 'a> NodeRef<marker::Mut<'a>, K, V, marker::Internal> {
    pub fn push(&mut self, key: K, val: V, edge: Root<K, V>) {
        assert!(edge.height == self.height - 1);

        let len = self.len_mut();
        let idx = usize::from(*len);
        assert!(idx < CAPACITY);
        *len += 1;
        unsafe {
            self.key_area_mut(idx).write(key);
            self.val_area_mut(idx).write(val);
            self.edge_area_mut(idx + 1).write(edge.node);
            Handle::new_edge(self.reborrow_mut(), idx + 1)
                .correct_parent_link();
        }
    }
}

impl<BorrowType, K, V> NodeRef<BorrowType, K, V, marker::Leaf> {
    pub fn forget_type(
        self,
    ) -> NodeRef<BorrowType, K, V, marker::LeafOrInternal> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<BorrowType, K, V> NodeRef<BorrowType, K, V, marker::Internal> {
    pub fn forget_type(
        self,
    ) -> NodeRef<BorrowType, K, V, marker::LeafOrInternal> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<BorrowType, K, V> NodeRef<BorrowType, K, V, marker::LeafOrInternal> {
    pub fn force(
        self,
    ) -> ForceResult<
        NodeRef<BorrowType, K, V, marker::Leaf>,
        NodeRef<BorrowType, K, V, marker::Internal>,
    > {
        if self.height == 0 {
            ForceResult::Leaf(NodeRef {
                height: self.height,
                node: self.node,
                _marker: PhantomData,
            })
        } else {
            ForceResult::Internal(NodeRef {
                height: self.height,
                node: self.node,
                _marker: PhantomData,
            })
        }
    }
}

impl<'a, K, V> NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal> {
    unsafe fn cast_to_leaf_unchecked(
        self,
    ) -> NodeRef<marker::Mut<'a>, K, V, marker::Leaf> {
        debug_assert!(self.height == 0);
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }

    unsafe fn cast_to_internal_unchecked(
        self,
    ) -> NodeRef<marker::Mut<'a>, K, V, marker::Internal> {
        debug_assert!(self.height > 0);
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

pub struct Handle<NodeType, Type> {
    node: NodeType,
    idx: usize,
    _marker: PhantomData<Type>,
}

impl<Node: Copy, Type> Copy for Handle<Node, Type> {}
impl<Node: Copy, Type> Clone for Handle<Node, Type> {
    fn clone(&self) -> Self { *self }
}

impl<Node, Type> Handle<Node, Type> {
    pub fn into_node(self) -> Node { self.node }
    pub fn idx(&self) -> usize { self.idx }
}

impl<BorrowType, K, V, NodeType>
    Handle<NodeRef<BorrowType, K, V, NodeType>, marker::KV>
{
    /// `node` の中の key-value pair への handle を返す。
    ///
    /// # Safety
    /// `idx < node.len()` である必要がある。
    pub unsafe fn new_kv(
        node: NodeRef<BorrowType, K, V, NodeType>,
        idx: usize,
    ) -> Self {
        debug_assert!(idx < node.len());
        Handle { node, idx, _marker: PhantomData }
    }

    pub fn left_edge(
        self,
    ) -> Handle<NodeRef<BorrowType, K, V, NodeType>, marker::Edge> {
        unsafe { Handle::new_edge(self.node, self.idx) }
    }

    pub fn right_edge(
        self,
    ) -> Handle<NodeRef<BorrowType, K, V, NodeType>, marker::Edge> {
        unsafe { Handle::new_edge(self.node, self.idx + 1) }
    }
}

impl<BorrowType, K, V, NodeType, HandleType> PartialEq
    for Handle<NodeRef<BorrowType, K, V, NodeType>, HandleType>
{
    fn eq(&self, other: &Self) -> bool {
        let Self { node, idx, .. } = self;
        node.eq(&other.node) && *idx == other.idx
    }
}

impl<BorrowType, K, V, NodeType, HandleType>
    Handle<NodeRef<BorrowType, K, V, NodeType>, HandleType>
{
    pub fn reborrow(
        &self,
    ) -> Handle<NodeRef<marker::Immut<'_>, K, V, NodeType>, HandleType> {
        Handle {
            node: self.node.reborrow(),
            idx: self.idx,
            _marker: PhantomData,
        }
    }
}

impl<'a, K, V, NodeType, HandleType>
    Handle<NodeRef<marker::Mut<'a>, K, V, NodeType>, HandleType>
{
    pub unsafe fn reborrow_mut(
        &mut self,
    ) -> Handle<NodeRef<marker::Mut<'_>, K, V, NodeType>, HandleType> {
        Handle {
            node: unsafe { self.node.reborrow_mut() },
            idx: self.idx,
            _marker: PhantomData,
        }
    }

    pub fn dormant(
        &self,
    ) -> Handle<NodeRef<marker::DormantMut, K, V, NodeType>, HandleType> {
        Handle {
            node: self.node.dormant(),
            idx: self.idx,
            _marker: PhantomData,
        }
    }
}

impl<K, V, NodeType, HandleType>
    Handle<NodeRef<marker::DormantMut, K, V, NodeType>, HandleType>
{
    pub unsafe fn awaken<'a>(
        self,
    ) -> Handle<NodeRef<marker::Mut<'a>, K, V, NodeType>, HandleType> {
        Handle {
            node: unsafe { self.node.awaken() },
            idx: self.idx,
            _marker: PhantomData,
        }
    }
}

impl<BorrowType, K, V, NodeType>
    Handle<NodeRef<BorrowType, K, V, NodeType>, marker::Edge>
{
    /// `node` の中の辺への handle を返す。
    ///
    /// # Safety
    /// `idx <= node.len()` である必要がある。
    pub unsafe fn new_edge(
        node: NodeRef<BorrowType, K, V, NodeType>,
        idx: usize,
    ) -> Self {
        debug_assert!(idx <= node.len());
        Handle { node, idx, _marker: PhantomData }
    }

    pub fn left_kv(
        self,
    ) -> Result<Handle<NodeRef<BorrowType, K, V, NodeType>, marker::KV>, Self>
    {
        if self.idx > 0 {
            Ok(unsafe { Handle::new_kv(self.node, self.idx - 1) })
        } else {
            Err(self)
        }
    }

    pub fn right_kv(
        self,
    ) -> Result<Handle<NodeRef<BorrowType, K, V, NodeType>, marker::KV>, Self>
    {
        if self.idx < self.node.len() {
            Ok(unsafe { Handle::new_kv(self.node, self.idx) })
        } else {
            Err(self)
        }
    }
}

pub enum LeftOrRight<T> {
    Left(T),
    Right(T),
}

fn splitpoint(edge_idx: usize) -> (usize, LeftOrRight<usize>) {
    debug_assert!(edge_idx <= CAPACITY);
    match edge_idx {
        EDGE_IDX_LEFT_OF_CENTER => (KV_IDX_CENTER, LeftOrRight::Left(edge_idx)),
        EDGE_IDX_RIGHT_OF_CENTER => (KV_IDX_CENTER, LeftOrRight::Right(0)),
        0..=EDGE_IDX_LEFT_OF_CENTER => {
            (KV_IDX_CENTER - 1, LeftOrRight::Left(edge_idx))
        }
        _ => (
            KV_IDX_CENTER + 1,
            LeftOrRight::Right(edge_idx - (KV_IDX_CENTER + 1 + 1)),
        ),
    }
}

impl<'a, K: 'a, V: 'a>
    Handle<NodeRef<marker::Mut<'a>, K, V, marker::Leaf>, marker::Edge>
{
    /// 新しい key-value pair を挿入する。
    ///
    /// 挿入位置は、参照している辺の左右にある pair の間である。
    ///
    /// # Safety
    /// 新しい pair を挿入する領域が残っている必要がある。
    /// すなわち、`len() < CAPACITY` である必要がある。
    unsafe fn insert_fit(
        mut self,
        key: K,
        val: V,
    ) -> Handle<NodeRef<marker::Mut<'a>, K, V, marker::Leaf>, marker::KV> {
        debug_assert!(self.node.len() < CAPACITY);
        let new_len = self.node.len() + 1;

        unsafe {
            slice_insert(self.node.key_area_mut(..new_len), self.idx, key);
            slice_insert(self.node.val_area_mut(..new_len), self.idx, val);
            *self.node.len_mut() = new_len as u16;
            Handle::new_kv(self.node, self.idx)
        }
    }
}

impl<'a, K: 'a, V: 'a>
    Handle<NodeRef<marker::Mut<'a>, K, V, marker::Leaf>, marker::Edge>
{
    /// 新しい key-value pair を挿入する。
    ///
    /// 挿入位置は、参照している辺の左右にある pair の間である。
    /// 新しい pair を挿入する領域が足りない場合、ノードを分割する。
    ///
    /// 挿入したノードへの dormant handle を返す。これは、split が完了した後に
    /// reawaken することができる。
    fn insert(
        self,
        key: K,
        val: V,
    ) -> (
        Option<SplitResult<'a, K, V, marker::Leaf>>,
        Handle<NodeRef<marker::DormantMut, K, V, marker::Leaf>, marker::KV>,
    ) {
        if self.node.len() < CAPACITY {
            let handle = unsafe { self.insert_fit(key, val) };
            (None, handle.dormant())
        } else {
            let (middle_kv_idx, insertion) = splitpoint(self.idx);
            let middle = unsafe { Handle::new_kv(self.node, middle_kv_idx) };
            let mut result = middle.split();
            let insertion_edge = match insertion {
                LeftOrRight::Left(insert_idx) => unsafe {
                    Handle::new_edge(result.left.reborrow_mut(), insert_idx)
                },
                LeftOrRight::Right(insert_idx) => unsafe {
                    Handle::new_edge(result.right.borrow_mut(), insert_idx)
                },
            };
            let handle =
                unsafe { insertion_edge.insert_fit(key, val).dormant() };
            (Some(result), handle)
        }
    }
}

impl<'a, K, V>
    Handle<NodeRef<marker::Mut<'a>, K, V, marker::Internal>, marker::Edge>
{
    fn correct_parent_link(self) {
        let ptr = unsafe {
            NonNull::new_unchecked(NodeRef::as_internal_ptr(&self.node))
        };
        let idx = self.idx;
        let mut child = self.descend();
        child.set_parent_link(ptr, idx);
    }
}

impl<'a, K: 'a, V: 'a>
    Handle<NodeRef<marker::Mut<'a>, K, V, marker::Internal>, marker::Edge>
{
    fn insert_fit(&mut self, key: K, val: V, edge: Root<K, V>) {
        debug_assert!(self.node.len() < CAPACITY);
        debug_assert!(edge.height == self.node.height - 1);
        let new_len = self.node.len() + 1;

        unsafe {
            slice_insert(self.node.key_area_mut(..new_len), self.idx, key);
            slice_insert(self.node.val_area_mut(..new_len), self.idx, val);
            slice_insert(
                self.node.edge_area_mut(..new_len + 1),
                self.idx + 1,
                edge.node,
            );
            *self.node.len_mut() = new_len as u16;
            self.node.correct_childrens_parent_links(self.idx + 1..new_len + 1);
        }
    }

    fn insert(
        mut self,
        key: K,
        val: V,
        edge: Root<K, V>,
    ) -> Option<SplitResult<'a, K, V, marker::Internal>> {
        assert!(edge.height == self.node.height - 1);

        if self.node.len() < CAPACITY {
            self.insert_fit(key, val, edge);
            None
        } else {
            let (middle_kv_idx, insertion) = splitpoint(self.idx);
            let middle = unsafe { Handle::new_kv(self.node, middle_kv_idx) };
            let mut result = middle.split();
            let mut insertion_edge = match insertion {
                LeftOrRight::Left(insert_idx) => unsafe {
                    Handle::new_edge(result.left.reborrow_mut(), insert_idx)
                },
                LeftOrRight::Right(insert_idx) => unsafe {
                    Handle::new_edge(result.right.borrow_mut(), insert_idx)
                },
            };
            insertion_edge.insert_fit(key, val, edge);
            Some(result)
        }
    }
}

impl<'a, K: 'a, V: 'a>
    Handle<NodeRef<marker::Mut<'a>, K, V, marker::Leaf>, marker::Edge>
{
    /// 新しい key-value pair を挿入する。
    ///
    /// 挿入位置は、参照している辺の左右にある pair の間である。新しい pair
    /// を挿入する領域が足りない場合、分離した部分を再帰的に [`.insert()`]
    /// を用いて親ノードに挿入する。再帰は根ノードに到達するまで行う。
    ///
    /// [`.insert()`] の返り値が `Some(split_result)` の場合、`left` が根ノードである。
    ///
    /// TODO: `split_root` が行うべき処理の内容は？
    ///
    /// 返り値は、挿入した値への handle である。`SplitResult` の `left`
    /// に含まれる場合と `right` に含まれる場合がある。
    ///
    /// [`.insert()`]: #method.insert-1
    pub fn insert_recursing(
        self,
        key: K,
        value: V,
        split_root: impl FnOnce(SplitResult<'a, K, V, marker::LeafOrInternal>),
    ) -> Handle<NodeRef<marker::Mut<'a>, K, V, marker::Leaf>, marker::KV> {
        let (mut split, handle) = match self.insert(key, value) {
            (None, handle) => return unsafe { handle.awaken() },
            (Some(split), handle) => (split.forget_node_type(), handle),
        };

        loop {
            split = match split.left.ascend() {
                Ok(parent) => {
                    let SplitResult { kv: (k, v), right, .. } = split;
                    match parent.insert(k, v, right) {
                        None => return unsafe { handle.awaken() },
                        Some(split) => split.forget_node_type(),
                    }
                }
                Err(root) => {
                    split_root(SplitResult { left: root, ..split });
                    return unsafe { handle.awaken() };
                }
            }
        }
    }
}

impl<BorrowType: marker::BorrowType, K, V>
    Handle<NodeRef<BorrowType, K, V, marker::Internal>, marker::Edge>
{
    pub fn descend(self) -> NodeRef<BorrowType, K, V, marker::LeafOrInternal> {
        // const { assert!(BorrowType::TRAVERSAL_PERMIT) };

        let parent_ptr = NodeRef::as_internal_ptr(&self.node);
        let node = unsafe {
            (*parent_ptr).edges.get_unchecked(self.idx).assume_init_read()
        };
        NodeRef {
            node,
            height: self.node.height - 1,
            _marker: PhantomData,
        }
    }
}

impl<'a, K: 'a, V: 'a, NodeType>
    Handle<NodeRef<marker::Immut<'a>, K, V, NodeType>, marker::KV>
{
    /// key-value pair への参照を返す。
    pub fn into_kv(self) -> (&'a K, &'a V) {
        debug_assert!(self.idx < self.node.len());
        let leaf = self.node.into_leaf();
        let k = unsafe { leaf.keys.get_unchecked(self.idx).assume_init_ref() };
        let v = unsafe { leaf.vals.get_unchecked(self.idx).assume_init_ref() };
        (k, v)
    }
}

impl<'a, K: 'a, V: 'a, NodeType>
    Handle<NodeRef<marker::Mut<'a>, K, V, NodeType>, marker::KV>
{
    /// key への可変参照を返す。
    pub fn key_mut(&mut self) -> &mut K {
        unsafe { self.node.key_area_mut(self.idx).assume_init_mut() }
    }

    /// value への可変参照を返す。
    pub fn into_val_mut(self) -> &'a mut V {
        debug_assert!(self.idx < self.node.len());
        let leaf = self.node.into_leaf_mut();
        unsafe { leaf.vals.get_unchecked_mut(self.idx).assume_init_mut() }
    }

    /// key-value pair への可変参照を返す。
    pub fn into_kv_mut(self) -> (&'a mut K, &'a mut V) {
        debug_assert!(self.idx < self.node.len());
        let leaf = self.node.into_leaf_mut();
        let k =
            unsafe { leaf.keys.get_unchecked_mut(self.idx).assume_init_mut() };
        let v =
            unsafe { leaf.vals.get_unchecked_mut(self.idx).assume_init_mut() };
        (k, v)
    }
}

impl<'a, K, V, NodeType>
    Handle<NodeRef<marker::ValMut<'a>, K, V, NodeType>, marker::KV>
{
    /// key-value pair への参照を返す。
    pub fn into_kv_valmut(self) -> (&'a K, &'a mut V) {
        unsafe { self.node.into_key_val_mut_at(self.idx) }
    }
}

impl<'a, K: 'a, V: 'a, NodeType>
    Handle<NodeRef<marker::Mut<'a>, K, V, NodeType>, marker::KV>
{
    /// key-value pair への可変参照を返す。
    pub fn kv_mut(&mut self) -> (&mut K, &mut V) {
        debug_assert!(self.idx < self.node.len());
        unsafe {
            let leaf = self.node.as_leaf_mut();
            let key = leaf.keys.get_unchecked_mut(self.idx).assume_init_mut();
            let val = leaf.vals.get_unchecked_mut(self.idx).assume_init_mut();
            (key, val)
        }
    }

    /// 参照している key-value pair を置き換える。
    pub fn replace_kv(&mut self, k: K, v: V) -> (K, V) {
        let (key, val) = self.kv_mut();
        (std::mem::replace(key, k), std::mem::replace(val, v))
    }
}

impl<K, V, NodeType>
    Handle<NodeRef<marker::Dying, K, V, NodeType>, marker::KV>
{
    /// 参照している key-value pair を抽出する。
    ///
    /// # Safety
    /// 該当の pair はまだ解放されていない必要がある。
    pub unsafe fn into_key_val(mut self) -> (K, V) {
        debug_assert!(self.idx < self.node.len());
        let leaf = self.node.as_leaf_dying();
        unsafe {
            let key = leaf.keys.get_unchecked_mut(self.idx).assume_init_read();
            let val = leaf.vals.get_unchecked_mut(self.idx).assume_init_read();
            (key, val)
        }
    }

    /// 参照している key-value pair を drop する。
    ///
    /// # Safety
    /// 該当の pair はまだ解放されていない必要がある。
    pub unsafe fn drop_key_val(mut self) {
        debug_assert!(self.idx < self.node.len());
        let leaf = self.node.as_leaf_dying();
        unsafe {
            leaf.keys.get_unchecked_mut(self.idx).assume_init_drop();
            leaf.vals.get_unchecked_mut(self.idx).assume_init_drop();
        }
    }
}

impl<'a, K: 'a, V: 'a, NodeType>
    Handle<NodeRef<marker::Mut<'a>, K, V, NodeType>, marker::KV>
{
    /// [`.split()`] のヘルパー実装。
    fn split_leaf_data(&mut self, new_node: &mut LeafNode<K, V>) -> (K, V) {
        debug_assert!(self.idx < self.node.len());
        let old_len = self.node.len();
        let new_len = old_len - self.idx - 1;
        new_node.len = new_len as u16;
        unsafe {
            let k = self.node.key_area_mut(self.idx).assume_init_read();
            let v = self.node.val_area_mut(self.idx).assume_init_read();

            move_to_slice(
                self.node.key_area_mut(self.idx + 1..old_len),
                &mut new_node.keys[..new_len],
            );
            move_to_slice(
                self.node.val_area_mut(self.idx + 1..old_len),
                &mut new_node.vals[..new_len],
            );

            *self.node.len_mut() = self.idx as u16;
            (k, v)
        }
    }
}

impl<'a, K: 'a, V: 'a>
    Handle<NodeRef<marker::Mut<'a>, K, V, marker::Leaf>, marker::KV>
{
    /// 参照しているノードを 3 つの要素に分割する。
    ///
    /// - 既存のノードは、handle の左にある要素のみになる。
    /// - 参照している key-value pair は抽出される。
    /// - handle の右にある要素は、新しく確保されたノードに詰められる。
    pub fn split(mut self) -> SplitResult<'a, K, V, marker::Leaf> {
        let mut new_node = LeafNode::new();
        let kv = self.split_leaf_data(&mut new_node);
        let right = NodeRef::from_new_leaf(new_node);
        SplitResult { left: self.node, kv, right }
    }

    /// 参照している key-value pair を取り除く。
    ///
    /// 指していた値と、それに付随する辺が返される。
    pub fn remove(
        mut self,
    ) -> (
        (K, V),
        Handle<NodeRef<marker::Mut<'a>, K, V, marker::Leaf>, marker::Edge>,
    ) {
        let old_len = self.node.len();
        unsafe {
            let k = slice_remove(self.node.key_area_mut(..old_len), self.idx);
            let v = slice_remove(self.node.val_area_mut(..old_len), self.idx);
            *self.node.len_mut() = (old_len - 1) as u16;
            ((k, v), self.left_edge())
        }
    }
}

impl<'a, K: 'a, V: 'a>
    Handle<NodeRef<marker::Mut<'a>, K, V, marker::Internal>, marker::KV>
{
    /// 指しているノードを 3 つの要素に分割する。
    pub fn split(mut self) -> SplitResult<'a, K, V, marker::Internal> {
        let old_len = self.node.len();
        unsafe {
            let mut new_node = InternalNode::new();
            let kv = self.split_leaf_data(&mut new_node.data);
            let new_len = usize::from(new_node.data.len);
            move_to_slice(
                self.node.edge_area_mut(self.idx + 1..old_len + 1),
                &mut new_node.edges[..new_len + 1],
            );

            let height = self.node.height;
            let right = NodeRef::from_new_internal(new_node, height);

            SplitResult { left: self.node, kv, right }
        }
    }
}

pub struct BalancingContext<'a, K, V> {
    parent:
        Handle<NodeRef<marker::Mut<'a>, K, V, marker::Internal>, marker::KV>,
    left_child: NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal>,
    right_child: NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal>,
}

impl<'a, K, V>
    Handle<NodeRef<marker::Mut<'a>, K, V, marker::Internal>, marker::KV>
{
    pub fn consider_for_balancing(self) -> BalancingContext<'a, K, V> {
        let self1 = unsafe { ptr::read(&self) };
        let self2 = unsafe { ptr::read(&self) };
        BalancingContext {
            parent: self,
            left_child: self1.left_edge().descend(),
            right_child: self2.right_edge().descend(),
        }
    }
}

impl<'a, K, V> NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal> {
    pub fn choose_parent_kv(
        self,
    ) -> Result<LeftOrRight<BalancingContext<'a, K, V>>, Self> {
        match unsafe { ptr::read(&self) }.ascend() {
            Ok(parent_edge) => match parent_edge.left_kv() {
                Ok(left_parent_kv) => Ok(LeftOrRight::Left(BalancingContext {
                    parent: unsafe { ptr::read(&left_parent_kv) },
                    left_child: left_parent_kv.left_edge().descend(),
                    right_child: self,
                })),
                Err(parent_edge) => match parent_edge.right_kv() {
                    Ok(right_parent_kv) => {
                        Ok(LeftOrRight::Right(BalancingContext {
                            parent: unsafe { ptr::read(&right_parent_kv) },
                            left_child: self,
                            right_child: right_parent_kv.right_edge().descend(),
                        }))
                    }
                    Err(_) => unreachable!("empty internal node"),
                },
            },
            Err(root) => Err(root),
        }
    }
}

impl<'a, K, V> BalancingContext<'a, K, V> {
    pub fn left_child_len(&self) -> usize { self.left_child.len() }
    pub fn right_child_len(&self) -> usize { self.right_child.len() }
    pub fn into_left_child(
        self,
    ) -> NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal> {
        self.left_child
    }
    pub fn into_right_child(
        self,
    ) -> NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal> {
        self.right_child
    }
    pub fn can_merge(&self) -> bool {
        self.left_child.len() + 1 + self.right_child.len() <= CAPACITY
    }
}

impl<'a, K: 'a, V: 'a> BalancingContext<'a, K, V> {
    fn do_merge<
        F: FnOnce(
            NodeRef<marker::Mut<'a>, K, V, marker::Internal>,
            NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal>,
        ) -> R,
        R,
    >(
        self,
        result: F,
    ) -> R {
        let Handle { node: mut parent_node, idx: parent_idx, .. } = self.parent;
        let old_parent_len = parent_node.len();
        let mut left_node = self.left_child;
        let old_left_len = left_node.len();
        let mut right_node = self.right_child;
        let right_len = right_node.len();
        let new_left_len = old_left_len + 1 + right_len;

        assert!(new_left_len <= CAPACITY);

        unsafe {
            *left_node.len_mut() = new_left_len as u16;

            let parent_key = slice_remove(
                parent_node.key_area_mut(..old_parent_len),
                parent_idx,
            );
            left_node.key_area_mut(old_left_len).write(parent_key);
            move_to_slice(
                right_node.key_area_mut(..right_len),
                left_node.key_area_mut(old_left_len + 1..new_left_len),
            );

            let parent_val = slice_remove(
                parent_node.val_area_mut(..old_parent_len),
                parent_idx,
            );
            left_node.val_area_mut(old_left_len).write(parent_val);
            move_to_slice(
                right_node.val_area_mut(..right_len),
                left_node.val_area_mut(old_left_len + 1..new_left_len),
            );

            slice_remove(
                &mut parent_node.edge_area_mut(..old_parent_len + 1),
                parent_idx + 1,
            );
            parent_node
                .correct_childrens_parent_links(parent_idx + 1..old_parent_len);
            *parent_node.len_mut() -= 1;

            if parent_node.height > 1 {
                let mut left_node =
                    left_node.reborrow_mut().cast_to_internal_unchecked();
                let mut right_node = right_node.cast_to_internal_unchecked();
                move_to_slice(
                    right_node.edge_area_mut(..right_len + 1),
                    left_node.edge_area_mut(old_left_len + 1..new_left_len + 1),
                );

                left_node.correct_childrens_parent_links(
                    old_left_len + 1..new_left_len + 1,
                );
                drop(Box::from_raw(NodeRef::as_internal_ptr(&mut right_node)));
            } else {
                drop(Box::from_raw(right_node.node.as_ptr()));
            }
        }
        result(parent_node, left_node)
    }

    pub fn merge_tracking_parent(
        self,
    ) -> NodeRef<marker::Mut<'a>, K, V, marker::Internal> {
        self.do_merge(|parent, _child| parent)
    }

    pub fn merge_tracking_child(
        self,
    ) -> NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal> {
        self.do_merge(|_parent, child| child)
    }

    pub fn merge_tracking_child_edge(
        self,
        track_edge_idx: LeftOrRight<usize>,
    ) -> Handle<
        NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal>,
        marker::Edge,
    > {
        let old_left_len = self.left_child.len();
        let right_len = self.right_child.len();
        assert!(match track_edge_idx {
            LeftOrRight::Left(idx) => idx <= old_left_len,
            LeftOrRight::Right(idx) => idx <= right_len,
        });
        let child = self.merge_tracking_child();
        let new_idx = match track_edge_idx {
            LeftOrRight::Left(idx) => idx,
            LeftOrRight::Right(idx) => old_left_len + 1 + idx,
        };
        unsafe { Handle::new_edge(child, new_idx) }
    }

    pub fn steal_left(
        mut self,
        track_right_edge_idx: usize,
    ) -> Handle<
        NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal>,
        marker::Edge,
    > {
        self.bulk_steal_left(1);
        unsafe { Handle::new_edge(self.right_child, 1 + track_right_edge_idx) }
    }

    pub fn steal_right(
        mut self,
        track_left_edge_idx: usize,
    ) -> Handle<
        NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal>,
        marker::Edge,
    > {
        self.bulk_steal_right(1);
        unsafe { Handle::new_edge(self.left_child, track_left_edge_idx) }
    }

    pub fn bulk_steal_left(&mut self, count: usize) { todo!() }
    pub fn bulk_steal_right(&mut self, count: usize) { todo!() }
}

impl<BorrowType, K, V>
    Handle<NodeRef<BorrowType, K, V, marker::Leaf>, marker::Edge>
{
    pub fn forget_node_type(
        self,
    ) -> Handle<NodeRef<BorrowType, K, V, marker::LeafOrInternal>, marker::Edge>
    {
        unsafe { Handle::new_edge(self.node.forget_type(), self.idx) }
    }
}

impl<BorrowType, K, V>
    Handle<NodeRef<BorrowType, K, V, marker::Internal>, marker::Edge>
{
    pub fn forget_node_type(
        self,
    ) -> Handle<NodeRef<BorrowType, K, V, marker::LeafOrInternal>, marker::Edge>
    {
        unsafe { Handle::new_edge(self.node.forget_type(), self.idx) }
    }
}

impl<BorrowType, K, V>
    Handle<NodeRef<BorrowType, K, V, marker::Leaf>, marker::KV>
{
    pub fn forget_node_type(
        self,
    ) -> Handle<NodeRef<BorrowType, K, V, marker::LeafOrInternal>, marker::KV>
    {
        unsafe { Handle::new_kv(self.node.forget_type(), self.idx) }
    }
}

impl<BorrowType, K, V, Type>
    Handle<NodeRef<BorrowType, K, V, marker::LeafOrInternal>, Type>
{
    pub fn force(
        self,
    ) -> ForceResult<
        Handle<NodeRef<BorrowType, K, V, marker::Leaf>, Type>,
        Handle<NodeRef<BorrowType, K, V, marker::Internal>, Type>,
    > {
        match self.node.force() {
            ForceResult::Leaf(node) => ForceResult::Leaf(Handle {
                node,
                idx: self.idx,
                _marker: PhantomData,
            }),
            ForceResult::Internal(node) => ForceResult::Internal(Handle {
                node,
                idx: self.idx,
                _marker: PhantomData,
            }),
        }
    }
}

impl<'a, K, V, Type>
    Handle<NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal>, Type>
{
    pub unsafe fn cast_to_leaf_unchecked(
        self,
    ) -> Handle<NodeRef<marker::Mut<'a>, K, V, marker::Leaf>, Type> {
        let node = unsafe { self.node.cast_to_leaf_unchecked() };
        Handle { node, idx: self.idx, _marker: PhantomData }
    }
}

impl<'a, K, V>
    Handle<NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal>, marker::Edge>
{
    pub fn move_suffix(
        &mut self,
        right: &mut NodeRef<marker::Mut<'a>, K, V, marker::LeafOrInternal>,
    ) {
        todo!()
    }
}

pub enum ForceResult<Leaf, Internal> {
    Leaf(Leaf),
    Internal(Internal),
}

pub struct SplitResult<'a, K, V, NodeType> {
    pub left: NodeRef<marker::Mut<'a>, K, V, NodeType>,
    pub kv: (K, V),
    pub right: NodeRef<marker::Owned, K, V, NodeType>,
}

impl<'a, K, V> SplitResult<'a, K, V, marker::Leaf> {
    pub fn forget_node_type(
        self,
    ) -> SplitResult<'a, K, V, marker::LeafOrInternal> {
        SplitResult {
            left: self.left.forget_type(),
            kv: self.kv,
            right: self.right.forget_type(),
        }
    }
}

impl<'a, K, V> SplitResult<'a, K, V, marker::Internal> {
    pub fn forget_node_type(
        self,
    ) -> SplitResult<'a, K, V, marker::LeafOrInternal> {
        SplitResult {
            left: self.left.forget_type(),
            kv: self.kv,
            right: self.right.forget_type(),
        }
    }
}

/// マーカー。
///
/// 種類や lifetime を管理する。
pub mod marker {
    use std::marker::PhantomData;

    /// 葉ノードを指すことを表す。
    pub enum Leaf {}
    /// 内部ノードを指すことを表す。
    pub enum Internal {}
    /// 葉ノードまたは内部ノードを指すことを表す。
    pub enum LeafOrInternal {}

    /// `Boxed<Node>` を表す。
    pub enum Owned {}
    /// 破棄中の `Boxed<Node>` を表す。
    pub enum Dying {}
    /// `*mut Node` を表す。
    pub enum DormantMut {}
    /// `&'a Node` を表す。
    pub struct Immut<'a>(PhantomData<&'a ()>);
    /// `&'a mut Node` を表す。
    pub struct Mut<'a>(PhantomData<&'a mut ()>);
    /// `&'a Node` を表しつつ、`&'a mut V` たちの共存も許容する。
    pub struct ValMut<'a>(PhantomData<&'a mut ()>);

    /// 借用の種類を表すマーカー。
    pub trait BorrowType {
        /// 木を探索することを許容するときは `true` とする。コンパイル時の
        /// assertion に使用可能。
        const TRAVERSAL_PERMIT: bool = true;
    }
    impl BorrowType for Owned {
        /// 必要がないため許可しない。必要なときは `borrow_mut()` の返り値を使う。
        /// 新しく参照を作るときは根ノードのみで、探索を無効化しているため、`Owned`
        /// の参照はすべて根ノードを指すことになる。
        const TRAVERSAL_PERMIT: bool = false;
    }
    impl BorrowType for Dying {}
    impl<'a> BorrowType for Immut<'a> {}
    impl<'a> BorrowType for Mut<'a> {}
    impl<'a> BorrowType for ValMut<'a> {}
    impl BorrowType for DormantMut {}

    pub enum KV {}
    pub enum Edge {}
}

unsafe fn slice_insert<T>(slice: &mut [MaybeUninit<T>], idx: usize, val: T) {
    unsafe {
        let len = slice.len();
        debug_assert!(len > idx);
        let slice_ptr = slice.as_mut_ptr();
        if len > idx + 1 {
            ptr::copy(
                slice_ptr.add(idx),
                slice_ptr.add(idx + 1),
                len - idx - 1,
            );
        }
        (*slice_ptr.add(idx)).write(val);
    }
}

unsafe fn slice_remove<T>(slice: &mut [MaybeUninit<T>], idx: usize) -> T {
    unsafe {
        let len = slice.len();
        debug_assert!(idx < len);
        let slice_ptr = slice.as_mut_ptr();
        let ret = (*slice_ptr.add(idx)).assume_init_read();
        ptr::copy(slice_ptr.add(idx + 1), slice_ptr.add(idx), len - idx - 1);
        ret
    }
}

unsafe fn slice_shl<T>(slice: &mut [MaybeUninit<T>], distance: usize) {
    unsafe {
        let slice_ptr = slice.as_mut_ptr();
        ptr::copy(slice_ptr.add(distance), slice_ptr, slice.len() - distance);
    }
}

unsafe fn slice_shr<T>(slice: &mut [MaybeUninit<T>], distance: usize) {
    unsafe {
        let slice_ptr = slice.as_mut_ptr();
        ptr::copy(slice_ptr, slice_ptr.add(distance), slice.len() - distance);
    }
}

fn move_to_slice<T>(src: &mut [MaybeUninit<T>], dst: &mut [MaybeUninit<T>]) {
    assert_eq!(src.len(), dst.len());
    unsafe {
        ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }
}

#[test]
fn insert_test() {
    use crate::borrow::DormantMutRef;

    struct BTreeMap<'a, K, V> {
        root: Option<NodeRef<marker::Owned, K, V, marker::LeafOrInternal>>,
        len: usize,
        _marker: PhantomData<&'a mut (K, V)>,
    }
    let mut map = BTreeMap {
        root: Some(NodeRef::new()),
        len: 0,
        _marker: PhantomData,
    };

    for i in 0..CAPACITY + 1 {
        let (map, mut dormant_map) = DormantMutRef::new(&mut map);
        let mut_root = map.root.as_mut().unwrap().borrow_mut();
        let mut handle = mut_root.first_leaf_edge();
        handle.insert_recursing(0, 0, |ins| {
            let SplitResult { left: _, kv: (key, val), right } = ins;
            let mut map = unsafe { dormant_map.reborrow() };
            let root = map.root.as_mut().unwrap();
            root.push_internal_level().push(key, val, right)
        });
        // let mut map = unsafe { dormant_map.awaken() };
        map.len += 1;
    }

    let mut dying_root = map.root.unwrap().into_dying();
    eprintln!("{:?}", dying_root.height);
    {
        let root1 = unsafe { ptr::read(&dying_root) };
        let root2 = unsafe { ptr::read(&dying_root) };
        let mut handle = match root1.first_edge().force() {
            ForceResult::Internal(internal) => internal,
            _ => unreachable!(),
        };
        unsafe {
            handle.descend().deallocate_and_ascend();
        }
        let mut handle = match root2.last_edge().force() {
            ForceResult::Internal(internal) => internal,
            _ => unreachable!(),
        };
        unsafe {
            let parent = handle.descend().deallocate_and_ascend().unwrap();
            parent.into_node().forget_type().deallocate_and_ascend();
        }
    }
}
