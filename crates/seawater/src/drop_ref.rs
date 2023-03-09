use parking_lot::MappedRwLockReadGuard;

use crate::DropStore;
use crate::LiquidDrop;
use std::fmt::Display;
use std::hash::Hash;
use std::{fmt::Debug, marker::PhantomData};

pub struct DropRef<
  'store,
  D: LiquidDrop + Clone + 'static,
  StoreID: From<D::ID> + Hash + Eq + Display + Copy + Debug = <D as LiquidDrop>::ID,
> {
  id: D::ID,
  store: &'store DropStore<StoreID>,
  _phantom: PhantomData<D>,
}

impl<
    'store,
    D: LiquidDrop + Clone + 'static,
    StoreID: From<D::ID> + Hash + Eq + Display + Copy + Debug,
  > DropRef<'store, D, StoreID>
{
  pub fn new(id: D::ID, store: &'store DropStore<StoreID>) -> Self {
    DropRef {
      id,
      store,
      _phantom: Default::default(),
    }
  }

  pub fn fetch(&self) -> MappedRwLockReadGuard<D> {
    self.try_fetch().unwrap()
  }

  pub fn try_fetch(&self) -> Option<MappedRwLockReadGuard<D>> {
    self.store.get(self.id.into())
  }

  pub fn id(&self) -> D::ID {
    self.id
  }
}

impl<
    'store,
    D: LiquidDrop + Clone + 'static,
    StoreID: From<D::ID> + Hash + Eq + Display + Copy + Debug,
  > Clone for DropRef<'store, D, StoreID>
{
  fn clone(&self) -> Self {
    Self {
      id: self.id,
      store: self.store,
      _phantom: Default::default(),
    }
  }
}

impl<
    'store,
    D: LiquidDrop + Clone + 'static,
    StoreID: From<D::ID> + Hash + Eq + Display + Copy + Debug,
  > Debug for DropRef<'store, D, StoreID>
where
  D::ID: Debug,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DropRef")
      .field("id", &self.id)
      .field("type", &std::any::type_name::<D>())
      .finish_non_exhaustive()
  }
}
