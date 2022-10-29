use lazy_liquid_value_view::{ArcValueView, LiquidDrop, LiquidDropWithID};
use once_cell::race::OnceBox;
use std::{
  hash::Hash,
  sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::any_map::AnyMap;

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

  pub fn put<D: LiquidDrop + LiquidDropWithID + Send + Sync + 'static>(
    &self,
    value: D,
  ) -> Result<ArcValueView<D>, PoisonError<RwLockWriteGuard<AnyMap<ID>>>>
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

      once_box
        .get_or_init(|| Box::new(ArcValueView(Arc::new(value))))
        .clone()
    })
  }
}
