#![allow(clippy::type_complexity)]
use std::{
  collections::HashMap,
  fmt::{Debug, Display},
  marker::PhantomData,
  pin::Pin,
};

use crate::{
  loaders::{load_all_linked, EntityLinkLoaderResult},
  ConnectionWrapper, DropRef, DropResult, DropResultTrait, IntoDropResult, LiquidDrop,
};
use async_trait::async_trait;
use liquid::ValueView;
use once_cell::race::OnceBox;
use parking_lot::MappedRwLockReadGuard;
use sea_orm::{EntityTrait, Linked, ModelTrait, PrimaryKeyToColumn, PrimaryKeyTrait, TryGetable};

use crate::{
  DropEntity, DropError, DropPrimaryKey, DropPrimaryKeyValue, DropStore, ModelBackedDrop,
};

use super::{
  DropsToValueFn, GetInverseOnceCellFn, GetOnceCellFn, IncompleteBuilderError,
  LoaderResultToDropsFn, Preloader, PreloaderBuilder,
};

pub struct EntityLinkPreloader<
  From: EntityTrait<PrimaryKey = PK>,
  To: EntityTrait,
  PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
  FromDrop: LiquidDrop + Clone + Send + Sync + 'static,
  ToDrop: LiquidDrop + Clone + Send + Sync + 'static,
  Value: ValueView + Clone,
  Context: crate::Context,
> where
  PK::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<Context::StoreID>,
  Value: IntoDropResult,
{
  pk_column: PK::Column,
  link: BoxLink<From, To>,
  context: Context,
  loader_result_to_drops: Pin<
    Box<
      dyn LoaderResultToDropsFn<
        EntityLinkLoaderResult<From, To>,
        FromDrop,
        ToDrop,
        Context::StoreID,
      >,
    >,
  >,
  drops_to_value: Pin<Box<dyn DropsToValueFn<ToDrop, Value, Context::StoreID>>>,
  get_once_cell: Pin<Box<dyn GetOnceCellFn<FromDrop, Value>>>,
  get_inverse_once_cell: Option<Pin<Box<dyn GetInverseOnceCellFn<FromDrop, ToDrop>>>>,
  _phantom: PhantomData<(From, To, FromDrop, ToDrop)>,
}

impl<
    From: EntityTrait<PrimaryKey = PK>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
    FromDrop: LiquidDrop + Clone + Send + Sync,
    ToDrop: LiquidDrop + Clone + Send + Sync,
    Value: ValueView + Clone,
    Context: crate::Context,
  > Debug for EntityLinkPreloader<From, To, PK, FromDrop, ToDrop, Value, Context>
where
  PK::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<Context::StoreID>,
  Value: IntoDropResult,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("EntityLinkPreloader")
      .field("pk_column", &self.pk_column)
      .field("_phantom", &self._phantom)
      .finish_non_exhaustive()
  }
}

#[async_trait]
impl<
    From: EntityTrait<PrimaryKey = PK>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
    FromDrop: LiquidDrop<ID = PK::ValueType> + Send + Sync + Clone,
    ToDrop: LiquidDrop<ID = PK::ValueType> + Send + Sync + Clone,
    Value: ValueView + Clone + Send + Sync + DropResultTrait<Value> + 'static,
    Context: crate::Context,
  > Preloader<FromDrop, ToDrop, Value, Context::StoreID>
  for EntityLinkPreloader<From, To, PK, FromDrop, ToDrop, Value, Context>
