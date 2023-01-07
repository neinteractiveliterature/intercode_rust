use std::{fmt::Debug, hash::Hash};

use lazy_liquid_value_view::{DropResult, LiquidDrop, LiquidDropCache, LiquidDropWithID};
use liquid::ValueView;
use once_cell::race::OnceBox;
use sea_orm::Related;

use crate::{
  loaders::{AssociationLoaderResult, EntityRelationLoaderResult},
  preloaders::{EntityRelationPreloader, PreloaderBuilder},
  Context, ContextContainer, DropEntity, DropError, DropModel, DropPrimaryKey, DropPrimaryKeyValue,
  ModelBackedDrop,
};

use super::{value_adapters::ModelBackedDropAssociationValueAdapter, AssociationMeta};

pub trait ModelBackedDropAssociationAssociationTypeAdapter: Debug {
  type FromDrop: ModelBackedDrop + ContextContainer;
  type ToDrop: ModelBackedDrop<Context = <Self::FromDrop as ContextContainer>::Context>;
  type Preloader;
  type Value: ValueView;

  fn preloader(&self, context: <Self::FromDrop as ContextContainer>::Context) -> Self::Preloader;
  fn get_meta(&self) -> &AssociationMeta<Self::FromDrop, Self::ToDrop>;

  fn loader_result_to_drops(
    &self,
    result: Option<
      &dyn AssociationLoaderResult<DropModel<Self::FromDrop>, DropModel<Self::ToDrop>>,
    >,
    from_drop: &Self::FromDrop,
  ) -> Result<Vec<Self::ToDrop>, DropError> {
    result
      .map(|result| {
        Ok(
          result
            .get_models()
            .iter()
            .map(|model| Self::ToDrop::new(model.clone(), from_drop.get_context().clone()))
            .collect(),
        )
      })
      .unwrap_or_else(|| Ok(vec![]))
  }

  fn get_once_cell(
    &self,
    cache: &<Self::FromDrop as LiquidDrop>::Cache,
  ) -> &OnceBox<DropResult<Self::Value>> {
    cache.get_once_cell(&self.get_meta().field_name).unwrap()
  }
}

#[derive(Debug)]
pub struct ModelBackedDropRelatedAssociationTypeAdapter<
  'assoc,
  C: Context,
  FromDrop: ModelBackedDrop + LiquidDropWithID + ContextContainer<Context = C> + Send + Sync,
  ToDrop: ModelBackedDrop + ContextContainer<Context = C> + Send + Sync,
  Value: ValueView + Into<DropResult<Value>>,
> {
  pk: DropPrimaryKey<FromDrop>,
  value_adapter: &'assoc dyn ModelBackedDropAssociationValueAdapter<
    Id = DropPrimaryKeyValue<FromDrop>,
    ToDrop = ToDrop,
    Value = Value,
  >,
  meta: &'assoc AssociationMeta<FromDrop, ToDrop>,
}

impl<
    'assoc,
    C: Context,
    FromDrop: ModelBackedDrop + LiquidDropWithID + ContextContainer<Context = C> + Send + Sync,
    ToDrop: ModelBackedDrop + ContextContainer<Context = C> + Send + Sync,
    Value: ValueView + Into<DropResult<Value>>,
  > ModelBackedDropRelatedAssociationTypeAdapter<'assoc, C, FromDrop, ToDrop, Value>
{
}

impl<
    'assoc,
    C: Context,
    FromDrop: ModelBackedDrop + LiquidDropWithID + ContextContainer<Context = C> + Send + Sync,
    ToDrop: ModelBackedDrop + ContextContainer<Context = C> + Send + Sync,
    Value: ValueView + Into<DropResult<Value>> + Clone,
  > ModelBackedDropAssociationAssociationTypeAdapter
  for ModelBackedDropRelatedAssociationTypeAdapter<'assoc, C, FromDrop, ToDrop, Value>
where
  DropEntity<FromDrop>: Related<DropEntity<ToDrop>>,
  DropPrimaryKeyValue<FromDrop>: Eq + Clone + From<i64> + Hash + Sync + From<FromDrop::ID>,
{
  type FromDrop = FromDrop;
  type ToDrop = ToDrop;
  type Value = Value;

  type Preloader = EntityRelationPreloader<
    DropEntity<FromDrop>,
    DropEntity<ToDrop>,
    DropPrimaryKey<FromDrop>,
    FromDrop,
    ToDrop,
    Value,
    FromDrop::Context,
  >;

  fn preloader(&self, context: <Self::FromDrop as ContextContainer>::Context) -> Self::Preloader {
    let mut builder = FromDrop::relation_preloader::<ToDrop, Value>(self.pk)
      .with_context(context)
      .with_id_getter(|drop: &FromDrop| drop.id().into())
      .with_loader_result_to_drops(
        |result: Option<&EntityRelationLoaderResult<DropModel<FromDrop>, DropModel<ToDrop>>>,
         from_drop| {
          self.loader_result_to_drops(
            result.map(|res| {
              res as &dyn AssociationLoaderResult<DropModel<FromDrop>, DropModel<ToDrop>>
            }),
            from_drop,
          )
        },
      )
      .with_drops_to_value(|drops| self.value_adapter.drops_to_value(drops))
      .with_once_cell_getter(|cache| self.get_once_cell(cache));

    if let Some(inverse_field_name) = self.meta.inverse_field_name {
      builder = builder.with_inverse_once_cell_getter(|to_drop: &ToDrop| {
        to_drop
          .get_cache()
          .get_once_cell(&inverse_field_name)
          .unwrap()
      });
    }
    builder.finalize().unwrap()
  }

  fn get_meta(&self) -> &AssociationMeta<Self::FromDrop, Self::ToDrop> {
    self.meta
  }
}
