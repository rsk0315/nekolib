//! split の実装。
//!
//! 構造体などの定義はなく、[`Root`] に対する `impl` をしている。

use std::borrow::Borrow;

use crate::node::Root;

impl<K, V> Root<K, V> {
    pub fn calc_split_length(
        total_num: usize,
        root_a: &Root<K, V>,
        root_b: &Root<K, V>,
    ) -> (usize, usize) {
        todo!()
    }

    pub fn split_off<Q: ?Sized + Ord>(&mut self, key: &Q) -> Self
    where
        K: Borrow<Q>,
    {
        todo!()
    }

    fn new_pillar(height: usize) -> Self { todo!() }
}