where
  PK::ValueType: Eq
    + std::hash::Hash
    + Clone
    + Send
    + Sync
    + Copy
    + Display
    + TryGetable
    + std::convert::From<Context::StoreID>,
  To::Model: Send + Sync,
  <To::PrimaryKey as PrimaryKeyTrait>::ValueType: Eq + std::hash::Hash + Clone + Send + Sync + Copy,
  Value: IntoDropResult,
  Context::StoreID: std::convert::From<PK::ValueType>,
{
  type LoaderResult = EntityLinkLoaderResult<From, To>;

  fn with_drop_store<'store, R, F: FnOnce(&'store DropStore<Context::StoreID>) -> R>(
    &self,
    f: F,
  ) -> R {
    self.context.with_drop_store(f)
  }

  fn loader_result_to_drops(
    &self,
    result: Option<Self::LoaderResult>,
    drop: DropRef<FromDrop, Context::StoreID>,
  ) -> Result<Vec<ToDrop>, DropError> {
    (self.loader_result_to_drops)(result, drop)
  }

  fn drops_to_value(
    &self,
    drops: Vec<DropRef<ToDrop, Context::StoreID>>,
  ) -> Result<DropResult<Value>, DropError> {
    (self.drops_to_value)(drops)
  }

  fn get_once_cell<'a>(
    &self,
    cache: MappedRwLockReadGuard<'a, FromDrop::Cache>,
  ) -> MappedRwLockReadGuard<'a, OnceBox<DropResult<Value>>> {
    (self.get_once_cell)(cache)
  }

  fn get_inverse_once_cell<'a>(
    &self,
    cache: MappedRwLockReadGuard<'a, ToDrop::Cache>,
  ) -> Option<MappedRwLockReadGuard<'a, OnceBox<DropResult<FromDrop>>>> {
    self
      .get_inverse_once_cell
      .as_ref()
      .map(|get_inverse_once_cell| (get_inverse_once_cell)(cache))
  }

  async fn load_model_lists(
    &self,
    db: &ConnectionWrapper,
    ids: &[Context::StoreID],
  ) -> Result<HashMap<Context::StoreID, Self::LoaderResult>, DropError> {
    let ids_converted: Vec<<From::PrimaryKey as PrimaryKeyTrait>::ValueType> =
      ids.iter().map(|id| (*id).into()).collect();
    load_all_linked(self.pk_column, &ids_converted, &self.link, db)
      .await
      .map(|result| {
        result
          .into_iter()
          .map(|(id, value)| (id.into(), value))
          .collect()
      })
      .map_err(|err| err.into())
  }
}

struct ModelBackedDropLink<FromDrop: ModelBackedDrop, ToDrop: ModelBackedDrop> {
  link: Box<
    dyn Linked<
        FromEntity = <FromDrop::Model as ModelTrait>::Entity,
        ToEntity = <ToDrop::Model as ModelTrait>::Entity,
      > + Send
      + Sync,
  >,
  _phantom: PhantomData<(FromDrop, ToDrop)>,
}

impl<FromDrop: ModelBackedDrop, ToDrop: ModelBackedDrop> Linked
  for ModelBackedDropLink<FromDrop, ToDrop>
{
  type FromEntity = DropEntity<FromDrop>;
  type ToEntity = DropEntity<ToDrop>;

  fn link(&self) -> Vec<sea_orm::LinkDef> {
    self.link.link()
  }
}

pub struct BoxLink<FromEntity, ToEntity>(
  Box<dyn Linked<FromEntity = FromEntity, ToEntity = ToEntity> + Send + Sync>,
);

impl<FromEntity: EntityTrait, ToEntity: EntityTrait> Linked for &BoxLink<FromEntity, ToEntity> {
  type FromEntity = FromEntity;
  type ToEntity = ToEntity;

  fn link(&self) -> Vec<sea_orm::LinkDef> {
    self.0.link()
  }
}

pub struct EntityLinkPreloaderBuilder<
  FromDrop: ModelBackedDrop,
  ToDrop: ModelBackedDrop,
  Value: ValueView + Clone,
  Context: crate::Context,
