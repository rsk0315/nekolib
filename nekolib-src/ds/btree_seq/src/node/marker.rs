use std::marker::PhantomData;

pub enum Owned {}
pub enum Dying {}
pub enum DormantMut {}
pub struct Immut<'a>(PhantomData<&'a ()>);
pub struct Mut<'a>(PhantomData<&'a mut ()>);
pub struct ValMut<'a>(PhantomData<&'a mut ()>);

pub enum Leaf {}
pub enum Internal {}
pub enum LeafOrInternal {}

pub enum Edge {}
pub enum Value {}

pub trait Traversable {}

impl Traversable for Dying {}
impl Traversable for DormantMut {}
impl Traversable for Immut<'_> {}
impl Traversable for Mut<'_> {}
impl Traversable for ValMut<'_> {}
