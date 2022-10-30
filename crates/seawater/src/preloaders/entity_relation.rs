use std::{collections::HashMap, marker::PhantomData, pin::Pin};

use async_trait::async_trait;
use intercode_graphql::loaders::{load_all_related, EntityRelationLoaderResult};
use lazy_liquid_value_view::{ArcValueView, DropResult, LiquidDrop, LiquidDropWithID};
use liquid::ValueView;
use once_cell::race::OnceBox;
use sea_orm::{DatabaseConnection, EntityTrait, PrimaryKeyToColumn, PrimaryKeyTrait, Related};

use crate::{
  DropEntity, DropError, DropPrimaryKey, DropPrimaryKeyValue, ModelBackedDrop, NormalizedDropCache,
};

use super::{
  DropsToValueFn, GetIdFn, GetInverseOnceCellFn, GetOnceCellFn, IncompleteBuilderError,
  LoaderResultToDropsFn, Preloader, PreloaderBuilder,
};

pub struct EntityRelationPreloader<
  From: EntityTrait<PrimaryKey = PK> + Related<To>,
  To: EntityTrait,
  PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
  FromDrop: LiquidDrop + Send + Sync,
  ToDrop: LiquidDrop + Send + Sync,
  Value: ValueView + Into<DropResult<Value>>,
  Context: crate::Context,
> where
  PK::Column: Send + Sync,
  PK::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
{
  pk_column: PK::Column,
  context: Context,
  get_id: Pin<Box<dyn for<'a> GetIdFn<'a, PK::ValueType, FromDrop>>>,
  loader_result_to_drops: Pin<
    Box<
      dyn for<'a> LoaderResultToDropsFn<'a, EntityRelationLoaderResult<From, To>, FromDrop, ToDrop>,
    >,
  >,
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
    FromDrop: LiquidDrop + LiquidDropWithID + Send + Sync + Clone,
    ToDrop: LiquidDrop + Send + Sync + Clone + LiquidDropWithID + 'static,
    Value: ValueView + Into<DropResult<Value>> + Clone + Send + Sync,
    Context: crate::Context,
  > Preloader<FromDrop, ToDrop, PK::ValueType, Value>
  for EntityRelationPreloader<From, To, PK, FromDrop, ToDrop, Value, Context>
where
  PK::ValueType: Eq + std::hash::Hash + Copy + Clone + std::convert::From<i64> + Send + Sync,
  To::Model: Send + Sync,
  i64: std::convert::From<ToDrop::ID>,
{
  type LoaderResult = EntityRelationLoaderResult<From, To>;

  fn get_id(&self, drop: &FromDrop) -> PK::ValueType {
    (self.get_id)(drop)
  }

  fn loader_result_to_drops(
    &self,
    result: Option<&Self::LoaderResult>,
    drop: &FromDrop,
  ) -> Result<Vec<ToDrop>, DropError> {
    (self.loader_result_to_drops)(result, drop)
  }

  fn get_normalized_drop_cache(&self) -> Option<&NormalizedDropCache<i64>> {
    Some(self.context.drop_cache())
  }

  fn drops_to_value(
    &self,
    drops: Vec<ArcValueView<ToDrop>>,
  ) -> Result<DropResult<Value>, DropError> {
    (self.drops_to_value)(drops)
  }

  fn get_once_cell<'a>(&'a self, cache: &'a FromDrop::Cache) -> &'a OnceBox<DropResult<Value>> {
    (self.get_once_cell)(cache)
  }

  fn get_inverse_once_cell<'a>(
    &'a self,
    drop: &'a ToDrop,
  ) -> Option<&'a OnceBox<DropResult<FromDrop>>> {
    self
      .get_inverse_once_cell
      .as_ref()
      .map(|get_inverse_once_cell| (get_inverse_once_cell)(drop))
  }

  async fn load_model_lists(
    &self,
    db: &DatabaseConnection,
    ids: &[PK::ValueType],
  ) -> Result<HashMap<PK::ValueType, Self::LoaderResult>, DropError> {
    load_all_related::<From, To, PK>(self.pk_column, ids, db)
      .await
      .map_err(|err| err.into())
  }
}

pub struct EntityRelationPreloaderBuilder<
  FromDrop: ModelBackedDrop,
  ToDrop: ModelBackedDrop,
  Value: ValueView,
  Context: crate::Context,
> where
  FromDrop: Send + Sync,
  ToDrop: Send + Sync,
  DropEntity<FromDrop>: Related<DropEntity<ToDrop>>,
  DropPrimaryKeyValue<FromDrop>:
    Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
  Value: Into<DropResult<Value>>,
  DropPrimaryKeyValue<FromDrop>: Clone,
{
  pk_column: DropPrimaryKey<FromDrop>,
  context: Option<Context>,
  get_id: Option<Pin<Box<dyn for<'a> GetIdFn<'a, DropPrimaryKeyValue<FromDrop>, FromDrop>>>>,
  loader_result_to_drops: Option<
    Pin<
      Box<
        dyn for<'a> LoaderResultToDropsFn<
          'a,
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
    Value: ValueView,
    Context: crate::Context,
  > PreloaderBuilder for EntityRelationPreloaderBuilder<FromDrop, ToDrop, Value, Context>
where
  FromDrop: Send + Sync,
  ToDrop: Send + Sync,
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
      Some(get_id),
      Some(loader_result_to_drops),
      Some(drops_to_value),
      Some(get_once_cell),
    ) = (
      self.context,
      self.get_id,
      self.loader_result_to_drops,
      self.drops_to_value,
      self.get_once_cell,
    ) {
      Ok(EntityRelationPreloader {
        pk_column: self.pk_column.into_column(),
        context,
        get_id,
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
    Value: ValueView,
    Context: crate::Context,
  > EntityRelationPreloaderBuilder<FromDrop, ToDrop, Value, Context>
where
  FromDrop: Send + Sync,
  ToDrop: Send + Sync,
  DropEntity<FromDrop>: Related<DropEntity<ToDrop>>,
  Value: Into<DropResult<Value>>,
  DropPrimaryKeyValue<FromDrop>:
    Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
{
  pub fn new(pk_column: DropPrimaryKey<FromDrop>) -> Self {
    EntityRelationPreloaderBuilder {
      pk_column,
      get_id: None,
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

  pub fn with_id_getter<F>(mut self, get_id: F) -> Self
  where
    F: for<'a> GetIdFn<'a, DropPrimaryKeyValue<FromDrop>, FromDrop> + 'static,
  {
    self.get_id = Some(Box::pin(get_id));
    self
  }

  pub fn with_loader_result_to_drops<F>(mut self, loader_result_to_drops: F) -> Self
  where
    F: for<'a> LoaderResultToDropsFn<
        'a,
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