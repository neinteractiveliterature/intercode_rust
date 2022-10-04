use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData, pin::Pin};

use axum::async_trait;
use intercode_graphql::loaders::{
  load_all_linked, load_all_related, EntityLinkLoaderResult, EntityRelationLoaderResult,
};
use lazy_liquid_value_view::{DropResult, LiquidDrop};
use liquid::ValueView;
use once_cell::race::OnceBox;
use sea_orm::{
  DatabaseConnection, EntityTrait, Linked, PrimaryKeyToColumn, PrimaryKeyTrait, Related,
};
use tracing::warn;

use super::DropError;

#[derive(Debug)]
pub struct PreloaderResult<Id: Eq + Hash, Value: ValueView + Clone> {
  values_by_id: HashMap<Id, DropResult<Value>>,
}

impl<Id: Eq + Hash, Value: ValueView + Clone> PreloaderResult<Id, Value> {
  pub fn get(&self, id: &Id) -> DropResult<Value> {
    self.values_by_id.get(id).cloned().unwrap_or_default()
  }

  pub fn expect_value(&self, id: &Id) -> Result<Value, DropError> {
    self
      .get(id)
      .get_inner()
      .cloned()
      .ok_or_else(|| DropError::ExpectedEntityNotFound(std::any::type_name::<Value>().to_string()))
  }

  pub fn all_values(&self) -> Vec<DropResult<Value>> {
    self.values_by_id.values().cloned().collect::<Vec<_>>()
  }

  #[allow(dead_code)]
  pub fn all_values_unwrapped(&self) -> Vec<Value> {
    self
      .all_values()
      .into_iter()
      .map(|value| value.get_inner().unwrap().clone())
      .collect::<Vec<_>>()
  }
}

impl<Id: Eq + Hash, Value: ValueView + Clone> PreloaderResult<Id, Vec<Value>> {
  pub fn all_values_flat(&self) -> Vec<DropResult<Value>> {
    self
      .all_values()
      .iter()
      .flat_map(|drop_result| drop_result.get_inner().unwrap())
      .map(|value| value.into())
      .collect::<Vec<_>>()
  }

  pub fn all_values_flat_unwrapped(&self) -> Vec<Value> {
    self
      .all_values_flat()
      .into_iter()
      .map(|value| value.get_inner().unwrap().clone())
      .collect::<Vec<_>>()
  }
}

pub type GetIdFn<ID, Drop> = dyn Fn(&Drop) -> ID + Send + Sync;
pub type GetValueFn<LoaderResult, Value> =
  dyn Fn(Option<&LoaderResult>) -> Result<Value, DropError> + Send + Sync;
pub type GetOnceCellFn<Drop, Value> =
  dyn Fn(&<Drop as LiquidDrop>::Cache) -> &OnceBox<DropResult<Value>> + Send + Sync;

#[async_trait]
pub trait Preloader<Drop: Send + Sync + Debug, Id: Eq + Hash, V: ValueView + Clone> {
  fn get_id(&self, drop: &Drop) -> Id;
  async fn preload(
    &self,
    db: &DatabaseConnection,
    drops: &[&Drop],
  ) -> Result<PreloaderResult<Id, V>, DropError>;

  async fn load_single(&self, db: &DatabaseConnection, drop: &Drop) -> Result<V, DropError> {
    warn!(
      "N+1 query detected for {:?} -> {}",
      drop,
      std::any::type_name::<V>()
    );

    self
      .preload(db, &[drop])
      .await?
      .expect_value(&self.get_id(drop))
  }
}

async fn populate_db_preloader_results<
  ID: Eq + std::hash::Hash + Clone + From<i64> + Send + Sync,
  Drop: LiquidDrop + Send + Sync,
  Value: ValueView + Into<DropResult<Value>> + Clone + Sync + Send,
  LoaderResult,
>(
  drops: &[&Drop],
  model_lists: HashMap<ID, LoaderResult>,
  get_id: &Pin<Box<GetIdFn<ID, Drop>>>,
  get_value: &Pin<Box<GetValueFn<LoaderResult, Value>>>,
  get_once_cell: &Pin<Box<GetOnceCellFn<Drop, Value>>>,
) -> Result<PreloaderResult<ID, Value>, DropError> {
  let mut values_by_id: HashMap<ID, DropResult<Value>> = HashMap::with_capacity(model_lists.len());

  for drop in drops {
    let id = get_id(drop);
    let result = model_lists.get(&id);
    let value = (get_value)(result)?;
    let drop_result: DropResult<Value> = value.into();

    values_by_id.insert(id, drop_result.clone());

    let once_cell = (get_once_cell)(drop.get_cache());
    // once_cell.set(drop_result).ok();
    once_cell.get_or_init(|| Box::new(drop_result));
  }

  Ok(PreloaderResult { values_by_id })
}

pub struct EntityRelationPreloader<
  From: EntityTrait<PrimaryKey = PK> + Related<To>,
  To: EntityTrait,
  PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
  Drop: LiquidDrop + Send + Sync,
  Value: ValueView + Into<DropResult<Value>>,
> where
  PK::Column: Send + Sync,
  PK::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
{
  pk_column: PK::Column,
  get_id: Pin<Box<GetIdFn<PK::ValueType, Drop>>>,
  get_value: Pin<Box<GetValueFn<EntityRelationLoaderResult<From, To>, Value>>>,
  get_once_cell: Pin<Box<GetOnceCellFn<Drop, Value>>>,
  _phantom: PhantomData<(From, To, Drop)>,
}

