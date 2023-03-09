#![allow(clippy::type_complexity)]
use std::{collections::HashMap, fmt::Display, marker::PhantomData, pin::Pin};

use crate::{
  loaders::{load_all_related, EntityRelationLoaderResult},
  ConnectionWrapper, DropRef, DropResult, DropResultTrait, LiquidDrop,
};
use async_trait::async_trait;
use liquid::ValueView;
use once_cell::race::OnceBox;
use sea_orm::{EntityTrait, PrimaryKeyToColumn, PrimaryKeyTrait, Related};

use crate::{
  DropEntity, DropError, DropPrimaryKey, DropPrimaryKeyValue, DropStore, ModelBackedDrop,
};

use super::{
  DropsToValueFn, GetInverseOnceCellFn, GetOnceCellFn, IncompleteBuilderError,
  LoaderResultToDropsFn, Preloader, PreloaderBuilder,
};

pub struct EntityRelationPreloader<
  From: EntityTrait<PrimaryKey = PK> + Related<To>,
  To: EntityTrait,
  PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
  FromDrop: LiquidDrop + Send + Sync + Clone + 'static,
  ToDrop: LiquidDrop + Send + Sync + Clone + 'static,
  Value: ValueView + Clone + Into<DropResult<Value>>,
  Context: crate::Context,
> where
  PK::Column: Send + Sync,
  PK::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
{
  pk_column: PK::Column,
  context: Context,
  loader_result_to_drops:
    Pin<Box<dyn LoaderResultToDropsFn<EntityRelationLoaderResult<From, To>, FromDrop, ToDrop>>>,
  drops_to_value: Pin<Box<dyn DropsToValueFn<ToDrop, Value>>>,
  get_once_cell: Pin<Box<dyn for<'a> GetOnceCellFn<'a, FromDrop, Value>>>,
  get_inverse_once_cell: Option<Pin<Box<dyn for<'a> GetInverseOnceCellFn<'a, FromDrop, ToDrop>>>>,
  _phantom: PhantomData<(From, To, FromDrop, ToDrop)>,
}

#[async_trait]
impl<
    From: EntityTrait<PrimaryKey = PK> + Related<To>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
    FromDrop: LiquidDrop<ID = PK::ValueType> + Send + Sync + Clone,
    ToDrop: LiquidDrop<ID = PK::ValueType> + Send + Sync + Clone + 'static,
    Value: ValueView + Into<DropResult<Value>> + DropResultTrait<Value> + Clone + Send + Sync + 'static,
    Context: crate::Context,
  > Preloader<FromDrop, ToDrop, PK::ValueType, Value>
  for EntityRelationPreloader<From, To, PK, FromDrop, ToDrop, Value, Context>
where
  PK::ValueType:
    Eq + std::hash::Hash + Copy + Clone + std::convert::From<i64> + Send + Sync + Display,
  To::Model: Send + Sync,
  i64: std::convert::From<ToDrop::ID>,
{
  type LoaderResult = EntityRelationLoaderResult<From, To>;

  fn loader_result_to_drops(
    &self,
    result: Option<Self::LoaderResult>,
    drop: DropRef<FromDrop>,
  ) -> Result<Vec<ToDrop>, DropError> {
    (self.loader_result_to_drops)(result, drop)
  }

  fn with_drop_store<'store, R, F: FnOnce(&'store DropStore<PK::ValueType>) -> R>(
    &self,
    f: F,
  ) -> R {
    self.context.with_drop_store(f)
  }

  fn drops_to_value(&self, drops: Vec<DropRef<ToDrop>>) -> Result<DropResult<Value>, DropError> {
    (self.drops_to_value)(drops)
  }

  fn get_once_cell<'a>(&'a self, cache: &'a FromDrop::Cache) -> &'a OnceBox<DropResult<Value>> {
    (self.get_once_cell)(cache)
  }

  fn get_inverse_once_cell<'a>(
    &'a self,
    cache: &'a ToDrop::Cache,
  ) -> Option<&'a OnceBox<DropResult<FromDrop>>> {
    self
      .get_inverse_once_cell
      .as_ref()
      .map(|get_inverse_once_cell| (get_inverse_once_cell)(cache))
  }

  async fn load_model_lists(
    &self,
    db: &ConnectionWrapper,
    ids: &[PK::ValueType],
  ) -> Result<HashMap<PK::ValueType, Self::LoaderResult>, DropError> {
    load_all_related::<From, To>(self.pk_column, ids, db)
      .await
      .map_err(|err| err.into())
  }
}

pub struct EntityRelationPreloaderBuilder<
  FromDrop: ModelBackedDrop,
  ToDrop: ModelBackedDrop,
  Value: ValueView + Clone,
  Context: crate::Context,