> where
  FromDrop: Send + Sync + Clone + 'static,
  ToDrop: Send + Sync + Clone + 'static,
  DropPrimaryKeyValue<FromDrop>: Eq + std::hash::Hash + Clone + Send + Sync,
  Value: IntoDropResult,
  DropPrimaryKeyValue<FromDrop>: Clone,
{
  link: ModelBackedDropLink<FromDrop, ToDrop>,
  context: Option<Context>,
  pk_column: <<FromDrop::Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey,
  loader_result_to_drops: Option<
    Pin<
      Box<
        dyn LoaderResultToDropsFn<
          EntityLinkLoaderResult<DropEntity<FromDrop>, DropEntity<ToDrop>>,
          FromDrop,
          ToDrop,
          Context::StoreID,
        >,
      >,
    >,
  >,
  drops_to_value: Option<Pin<Box<dyn DropsToValueFn<ToDrop, Value, Context::StoreID>>>>,
  get_once_cell: Option<Pin<Box<dyn GetOnceCellFn<FromDrop, Value>>>>,
  get_inverse_once_cell: Option<Pin<Box<dyn GetInverseOnceCellFn<FromDrop, ToDrop>>>>,
  _phantom: PhantomData<(FromDrop, ToDrop, Value)>,
}

impl<
    FromDrop: ModelBackedDrop,
    ToDrop: ModelBackedDrop,
    Value: ValueView + Clone,
    Context: crate::Context,
  > PreloaderBuilder for EntityLinkPreloaderBuilder<FromDrop, ToDrop, Value, Context>
where
  FromDrop: Send + Sync + Clone + 'static,
  ToDrop: Send + Sync + Clone + 'static,
  Value: IntoDropResult,
  DropPrimaryKeyValue<FromDrop>:
    Eq + std::hash::Hash + Clone + Send + Sync + std::convert::From<Context::StoreID>,
{
  type Preloader = EntityLinkPreloader<
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
      Ok(EntityLinkPreloader {
        pk_column: self.pk_column.into_column(),
        link: BoxLink(Box::new(self.link)),
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
  > EntityLinkPreloaderBuilder<FromDrop, ToDrop, Value, Context>
where
  FromDrop: Send + Sync + Clone + 'static,
  ToDrop: Send + Sync + Clone + 'static,
  Value: IntoDropResult,
  DropPrimaryKeyValue<FromDrop>: Eq + std::hash::Hash + Clone + Send + Sync,
{
  pub fn new(
    link: impl Linked<
        FromEntity = <FromDrop::Model as ModelTrait>::Entity,
        ToEntity = <ToDrop::Model as ModelTrait>::Entity,
      > + Send
      + Sync
      + 'static,
    pk_column: <<FromDrop::Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey,
  ) -> Self {
    EntityLinkPreloaderBuilder {
      link: ModelBackedDropLink {
        link: Box::new(link),
        _phantom: Default::default(),
      },
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
        EntityLinkLoaderResult<DropEntity<FromDrop>, DropEntity<ToDrop>>,
        FromDrop,
        ToDrop,
        Context::StoreID,
      > + 'static,
  {
    self.loader_result_to_drops = Some(Box::pin(loader_result_to_drops));
    self
  }

  pub fn with_drops_to_value<F>(mut self, drops_to_value: F) -> Self
  where
    F: DropsToValueFn<ToDrop, Value, Context::StoreID> + 'static,
  {
    self.drops_to_value = Some(Box::pin(drops_to_value));
    self
  }

  pub fn with_once_cell_getter<F>(mut self, get_once_cell: F) -> Self
  where
    F: GetOnceCellFn<FromDrop, Value> + 'static,
  {
    self.get_once_cell = Some(Box::pin(get_once_cell));
    self
  }

  pub fn with_inverse_once_cell_getter<F>(mut self, get_inverse_once_cell: F) -> Self
  where
    F: GetInverseOnceCellFn<FromDrop, ToDrop> + 'static,
  {
    self.get_inverse_once_cell = Some(Box::pin(get_inverse_once_cell));
    self
  }
}