impl<
    From: EntityTrait<PrimaryKey = PK> + Related<To>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
    Drop: LiquidDrop + Send + Sync,
    Value: ValueView + Into<DropResult<Value>>,
  > EntityRelationPreloader<From, To, PK, Drop, Value>
where
  PK::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
{
  pub fn new(
    pk_column: PK::Column,
    get_id: impl Fn(&Drop) -> PK::ValueType + Send + Sync + 'static,
    get_value: impl Fn(Option<&EntityRelationLoaderResult<From, To>>) -> Result<Value, DropError>
      + Send
      + Sync
      + 'static,
    get_once_cell: impl Fn(&Drop::Cache) -> &OnceBox<DropResult<Value>> + Send + Sync + 'static,
  ) -> Self {
    EntityRelationPreloader {
      pk_column,
      get_id: Box::pin(get_id),
      get_value: Box::pin(get_value),
      get_once_cell: Box::pin(get_once_cell),
      _phantom: Default::default(),
    }
  }
}

#[async_trait]
impl<
    From: EntityTrait<PrimaryKey = PK> + Related<To>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
    Drop: LiquidDrop + Send + Sync,
    Value: ValueView + Into<DropResult<Value>> + Clone + Send + Sync,
  > Preloader<Drop, PK::ValueType, Value> for EntityRelationPreloader<From, To, PK, Drop, Value>
where
  PK::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
{
  fn get_id(&self, drop: &Drop) -> PK::ValueType {
    (self.get_id)(drop)
  }

  async fn preload(
    &self,
    db: &DatabaseConnection,
    drops: &[&Drop],
  ) -> Result<PreloaderResult<PK::ValueType, Value>, DropError> {
    let model_lists = load_all_related::<From, To, PK>(
      self.pk_column,
      &drops
        .iter()
        .map(|drop| self.get_id(drop))
        .collect::<Vec<_>>(),
      db,
    )
    .await?;

    populate_db_preloader_results(
      drops,
      model_lists,
      &self.get_id,
      &self.get_value,
      &self.get_once_cell,
    )
    .await
  }
}

pub struct EntityLinkPreloader<
  From: EntityTrait<PrimaryKey = PK>,
  Link: Linked<FromEntity = From, ToEntity = To>,
  To: EntityTrait,
  PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
  Drop: LiquidDrop + Send + Sync,
  Value: ValueView,
> where
  PK::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64>,
  Value: Into<DropResult<Value>>,
{
  pk_column: PK::Column,
  link: Link,
  get_id: Pin<Box<GetIdFn<PK::ValueType, Drop>>>,
  get_value: Pin<Box<GetValueFn<EntityLinkLoaderResult<From, To>, Value>>>,
  get_once_cell: Pin<Box<GetOnceCellFn<Drop, Value>>>,
  _phantom: PhantomData<(From, Link, To, Drop)>,
}

impl<
    From: EntityTrait<PrimaryKey = PK>,
    Link: Linked<FromEntity = From, ToEntity = To>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
    Drop: LiquidDrop + Send + Sync,
    Value: ValueView,
  > Debug for EntityLinkPreloader<From, Link, To, PK, Drop, Value>
where
  PK::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64>,
  Value: Into<DropResult<Value>>,
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
    Link: Linked<FromEntity = From, ToEntity = To>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
    Drop: LiquidDrop + Send + Sync,
    Value: ValueView,
  > EntityLinkPreloader<From, Link, To, PK, Drop, Value>
where
  PK::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64>,
  Value: Into<DropResult<Value>>,
{
  pub fn new(
    pk_column: PK::Column,
    link: Link,
    get_id: impl Fn(&Drop) -> PK::ValueType + Send + Sync + 'static,
    get_value: impl Fn(Option<&EntityLinkLoaderResult<From, To>>) -> Result<Value, DropError>
      + Send
      + Sync
      + 'static,
    get_once_cell: impl Fn(&Drop::Cache) -> &OnceBox<DropResult<Value>> + Send + Sync + 'static,
  ) -> Self {
    Self {
      pk_column,
      link,
      get_id: Box::pin(get_id),
      get_value: Box::pin(get_value),
      get_once_cell: Box::pin(get_once_cell),
      _phantom: PhantomData,
    }
  }
}

#[async_trait]
impl<
    From: EntityTrait<PrimaryKey = PK>,
    Link: Linked<FromEntity = From, ToEntity = To> + Clone + Send + Sync,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
    Drop: LiquidDrop + Send + Sync,
    Value: ValueView + Clone + Send + Sync,
  > Preloader<Drop, PK::ValueType, Value> for EntityLinkPreloader<From, Link, To, PK, Drop, Value>
where
  PK::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
  Value: Into<DropResult<Value>>,
{
  fn get_id(&self, drop: &Drop) -> PK::ValueType {
    (self.get_id)(drop)
  }

  async fn preload(
    &self,
    db: &DatabaseConnection,
    drops: &[&Drop],
  ) -> Result<PreloaderResult<PK::ValueType, Value>, DropError> {
    let model_lists = load_all_linked(
      self.pk_column,
      &drops
        .iter()
        .map(|drop| self.get_id(drop))
        .collect::<Vec<_>>(),
      &self.link,
      db,
    )
    .await?;

    populate_db_preloader_results(
      drops,
      model_lists,
      &self.get_id,
      &self.get_value,
      &self.get_once_cell,
    )
    .await
  }
}
