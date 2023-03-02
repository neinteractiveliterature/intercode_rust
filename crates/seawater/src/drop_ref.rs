use crate::{ArcValueView, LiquidDrop, LiquidDropWithID};
use crate::{DropError, DropStore};
use std::hash::Hash;
use std::{fmt::Debug, marker::PhantomData};

pub struct DropRef<
  'store,
  D: LiquidDrop + LiquidDropWithID + 'static,
  StoreID: From<D::ID> + Hash + Eq + Copy = <D as LiquidDropWithID>::ID,
> {
  id: D::ID,
  store: &'store DropStore<StoreID>,
  _phantom: PhantomData<D>,
}

impl<'store, D: LiquidDrop + LiquidDropWithID + 'static, StoreID: From<D::ID> + Hash + Eq + Copy>
  DropRef<'store, D, StoreID>
{
  pub fn new(id: D::ID, store: &'store DropStore<StoreID>) -> Self {
    DropRef {
      id,
      store,
      _phantom: Default::default(),
    }
  }

  pub fn fetch(&self) -> ArcValueView<D> {
    self.try_fetch().unwrap()
  }

  pub fn try_fetch(&self) -> Result<ArcValueView<D>, DropError> {
    Ok(self.store.get(self.id.into())?.unwrap())
  }

  pub fn id(&self) -> D::ID {
    self.id
  }
}

impl<'store, D: LiquidDrop + LiquidDropWithID + 'static, StoreID: From<D::ID> + Hash + Eq + Copy>
  Clone for DropRef<'store, D, StoreID>
{
  fn clone(&self) -> Self {
    Self {
      id: self.id,
      store: self.store.clone(),
      _phantom: Default::default(),
    }
  }
}

impl<'store, D: LiquidDrop + LiquidDropWithID + 'static, StoreID: From<D::ID> + Hash + Eq + Copy>
  Debug for DropRef<'store, D, StoreID>
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
