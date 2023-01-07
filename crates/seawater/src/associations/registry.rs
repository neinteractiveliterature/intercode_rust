use std::{
  any::{Any, TypeId},
  collections::HashMap,
  sync::{Arc, RwLock},
};

use lazy_liquid_value_view::LiquidDropCache;
use once_cell::sync::Lazy;

use crate::{DropPrimaryKeyValue, ModelBackedDrop};

use super::{elided_model::ElidedModel, ModelBackedDropAssociation};

struct BoxLiquidDropCache(Box<dyn LiquidDropCache>);

impl LiquidDropCache for BoxLiquidDropCache {
  fn get_once_cell<T>(&self, field_name: &str) -> Option<&once_cell::race::OnceBox<T>>
  where
    Self: Sized,
  {
    self.0.get_once_cell::<T>(field_name)
  }
}

pub type BoxAssociation<FromDrop: ModelBackedDrop, Value, Context> = Box<
  dyn ModelBackedDropAssociation<
      FromDrop = FromDrop,
      ToDrop = dyn ModelBackedDrop<
        Cache = BoxLiquidDropCache,
        Model = ElidedModel,
        Context = Context,
      >,
      Value = Value,
      Preloader = dyn Send + Sync,
      PrimaryKeyValue = DropPrimaryKeyValue<FromDrop>,
    > + Send
    + Sync,
>;

#[derive(Default)]
pub struct AssociationRegistry {
  registry: Arc<RwLock<HashMap<TypeId, HashMap<String, Box<dyn Any + Send + Sync + 'static>>>>>,
}

impl AssociationRegistry {
  pub fn get_association<FromDrop: ModelBackedDrop, Value, Context>(
    &self,
    name: &str,
  ) -> Option<&BoxAssociation<FromDrop, Value, Context>> {
    let lock = self.registry.read().unwrap();
    lock
      .get(&TypeId::of::<FromDrop>())
      .and_then(|type_associations| {
        type_associations
          .get(name)
          .and_then(|assoc| assoc.downcast_ref())
      })
  }

  pub fn set_association<A: Send + Sync>(&self, name: String, value: A)
  where
    A: ModelBackedDropAssociation,
  {
    let mut lock = self.registry.write().unwrap();
    let mut type_entry = lock.entry(TypeId::of::<A::FromDrop>());
    let mut association_registry = type_entry.or_insert_with(|| Default::default());
    association_registry.insert(name, Box::new(value));
  }
}

pub static ASSOCIATION_REGISTRY: Lazy<AssociationRegistry> = Lazy::new(|| Default::default());
