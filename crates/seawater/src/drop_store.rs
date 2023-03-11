use crate::{LiquidDrop, LiquidDropCache};
use once_cell::race::OnceBox;
use parking_lot::{lock_api::RwLockReadGuard, MappedRwLockReadGuard, RwLock};
use std::{
  fmt::{Debug, Display},
  hash::Hash,
  sync::{Arc, Weak},
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
  weak: Weak<Self>,
}

impl<ID: Eq + Hash + Copy + Display + Debug> DropStore<ID> {
  pub fn new() -> Arc<Self> {
    Arc::new_cyclic(|weak| DropStore {
      storage: RwLock::new(AnyMap::new()),
      weak: weak.clone(),
    })
  }

  pub fn get<D: LiquidDrop + 'static>(&self, id: ID) -> Option<MappedRwLockReadGuard<'_, D>> {
    let lock = self.storage.read();
    RwLockReadGuard::try_map(lock, |lock| {
      lock.get::<DropAndCache<D>>(id).map(|dc| &dc.drop)
    })
    .ok()
  }

  pub fn get_drop_cache<D: LiquidDrop + 'static>(
    &self,
    drop_id: ID,
  ) -> MappedRwLockReadGuard<'_, D::Cache>
  where
    ID: Debug,
  {
    let lock = self.storage.read();
    RwLockReadGuard::try_map(lock, |lock| {
      lock.get::<DropAndCache<D>>(drop_id).map(|dc| &dc.cache)
    })
    .unwrap()
  }

  pub fn store<D: LiquidDrop + Clone + Send + Sync + 'static>(&self, drop: D) -> DropRef<D, ID>
  where
    ID: From<D::ID> + Send + Sync,
  {
    self.get_or_insert(drop)
  }

  pub fn store_all<D: LiquidDrop + Clone + Send + Sync + 'static, I: IntoIterator<Item = D>>(
    &self,
    drops: I,
  ) -> Vec<DropRef<D, ID>>
  where
    ID: From<D::ID> + Send + Sync,
  {
    drops.into_iter().map(|drop| self.store(drop)).collect()
  }

  fn get_or_insert<D: LiquidDrop + Clone + Send + Sync + 'static>(&self, value: D) -> DropRef<D, ID>
  where
    ID: From<D::ID> + Send + Sync,
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

    DropRef::new(id.into(), self.weak.clone())
  }
}
