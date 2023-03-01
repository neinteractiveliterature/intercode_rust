use crate::NormalizedDropCache;
use crate::{ArcValueView, LiquidDrop, LiquidDropWithID};
use std::{fmt::Debug, hash::Hash, marker::PhantomData};

pub struct DropRef<D: LiquidDrop + LiquidDropWithID + 'static> {
  id: D::ID,
  _phantom: PhantomData<D>,
}

impl<D: LiquidDrop + LiquidDropWithID + 'static> DropRef<D> {
  pub fn fetch<CacheID: From<D::ID> + Eq + Hash + Copy>(
    &self,
    cache: &NormalizedDropCache<CacheID>,
  ) -> ArcValueView<D> {
    cache.get(self.id.into()).unwrap().unwrap()
  }

  pub fn id(&self) -> D::ID {
    self.id
  }
}

impl<D: LiquidDrop + LiquidDropWithID + 'static> Clone for DropRef<D> {
  fn clone(&self) -> Self {
    Self {
      id: self.id,
      _phantom: Default::default(),
    }
  }
}

impl<D: LiquidDrop + LiquidDropWithID + 'static> Copy for DropRef<D> {}

impl<D: LiquidDrop + LiquidDropWithID + 'static> From<D> for DropRef<D> {
  fn from(value: D) -> Self {
    DropRef {
      id: value.id(),
      _phantom: Default::default(),
    }
  }
}

impl<D: LiquidDrop + LiquidDropWithID + 'static> From<&D> for DropRef<D> {
  fn from(value: &D) -> Self {
    DropRef {
      id: value.id(),
      _phantom: Default::default(),
    }
  }
}

impl<D: LiquidDrop + LiquidDropWithID + 'static> From<ArcValueView<D>> for DropRef<D> {
  fn from(value: ArcValueView<D>) -> Self {
    DropRef {
      id: value.id(),
      _phantom: Default::default(),
    }
  }
}

impl<D: LiquidDrop + LiquidDropWithID + 'static> Debug for DropRef<D>
where
  D::ID: Debug,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DropRef")
      .field("id", &self.id)
      .field("type", &std::any::type_name::<D>())
      .finish()
  }
}
