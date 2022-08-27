use std::{collections::HashMap, hash::Hash, marker::PhantomData, pin::Pin};

use axum::async_trait;
use intercode_graphql::loaders::{
  load_all_linked, load_all_related, EntityLinkLoaderResult, EntityRelationLoaderResult,
};
use lazy_liquid_value_view::DropResult;
use liquid::ValueView;
use sea_orm::{
  DatabaseConnection, EntityTrait, Linked, PrimaryKeyToColumn, PrimaryKeyTrait, Related,
};

use super::DropError;

pub struct PreloaderResult<Id: Eq + Hash, Value: ValueView> {
  values_by_id: HashMap<Id, DropResult<Value>>,
}

impl<Id: Eq + Hash, Value: ValueView> PreloaderResult<Id, Value> {
  pub fn get(&self, id: &Id) -> DropResult<Value> {
    self.values_by_id.get(id).cloned().unwrap_or_default()
  }

  pub fn all_values(&self) -> Vec<DropResult<Value>> {
    self.values_by_id.values().cloned().collect::<Vec<_>>()
  }
}

impl<Id: Eq + Hash, Value: ValueView> PreloaderResult<Id, Vec<Value>> {
  pub fn all_values_flat(&self) -> Vec<&Value> {
    self
      .values_by_id
      .values()
      .flat_map(|values| values.get_inner().unwrap().iter())
      .collect::<Vec<_>>()
  }
}

pub type GetIdFn<ID, Drop> = dyn Fn(&Drop) -> ID + Send + Sync;
pub type GetValueFn<LoaderResult, Value> =
  dyn Fn(Option<&LoaderResult>) -> Result<Value, DropError> + Send + Sync;
pub type SetValueFn<Drop, Value> =
  dyn Fn(&Drop, DropResult<Value>) -> Result<(), DropError> + Send + Sync;

#[async_trait]
pub trait Preloader<Drop, Id: Eq + Hash, Value: ValueView> {
  async fn preload(
    &self,
    db: &DatabaseConnection,
    drops: &[&Drop],
  ) -> Result<PreloaderResult<Id, Value>, DropError>;
}

pub struct EntityRelationPreloader<
  From: EntityTrait<PrimaryKey = PK> + Related<To>,
  To: EntityTrait,
  PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
  Drop: Send + Sync,
  Value: ValueView + Into<DropResult<Value>>,
> where
  PK::Column: Send + Sync,
  PK::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
{
  pk_column: PK::Column,
  get_id: Pin<Box<GetIdFn<PK::ValueType, Drop>>>,
  get_value: Pin<Box<GetValueFn<EntityRelationLoaderResult<From, To>, Value>>>,
  set_value: Pin<Box<SetValueFn<Drop, Value>>>,
  _phantom: PhantomData<(From, To, Drop)>,
}

impl<
    From: EntityTrait<PrimaryKey = PK> + Related<To>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
    Drop: Send + Sync,
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
    set_value: impl Fn(&Drop, DropResult<Value>) -> Result<(), DropError> + Send + Sync + 'static,
  ) -> Self {
    EntityRelationPreloader {
      pk_column,
      get_id: Box::pin(get_id),
      get_value: Box::pin(get_value),
      set_value: Box::pin(set_value),
      _phantom: Default::default(),
    }
  }
}

#[async_trait]
impl<
    From: EntityTrait<PrimaryKey = PK> + Related<To>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
    Drop: Send + Sync,
    Value: ValueView + Into<DropResult<Value>>,
  > Preloader<Drop, PK::ValueType, Value> for EntityRelationPreloader<From, To, PK, Drop, Value>
where
  PK::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
{
  async fn preload(
    &self,
    db: &DatabaseConnection,
    drops: &[&Drop],
  ) -> Result<PreloaderResult<PK::ValueType, Value>, DropError> {
    let model_lists = load_all_related::<From, To, PK>(
      self.pk_column,
      &drops
        .iter()
        .map(|drop| (self.get_id)(drop))
        .collect::<Vec<_>>(),
      db,
    )
    .await?;

    let mut values_by_id: HashMap<PK::ValueType, DropResult<Value>> = Default::default();

    for drop in drops {
      let id = (self.get_id)(drop);
      let result = model_lists.get(&id);
      let value: DropResult<Value> = (self.get_value)(result)?.into();

      values_by_id.insert(id, value.clone());
      (self.set_value)(drop, value)?;
    }

    Ok(PreloaderResult { values_by_id })
  }
}

pub struct EntityLinkPreloader<
  From: EntityTrait<PrimaryKey = PK>,
  Link: Linked<FromEntity = From, ToEntity = To>,
  To: EntityTrait,
  PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
  Drop: Send + Sync,
  Value: ValueView,
> where
  PK::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64>,
  Value: Into<DropResult<Value>>,
{
  pk_column: PK::Column,
  link: Link,
  get_id: Pin<Box<GetIdFn<PK::ValueType, Drop>>>,
  get_value: Pin<Box<GetValueFn<EntityLinkLoaderResult<From, To>, Value>>>,
  set_value: Pin<Box<SetValueFn<Drop, Value>>>,
  _phantom: PhantomData<(From, Link, To, Drop)>,
}

impl<
    From: EntityTrait<PrimaryKey = PK>,
    Link: Linked<FromEntity = From, ToEntity = To>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
    Drop: Send + Sync,
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
    set_value: impl Fn(&Drop, DropResult<Value>) -> Result<(), DropError> + Send + Sync + 'static,
  ) -> Self {
    Self {
      pk_column,
      link,
      get_id: Box::pin(get_id),
      get_value: Box::pin(get_value),
      set_value: Box::pin(set_value),
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
    Drop: Send + Sync,
    Value: ValueView,
  > Preloader<Drop, PK::ValueType, Value> for EntityLinkPreloader<From, Link, To, PK, Drop, Value>
where
  PK::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
  Value: Into<DropResult<Value>>,
{
  async fn preload(
    &self,
    db: &DatabaseConnection,
    drops: &[&Drop],
  ) -> Result<PreloaderResult<PK::ValueType, Value>, DropError> {
    let model_lists = load_all_linked(
      self.pk_column,
      &drops
        .iter()
        .map(|drop| (self.get_id)(drop))
        .collect::<Vec<_>>(),
      &self.link,
      db,
    )
    .await?;

    let mut values_by_id: HashMap<PK::ValueType, DropResult<Value>> = Default::default();

    for drop in drops {
      let id = (self.get_id)(drop);
      let result = model_lists.get(&id);
      let value: DropResult<Value> = (self.get_value)(result)?.into();

      values_by_id.insert(id, value.clone());
      (self.set_value)(drop, value)?;
    }

    Ok(PreloaderResult { values_by_id })
  }
}
