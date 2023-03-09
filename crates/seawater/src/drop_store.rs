use crate::{LiquidDrop, LiquidDropCache};
use once_cell::race::OnceBox;
use parking_lot::{lock_api::RwLockReadGuard, MappedRwLockReadGuard, RwLock};
use std::{
  fmt::{Debug, Display},
  hash::Hash,
};

use crate::{any_map::AnyMap, DropRef};

struct DropAndCache<D: LiquidDrop> {
  drop: D,
  cache: D::Cache,
}

impl<D: LiquidDrop> DropAndCache<D> {
  pub fn new(drop: D) -> Self {
    DropAndCache {
      drop,
      cache: D::Cache::new(),
    }
  }
}

#[derive(Debug, Default)]
pub struct DropStore<ID: Eq + Hash + Copy + Display + Debug> {
  storage: RwLock<AnyMap<ID>>,
}

impl<ID: Eq + Hash + Copy + Display + Debug> DropStore<ID> {
  pub fn get<D: LiquidDrop + 'static>(&self, id: ID) -> Option<MappedRwLockReadGuard<D>> {
    let lock = self.storage.read();
    RwLockReadGuard::try_map(lock, |lock| {
      lock.get::<DropAndCache<D>>(id).map(|dc| &dc.drop)
    })
    .ok()
  }

  pub fn get_drop_cache<D: LiquidDrop + 'static>(
    &self,
    drop_id: D::ID,
  ) -> MappedRwLockReadGuard<D::Cache>
  where
    ID: From<D::ID>,
    D::ID: Debug,
  {
    let lock = self.storage.read();
    RwLockReadGuard::try_map(lock, |lock| {
      lock
        .get::<DropAndCache<D>>(drop_id.into())
        .map(|dc| &dc.cache)
    })
    .unwrap()
  }

  pub fn store<D: LiquidDrop + Clone + Send + Sync + 'static>(&self, drop: D) -> DropRef<D, ID>
  where
    ID: From<D::ID>,
  {
    self.get_or_insert(drop)
  }

  pub fn store_all<D: LiquidDrop + Clone + Send + Sync + 'static, I: IntoIterator<Item = D>>(
    &self,
    drops: I,
  ) -> Vec<DropRef<D, ID>>
  where
    ID: From<D::ID>,
  {
    drops.into_iter().map(|drop| self.store(drop)).collect()
  }

  fn get_or_insert_with<D: LiquidDrop + Clone + Send + Sync + 'static, F: FnOnce() -> D>(
    &self,
    id: ID,
    init: F,
  ) -> DropRef<D, ID>
  where
    ID: From<D::ID>,
    D::ID: From<ID>,
  {
    match self.get::<D>(id).as_ref() {
      Some(drop) => DropRef::new(drop.id(), self),
      None => {
        let drop = init();
        self.get_or_insert(drop)
      }
    }
  }

  fn get_or_insert<D: LiquidDrop + Clone + Send + Sync + 'static>(&self, value: D) -> DropRef<D, ID>
  where
    ID: From<D::ID>,
  {
    let mut lock = self.storage.write();
    let id = value.id();
    let once_box = lock.get::<OnceBox<DropAndCache<D>>>(id.into());

    let once_box = if let Some(once_box) = once_box {
      once_box
    } else {
      lock.insert::<OnceBox<DropAndCache<D>>>(id.into(), Default::default());
      lock.get::<OnceBox<DropAndCache<D>>>(id.into()).unwrap()
    };

    once_box.get_or_init(|| Box::new(DropAndCache::new(value)));

    DropRef::new(id, self)
  }
}
