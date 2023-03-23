use crate::{LiquidDrop, LiquidDropCache};
use parking_lot::RwLock;
use std::{
  fmt::{Debug, Display},
  hash::Hash,
  sync::{Arc, Weak},
};

use crate::{any_store::AnyStore, DropRef};

#[derive(Debug, Default)]
pub struct DropStore<ID: Eq + Hash + Copy + Display + Debug> {
  drops_by_id: RwLock<AnyStore<ID>>,
  weak: Weak<Self>,
}

impl<ID: Eq + Hash + Copy + Display + Debug> DropStore<ID> {
  pub fn new() -> Arc<Self> {
    Arc::new_cyclic(|weak| DropStore {
      drops_by_id: RwLock::new(AnyStore::new()),
      weak: weak.clone(),
    })
  }

  pub fn get<D: LiquidDrop + Send + Sync + 'static>(&self, id: ID) -> Option<Arc<D>> {
    let lock = self.drops_by_id.read();

    lock.get::<D>(id)
  }

  pub fn get_drop_cache<D: LiquidDrop + Send + Sync + 'static>(
    &self,
    drop_id: ID,
  ) -> Arc<D::Cache> {
    let lock = self.drops_by_id.read();
    lock.get::<D::Cache>(drop_id).unwrap()
  }

  pub fn get_all<D: LiquidDrop + Send + Sync + 'static>(
    &self,
    ids: impl IntoIterator<Item = ID>,
  ) -> Vec<D> {
    let lock = self.drops_by_id.read();

    ids
      .into_iter()
      .filter_map(|id| lock.get::<D>(id).as_deref().cloned())
      .collect()
  }

  pub fn store<
    D: LiquidDrop<Context = Context> + Clone + Send + Sync + 'static,
    Context: crate::Context<StoreID = ID>,
  >(
    &self,
    drop: D,
  ) -> DropRef<D>
  where
    ID: From<D::ID> + Send + Sync,
  {
    self.get_or_insert(drop)
  }

  pub fn store_all<
    D: LiquidDrop<Context = Context> + Clone + Send + Sync + 'static,
    I: IntoIterator<Item = D>,
    Context: crate::Context<StoreID = ID>,
  >(
    &self,
    drops: I,
  ) -> Vec<DropRef<D>>
  where
    ID: From<D::ID> + Send + Sync,
  {
    drops.into_iter().map(|drop| self.store(drop)).collect()
  }

  fn get_or_insert<
    D: LiquidDrop<Context = Context> + Clone + Send + Sync + 'static,
    Context: crate::Context<StoreID = ID>,
  >(
    &self,
    value: D,
  ) -> DropRef<D>
  where
    ID: From<D::ID> + Send + Sync,
  {
    let mut lock = self.drops_by_id.write();
    let id: ID = value.id().into();
    lock.get_or_insert(id, value);
    lock.get_or_insert(id, D::Cache::new());

    DropRef::new(id, self.weak.clone())
  }
}
