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
use sea_orm::{EntityTrait, Linked, ModelTrait, PrimaryKeyToColumn, PrimaryKeyTrait, TryGetable};

use crate::{DropEntity, DropError, DropStore, ModelBackedDrop};

use super::{
  DropsToValueFn, GetInverseOnceCellFn, GetOnceCellFn, LoaderResultToDropsFn, Preloader,
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
  loader_result_to_drops:
    Pin<Box<dyn LoaderResultToDropsFn<EntityLinkLoaderResult<From, To>, FromDrop, ToDrop>>>,
  drops_to_value: Pin<Box<dyn DropsToValueFn<ToDrop, Value>>>,
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

impl<
    From: EntityTrait<PrimaryKey = PK>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
    FromDrop: LiquidDrop + Clone + Send + Sync + 'static,
    ToDrop: LiquidDrop + Clone + Send + Sync + 'static,
    Value: ValueView + Clone,
    Context: crate::Context,
  > EntityLinkPreloader<From, To, PK, FromDrop, ToDrop, Value, Context>
where
  PK::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<Context::StoreID>,
  Value: IntoDropResult,
{
  pub fn new<
    Link: Linked<FromEntity = From, ToEntity = To> + Send + Sync + 'static,
    LRDF: LoaderResultToDropsFn<EntityLinkLoaderResult<From, To>, FromDrop, ToDrop> + 'static,
    DVF: DropsToValueFn<ToDrop, Value> + 'static,
    GOCF: GetOnceCellFn<FromDrop, Value> + 'static,
  >(
    pk: PK,
    link: Link,
    context: Context,
    loader_result_to_drops: LRDF,
    drops_to_value: DVF,
    get_once_cell: GOCF,
    get_inverse_once_cell: Option<Pin<Box<dyn GetInverseOnceCellFn<FromDrop, ToDrop>>>>,
  ) -> Self {
    Self {
      pk_column: pk.into_column(),
      link: BoxLink(Box::new(link)),
      context,
      loader_result_to_drops: Box::pin(loader_result_to_drops),
      drops_to_value: Box::pin(drops_to_value),
      get_once_cell: Box::pin(get_once_cell),
      get_inverse_once_cell,
      _phantom: PhantomData,
    }
  }
}

#[async_trait]
impl<
    From: EntityTrait<PrimaryKey = PK>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
    FromDrop: LiquidDrop<ID = PK::ValueType, Context = Context> + Send + Sync + Clone,
    ToDrop: LiquidDrop<ID = PK::ValueType, Context = Context> + Send + Sync + Clone,
    Value: ValueView + Clone + Send + Sync + DropResultTrait<Value> + 'static,
    Context: crate::Context,
  > Preloader<FromDrop, ToDrop, Value, Context>
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

  fn with_drop_store<'store, R: 'store, F: FnOnce(&'store DropStore<Context::StoreID>) -> R>(
    &'store self,
    f: F,
  ) -> R {
    self.context.with_drop_store(f)
  }

  fn loader_result_to_drops(
    &self,
    result: Option<Self::LoaderResult>,
    drop: DropRef<FromDrop>,
  ) -> Result<Vec<ToDrop>, DropError> {
    (self.loader_result_to_drops)(result, drop)
  }

  fn drops_to_value(
    &self,
    store: &DropStore<Context::StoreID>,
    drops: Vec<DropRef<ToDrop>>,
  ) -> Result<DropResult<Value>, DropError> {
    (self.drops_to_value)(store, drops)
  }

  fn get_once_cell<'a>(&self, cache: &'a FromDrop::Cache) -> &'a OnceBox<DropResult<Value>> {
    (self.get_once_cell)(cache)
  }

  fn get_inverse_once_cell<'a>(
    &self,
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
