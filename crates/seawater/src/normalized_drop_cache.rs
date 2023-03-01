use lazy_liquid_value_view::{ArcValueView, LiquidDrop, LiquidDropWithID};
use once_cell::race::OnceBox;
use std::{
  fmt::Debug,
  hash::Hash,
  sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{any_map::AnyMap, DropRef};

#[derive(Clone, Debug, Default)]
pub struct NormalizedDropCache<ID: Eq + Hash + Copy> {
  storage: Arc<RwLock<AnyMap<ID>>>,
}

impl<ID: Eq + Hash + Copy> NormalizedDropCache<ID> {
  pub fn get<D: LiquidDrop + LiquidDropWithID + 'static>(
    &self,
    id: ID,
  ) -> Result<Option<ArcValueView<D>>, PoisonError<RwLockReadGuard<AnyMap<ID>>>> {
    self.storage.read().map(|lock| {
      lock
        .get::<OnceBox<ArcValueView<D>>>(id)
        .and_then(|once_box| once_box.get())
        .cloned()
    })
  }

  pub fn normalize<D: LiquidDrop + LiquidDropWithID + Send + Sync + 'static>(
    &self,
    drop: D,
  ) -> Result<DropRef<D>, PoisonError<RwLockWriteGuard<AnyMap<ID>>>>
  where
    ID: From<D::ID>,
  {
    self.get_or_insert(drop)
  }

  pub fn normalize_all<
    D: LiquidDrop + LiquidDropWithID + Send + Sync + 'static,
    I: IntoIterator<Item = D>,
  >(
    &self,
    drops: I,
  ) -> Result<Vec<DropRef<D>>, PoisonError<RwLockWriteGuard<AnyMap<ID>>>>
  where
    ID: From<D::ID>,
  {
    drops.into_iter().map(|drop| self.normalize(drop)).collect()
  }

  fn get_or_insert_with<
    D: LiquidDrop + LiquidDropWithID + Send + Sync + 'static,
    F: FnOnce() -> D,
  >(
    &self,
    id: ID,
    init: F,
  ) -> Result<DropRef<D>, PoisonError<()>>
  where
    ID: From<D::ID>,
    D::ID: From<ID>,
  {
    match self.get::<D>(id).map_err(|_| PoisonError::new(()))? {
      Some(drop) => Ok(DropRef {
        id: id.into(),
        _phantom: Default::default(),
      }),
      None => {
        let drop = init();
        self.get_or_insert(drop).map_err(|_| PoisonError::new(()))
      }
    }
  }

  fn get_or_insert<D: LiquidDrop + LiquidDropWithID + Send + Sync + 'static>(
    &self,
    value: D,
  ) -> Result<DropRef<D>, PoisonError<RwLockWriteGuard<AnyMap<ID>>>>
  where
    D::ID: Into<ID>,
  {
    self.storage.write().map(|mut lock| {
      let id = value.id();
      let once_box = lock.get::<OnceBox<ArcValueView<D>>>(id.into());

      let once_box = if let Some(once_box) = once_box {
        once_box
      } else {
        lock.insert::<OnceBox<ArcValueView<D>>>(id.into(), Default::default());
        lock.get::<OnceBox<ArcValueView<D>>>(id.into()).unwrap()
      };

      once_box.get_or_init(|| Box::new(ArcValueView(Arc::new(value))));

      DropRef {
        id,
        _phantom: Default::default(),
      }
    })
  }
}
