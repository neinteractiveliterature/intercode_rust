use crate::DropStore;
use crate::DropStoreID;
use crate::IntoDropResult;
use crate::LiquidDrop;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::Weak;
use std::{fmt::Debug, marker::PhantomData};

pub(crate) trait StoreRef {
  type StoreID;

  fn get<ID: Into<Self::StoreID>, D: LiquidDrop + Send + Sync + 'static>(
    &self,
    id: ID,
  ) -> Option<Arc<D>>;
}

impl<StoreID: Eq + Hash + Copy + Display + Debug> StoreRef for Weak<DropStore<StoreID>> {
  type StoreID = StoreID;

  fn get<ID: Into<Self::StoreID>, D: LiquidDrop + Send + Sync + 'static>(
    &self,
    id: ID,
  ) -> Option<Arc<D>> {
    let arc = self.upgrade();
    let store = arc.as_deref();
    store.and_then(|store| store.get::<D>(id.into()))
  }
}

#[derive(Clone)]
pub struct DropRef<D: LiquidDrop + Clone + 'static> {
  id: DropStoreID<D>,
  store: Weak<DropStore<DropStoreID<D>>>,
  _phantom: PhantomData<D>,
}

impl<D: LiquidDrop + Clone + Send + Sync + 'static> DropRef<D> {
  pub fn new(id: DropStoreID<D>, store: Weak<DropStore<DropStoreID<D>>>) -> Self {
    DropRef {
      id,
      store,
      _phantom: Default::default(),
    }
  }

  pub fn fetch(&self) -> Arc<D> {
    self
      .try_fetch()
      .unwrap_or_else(|| panic!("Couldn't fetch {:?} from drop store", self))
  }

  pub fn try_fetch(&self) -> Option<Arc<D>> {
    let arc = self.store.upgrade();
    let store = arc.as_deref();
    store.and_then(|store| store.get::<D>(self.id))
  }

  pub fn id(&self) -> DropStoreID<D> {
    self.id
  }
}

impl<D: LiquidDrop + Clone + 'static> Debug for DropRef<D> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DropRef")
      .field("id", &self.id)
      .field("type", &std::any::type_name::<D>())
      .finish_non_exhaustive()
  }
}

impl<D: LiquidDrop + Clone + Send + Sync + 'static> IntoDropResult<D> for DropRef<D> {}
