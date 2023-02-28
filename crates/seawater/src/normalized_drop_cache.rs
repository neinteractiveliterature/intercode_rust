use lazy_liquid_value_view::{ArcValueView, LiquidDrop, LiquidDropWithID};
use once_cell::race::OnceBox;
use std::{
  hash::Hash,
  sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use tracing::log::warn;

use crate::any_map::AnyMap;

#[derive(Clone, Debug, Default)]
pub struct NormalizedDropCache<ID: Eq + Hash + Copy> {
  storage: Arc<RwLock<AnyMap<ID>>>,
}

impl<ID: Eq + Hash + Copy> Drop for NormalizedDropCache<ID> {
  fn drop(&mut self) {
    eprintln!(
      "Dropping NDC; storage has {} strong refs",
      Arc::strong_count(&self.storage)
    );
  }
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
  ) -> Result<ArcValueView<D>, PoisonError<RwLockWriteGuard<AnyMap<ID>>>>
  where
    ID: From<D::ID>,
  {
    self.get_or_insert(drop)
  }

  pub fn normalize_ref<D: Clone + LiquidDrop + LiquidDropWithID + Send + Sync + 'static>(
    &self,
    drop_ref: &D,
  ) -> Result<ArcValueView<D>, PoisonError<()>>
  where
    ID: From<D::ID>,
  {
    self.get_or_insert_with(drop_ref.id().into(), || drop_ref.clone())
  }

  pub fn normalize_all<
    D: LiquidDrop + LiquidDropWithID + Send + Sync + 'static,
    I: IntoIterator<Item = D>,
  >(
    &self,
    drops: I,
  ) -> Result<Vec<ArcValueView<D>>, PoisonError<RwLockWriteGuard<AnyMap<ID>>>>
  where
    ID: From<D::ID>,
  {
    drops.into_iter().map(|drop| self.normalize(drop)).collect()
  }

  pub fn normalize_all_refs<
    'a,
    D: Clone + LiquidDrop + LiquidDropWithID + Send + Sync + 'static,
    I: IntoIterator<Item = &'a D>,
  >(
    &'a self,
    drop_refs: I,
  ) -> Result<Vec<ArcValueView<D>>, PoisonError<()>>
  where
    ID: From<D::ID>,
  {
    drop_refs
      .into_iter()
      .map(|drop_ref| self.normalize_ref(drop_ref))
      .collect()
  }

  fn get_or_insert_with<
    D: LiquidDrop + LiquidDropWithID + Send + Sync + 'static,
    F: FnOnce() -> D,
  >(
    &self,
    id: ID,
    init: F,
  ) -> Result<ArcValueView<D>, PoisonError<()>>
  where
    ID: From<D::ID>,
  {
    match self.get(id).map_err(|_| PoisonError::new(()))? {
      Some(drop) => Ok(drop),
      None => {
        let drop = init();
        warn!(
          "NormalizedDropCache miss on {} {}",
          std::any::type_name::<D>(),
          drop.id()
        );
        self.get_or_insert(drop).map_err(|_| PoisonError::new(()))
      }
    }
  }

  fn get_or_insert<D: LiquidDrop + LiquidDropWithID + Send + Sync + 'static>(
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
