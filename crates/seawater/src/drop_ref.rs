use crate::DropStore;
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
pub struct DropRef<D: LiquidDrop + Clone + 'static, StoreID: Eq + Hash + Copy + Display + Debug> {
  id: StoreID,
  store: Weak<DropStore<StoreID>>,
  _phantom: PhantomData<D>,
}

impl<
    D: LiquidDrop + Clone + Send + Sync + 'static,
    StoreID: Eq + Hash + Copy + Send + Sync + Display + Debug,
  > DropRef<D, StoreID>
{
  pub fn new(id: StoreID, store: Weak<DropStore<StoreID>>) -> Self {
    DropRef {
      id,
      store,
      _phantom: Default::default(),
    }
  }

  pub fn fetch(&self) -> Arc<D> {
    self.try_fetch().unwrap()
  }

  pub fn try_fetch(&self) -> Option<Arc<D>> {
    let arc = self.store.upgrade();
    let store = arc.as_deref();
    store.and_then(|store| store.get::<D>(self.id))
  }

  pub fn id(&self) -> StoreID {
    self.id
  }
}

impl<D: LiquidDrop + Clone + 'static, StoreID: Eq + Hash + Copy + Send + Sync + Display + Debug>
  Debug for DropRef<D, StoreID>
where
  StoreID: Debug,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DropRef")
      .field("id", &self.id)
      .field("type", &std::any::type_name::<D>())
      .finish_non_exhaustive()
  }
}

impl<
    D: LiquidDrop + Clone + Send + Sync + 'static,
    StoreID: Eq + Hash + Copy + Send + Sync + Display + Debug + 'static,
  > IntoDropResult<D> for DropRef<D, StoreID>
{
}
