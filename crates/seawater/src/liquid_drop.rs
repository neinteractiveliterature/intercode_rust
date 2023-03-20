use crate::{Context, DropResult, DropStore};
use liquid::{ObjectView, ValueView};
use std::{
  fmt::{Debug, Display},
  hash::Hash,
};

pub type DropID<D> = <D as LiquidDrop>::ID;
pub type DropCache<D> = <D as LiquidDrop>::Cache;
pub type DropContext<D> = <D as LiquidDrop>::Context;
pub type DropStoreID<D> = <DropContext<D> as Context>::StoreID;

pub trait LiquidDrop: ValueView + ObjectView + Clone + Into<DropResult<Self>> {
  type Cache: LiquidDropCache;
  type ID: Eq + Hash + Copy + Display + Send + Sync + Debug;
  type Context: Context;

  fn id(&self) -> Self::ID;
  fn get_context(&self) -> &Self::Context;

  fn with_drop_store<
    'store,
    R: 'store,
    F: FnOnce(&'store DropStore<<Self::Context as Context>::StoreID>) -> R,
  >(
    &'store self,
    f: F,
  ) -> R {
    self.get_context().with_drop_store(f)
  }
}

pub trait LiquidDropCache: Send + Sync {
  fn new() -> Self;
}
