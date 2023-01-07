use crate::{
  preloaders::PreloaderResult, pretty_type_name, DropError, DropPrimaryKeyValue, ModelBackedDrop,
};
use lazy_liquid_value_view::{ArcValueView, DropResult};
use liquid::ValueView;
use std::{fmt::Debug, hash::Hash, marker::PhantomData};

pub trait ModelBackedDropAssociationValueAdapter: Debug + Send + Sync {
  type Id: Eq + Hash;
  type ToDrop: ModelBackedDrop;
  type Value: ValueView + Clone;

  fn preloader_result_to_drops(
    &self,
    preloader_result: &PreloaderResult<Self::Id, Self::Value>,
  ) -> Vec<&Self::ToDrop>;

  fn drops_to_value(
    &self,
    drops: Vec<ArcValueView<Self::ToDrop>>,
  ) -> Result<DropResult<Self::Value>, DropError>;
}

#[derive(Debug)]
pub struct ModelBackedDropAssociationOneOptionalAdapter<
  FromDrop: ModelBackedDrop,
  ToDrop: ModelBackedDrop,
> {
  _phantom: PhantomData<(FromDrop, ToDrop)>,
}

impl<FromDrop: ModelBackedDrop + Send + Sync, ToDrop: ModelBackedDrop + Send + Sync>
  ModelBackedDropAssociationValueAdapter
  for ModelBackedDropAssociationOneOptionalAdapter<FromDrop, ToDrop>
where
  DropPrimaryKeyValue<FromDrop>: Eq + Hash,
{
  type Id = DropPrimaryKeyValue<FromDrop>;
  type ToDrop = ToDrop;
  type Value = ArcValueView<ToDrop>;

  fn preloader_result_to_drops(
    &self,
    preloader_result: &PreloaderResult<Self::Id, Self::Value>,
  ) -> Vec<&Self::ToDrop> {
    preloader_result
      .all_values()
      .filter_map(|v| v.get_inner())
      .map(|v| v.as_ref())
      .collect::<Vec<_>>()
  }

  fn drops_to_value(
    &self,
    drops: Vec<ArcValueView<Self::ToDrop>>,
  ) -> Result<DropResult<Self::Value>, DropError> {
    if drops.len() == 1 {
      Ok(drops[0].clone().into())
    } else {
      Ok(None::<ArcValueView<Self::ToDrop>>.into())
    }
  }
}

#[derive(Debug)]
pub struct ModelBackedDropAssociationOneRequiredAdapter<
  FromDrop: ModelBackedDrop,
  ToDrop: ModelBackedDrop,
> {
  _phantom: PhantomData<(FromDrop, ToDrop)>,
}

impl<FromDrop: ModelBackedDrop + Send + Sync, ToDrop: ModelBackedDrop + Send + Sync>
  ModelBackedDropAssociationValueAdapter
  for ModelBackedDropAssociationOneRequiredAdapter<FromDrop, ToDrop>
where
  DropPrimaryKeyValue<FromDrop>: Eq + Hash,
{
  type Id = DropPrimaryKeyValue<FromDrop>;
  type ToDrop = ToDrop;
  type Value = ArcValueView<ToDrop>;

  fn preloader_result_to_drops(
    &self,
    preloader_result: &PreloaderResult<Self::Id, Self::Value>,
  ) -> Vec<&Self::ToDrop> {
    preloader_result
      .all_values_unwrapped()
      .map(|v| v.as_ref())
      .collect::<Vec<_>>()
  }

  fn drops_to_value(
    &self,
    drops: Vec<ArcValueView<Self::ToDrop>>,
  ) -> Result<DropResult<Self::Value>, DropError> {
    if drops.len() == 1 {
      Ok(drops[0].clone().into())
    } else {
      Err(DropError::ExpectedEntityNotFound(format!(
        "Expected one {}, but there are {}",
        pretty_type_name::<ToDrop>(),
        drops.len()
      )))
    }
  }
}

#[derive(Debug)]
pub struct ModelBackedDropAssociationManyAdapter<FromDrop: ModelBackedDrop, ToDrop: ModelBackedDrop>
{
  _phantom: PhantomData<(FromDrop, ToDrop)>,
}

impl<FromDrop: ModelBackedDrop + Send + Sync, ToDrop: ModelBackedDrop + Clone + Send + Sync>
  ModelBackedDropAssociationValueAdapter for ModelBackedDropAssociationManyAdapter<FromDrop, ToDrop>
where
  DropPrimaryKeyValue<FromDrop>: Eq + Hash,
{
  type Id = DropPrimaryKeyValue<FromDrop>;
  type ToDrop = ToDrop;
  type Value = Vec<ArcValueView<ToDrop>>;

  fn preloader_result_to_drops(
    &self,
    preloader_result: &PreloaderResult<Self::Id, Self::Value>,
  ) -> Vec<&Self::ToDrop> {
    preloader_result
      .all_values_flat_unwrapped()
      .collect::<Vec<_>>()
  }

  fn drops_to_value(
    &self,
    drops: Vec<ArcValueView<Self::ToDrop>>,
  ) -> Result<DropResult<Self::Value>, DropError> {
    Ok(drops.into())
  }
}
