#![allow(clippy::type_complexity)]
use std::{collections::HashMap, fmt::Display, marker::PhantomData, pin::Pin};

use crate::{
  loaders::{load_all_related, EntityRelationLoaderResult},
  ConnectionWrapper, DropRef, DropResult, DropResultTrait, IntoDropResult, LiquidDrop,
};
use async_trait::async_trait;
use liquid::ValueView;
use once_cell::race::OnceBox;
use sea_orm::{EntityTrait, PrimaryKeyToColumn, PrimaryKeyTrait, Related, TryGetable};

use crate::{DropError, DropStore};

use super::{
  DropsToValueFn, GetInverseOnceCellFn, GetOnceCellFn, LoaderResultToDropsFn, Preloader,
};

pub struct EntityRelationPreloader<
  From: EntityTrait<PrimaryKey = PK> + Related<To>,
  To: EntityTrait,
  PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
  FromDrop: LiquidDrop + Send + Sync + Clone + 'static,
  ToDrop: LiquidDrop + Send + Sync + Clone + 'static,
  Value: ValueView + Clone + IntoDropResult,
  Context: crate::Context,
> where
  PK::Column: Send + Sync,
  PK::ValueType: Eq + std::hash::Hash + Clone + Send + Sync,
{
  pk_column: PK::Column,
  context: Context,
  loader_result_to_drops:
    Pin<Box<dyn LoaderResultToDropsFn<EntityRelationLoaderResult<From, To>, FromDrop, ToDrop>>>,
  drops_to_value: Pin<Box<dyn DropsToValueFn<ToDrop, Value>>>,
  get_once_cell: Pin<Box<dyn GetOnceCellFn<FromDrop, Value>>>,
  get_inverse_once_cell: Option<Pin<Box<dyn GetInverseOnceCellFn<FromDrop, ToDrop>>>>,
  _phantom: PhantomData<(From, To, FromDrop, ToDrop)>,
}

impl<
    From: EntityTrait<PrimaryKey = PK> + Related<To>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
    FromDrop: LiquidDrop + Send + Sync + Clone + 'static,
    ToDrop: LiquidDrop + Send + Sync + Clone + 'static,
    Value: ValueView + Clone + IntoDropResult,
    Context: crate::Context,
  > EntityRelationPreloader<From, To, PK, FromDrop, ToDrop, Value, Context>
where
  PK::Column: Send + Sync,
  PK::ValueType: Eq + std::hash::Hash + Clone + Send + Sync,
{
  pub fn new<
    LRDF: LoaderResultToDropsFn<EntityRelationLoaderResult<From, To>, FromDrop, ToDrop> + 'static,
    DVF: DropsToValueFn<ToDrop, Value> + 'static,
    GOCF: GetOnceCellFn<FromDrop, Value> + 'static,
  >(
    pk: PK,
    context: Context,
    loader_result_to_drops: LRDF,
    drops_to_value: DVF,
    get_once_cell: GOCF,
    get_inverse_once_cell: Option<Pin<Box<dyn GetInverseOnceCellFn<FromDrop, ToDrop>>>>,
  ) -> Self {
    Self {
      pk_column: PrimaryKeyToColumn::into_column(pk),
      context,
      loader_result_to_drops: Box::pin(loader_result_to_drops),
      drops_to_value: Box::pin(drops_to_value),
      get_once_cell: Box::pin(get_once_cell),
      get_inverse_once_cell,
      _phantom: Default::default(),
    }
  }
}

#[async_trait]
impl<
    From: EntityTrait<PrimaryKey = PK> + Related<To>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
    FromDrop: LiquidDrop<ID = PK::ValueType, Context = Context> + Send + Sync + Clone,
    ToDrop: LiquidDrop<ID = PK::ValueType, Context = Context> + Send + Sync + Clone + 'static,
    Value: ValueView + IntoDropResult + DropResultTrait<Value> + Clone + Send + Sync + 'static,
    Context: crate::Context,
  > Preloader<FromDrop, ToDrop, Value, Context>
  for EntityRelationPreloader<From, To, PK, FromDrop, ToDrop, Value, Context>
where
  PK::ValueType: Eq + std::hash::Hash + Copy + Clone + Send + Sync + Display + TryGetable,
  To::Model: Send + Sync,
  Context::StoreID: std::convert::From<PK::ValueType> + Into<PK::ValueType>,
{
  type LoaderResult = EntityRelationLoaderResult<From, To>;

  fn loader_result_to_drops(
    &self,
    result: Option<Self::LoaderResult>,
    drop: DropRef<FromDrop>,
  ) -> Result<Vec<ToDrop>, DropError> {
    (self.loader_result_to_drops)(result, drop)
  }

  fn with_drop_store<'store, R: 'store, F: FnOnce(&'store DropStore<Context::StoreID>) -> R>(
    &'store self,
    f: F,
  ) -> R {
    self.context.with_drop_store(f)
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
    load_all_related::<From, To>(self.pk_column, &ids_converted, db)
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