> where
  FromDrop: Send + Sync + Clone + 'static,
  ToDrop: Send + Sync + Clone + 'static,
  DropEntity<FromDrop>: Related<DropEntity<ToDrop>>,
  DropPrimaryKeyValue<FromDrop>:
    Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
  Value: Into<DropResult<Value>>,
  DropPrimaryKeyValue<FromDrop>: Clone,
{
  pk_column: DropPrimaryKey<FromDrop>,
  context: Option<Context>,
  loader_result_to_drops: Option<
    Pin<
      Box<
        dyn LoaderResultToDropsFn<
          EntityRelationLoaderResult<DropEntity<FromDrop>, DropEntity<ToDrop>>,
          FromDrop,
          ToDrop,
        >,
      >,
    >,
  >,
  drops_to_value: Option<Pin<Box<dyn DropsToValueFn<ToDrop, Value>>>>,
  get_once_cell: Option<Pin<Box<dyn for<'a> GetOnceCellFn<'a, FromDrop, Value>>>>,
  get_inverse_once_cell: Option<Pin<Box<dyn for<'a> GetInverseOnceCellFn<'a, FromDrop, ToDrop>>>>,
  _phantom: PhantomData<(FromDrop, ToDrop, Value)>,
}

impl<
    FromDrop: ModelBackedDrop,
    ToDrop: ModelBackedDrop,
    Value: ValueView + Clone,
    Context: crate::Context,
  > PreloaderBuilder for EntityRelationPreloaderBuilder<FromDrop, ToDrop, Value, Context>
where
  FromDrop: Send + Sync + Clone,
  ToDrop: Send + Sync + Clone,
  DropEntity<FromDrop>: Related<DropEntity<ToDrop>>,
  Value: Into<DropResult<Value>>,
  DropPrimaryKeyValue<FromDrop>:
    Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
{
  type Preloader = EntityRelationPreloader<
    DropEntity<FromDrop>,
    DropEntity<ToDrop>,
    DropPrimaryKey<FromDrop>,
    FromDrop,
    ToDrop,
    Value,
    Context,
  >;
  type Error = IncompleteBuilderError;

  fn finalize(self) -> Result<Self::Preloader, Self::Error> {
    if let (
      Some(context),
      Some(loader_result_to_drops),
      Some(drops_to_value),
      Some(get_once_cell),
    ) = (
      self.context,
      self.loader_result_to_drops,
      self.drops_to_value,
      self.get_once_cell,
    ) {
      Ok(EntityRelationPreloader {
        pk_column: self.pk_column.into_column(),
        context,
        loader_result_to_drops,
        drops_to_value,
        get_once_cell,
        get_inverse_once_cell: self.get_inverse_once_cell,
        _phantom: Default::default(),
      })
    } else {
      Err(IncompleteBuilderError)
    }
  }
}

impl<
    FromDrop: ModelBackedDrop,
    ToDrop: ModelBackedDrop,
    Value: ValueView + Clone,
    Context: crate::Context,
  > EntityRelationPreloaderBuilder<FromDrop, ToDrop, Value, Context>
where
  FromDrop: Send + Sync + Clone,
  ToDrop: Send + Sync + Clone + 'static,
  DropEntity<FromDrop>: Related<DropEntity<ToDrop>>,
  Value: Into<DropResult<Value>>,
  DropPrimaryKeyValue<FromDrop>:
    Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
{
  pub fn new(pk_column: DropPrimaryKey<FromDrop>) -> Self {
    EntityRelationPreloaderBuilder {
      pk_column,
      context: None,
      loader_result_to_drops: None,
      drops_to_value: None,
      get_once_cell: None,
      get_inverse_once_cell: None,
      _phantom: Default::default(),
    }
  }

  pub fn with_context(mut self, context: Context) -> Self {
    self.context = Some(context);
    self
  }

  pub fn with_loader_result_to_drops<F>(mut self, loader_result_to_drops: F) -> Self
  where
    F: LoaderResultToDropsFn<
        EntityRelationLoaderResult<DropEntity<FromDrop>, DropEntity<ToDrop>>,
        FromDrop,
        ToDrop,
      > + 'static,
  {
    self.loader_result_to_drops = Some(Box::pin(loader_result_to_drops));
    self
  }

  pub fn with_drops_to_value<F>(mut self, drops_to_value: F) -> Self
  where
    F: DropsToValueFn<ToDrop, Value> + 'static,
  {
    self.drops_to_value = Some(Box::pin(drops_to_value));
    self
  }

  pub fn with_once_cell_getter<F>(mut self, get_once_cell: F) -> Self
  where
    F: for<'a> GetOnceCellFn<'a, FromDrop, Value> + 'static,
  {
    self.get_once_cell = Some(Box::pin(get_once_cell));
    self
  }

  pub fn with_inverse_once_cell_getter<F>(mut self, get_inverse_once_cell: F) -> Self
  where
    F: for<'a> GetInverseOnceCellFn<'a, FromDrop, ToDrop> + 'static,
  {
    self.get_inverse_once_cell = Some(Box::pin(get_inverse_once_cell));
    self
  }
}
