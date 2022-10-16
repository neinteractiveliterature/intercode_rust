use std::{collections::HashMap, marker::PhantomData, pin::Pin};

use async_trait::async_trait;
use intercode_graphql::loaders::{load_all_related, EntityRelationLoaderResult};
use lazy_liquid_value_view::{DropResult, LiquidDrop};
use liquid::ValueView;
use sea_orm::{
  DatabaseConnection, EntityTrait, ModelTrait, PrimaryKeyToColumn, PrimaryKeyTrait, Related,
};

use crate::{DropError, ModelBackedDrop};

use super::{
  populate_db_preloader_results, populate_inverse_caches, GetIdFn, GetInverseOnceCellFn,
  GetOnceCellFn, GetValueFn, IncompleteBuilderError, Preloader, PreloaderBuilder, PreloaderResult,
};

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
  get_id: Pin<Box<dyn for<'a> GetIdFn<'a, PK::ValueType, Drop>>>,
  get_value:
    Pin<Box<dyn for<'a> GetValueFn<'a, EntityRelationLoaderResult<From, To>, Value, Drop>>>,
  get_once_cell: Pin<Box<dyn for<'a> GetOnceCellFn<'a, Drop, Value>>>,
  get_inverse_once_cell: Option<Pin<Box<dyn for<'a> GetInverseOnceCellFn<'a, Drop, Value>>>>,
  _phantom: PhantomData<(From, To, Drop)>,
}

#[async_trait]
impl<
    From: EntityTrait<PrimaryKey = PK> + Related<To>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
    Drop: LiquidDrop + Send + Sync + Clone,
    Value: ValueView + Into<DropResult<Value>> + Clone + Send + Sync,
  > Preloader<Drop, PK::ValueType, Value> for EntityRelationPreloader<From, To, PK, Drop, Value>
where
  PK::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
  To::Model: Send + Sync,
{
  fn get_id(&self, drop: &Drop) -> PK::ValueType {
    (self.get_id)(drop)
  }

  async fn preload(
    &self,
    db: &DatabaseConnection,
    drops: &[&Drop],
  ) -> Result<PreloaderResult<PK::ValueType, Value>, DropError> {
    let drops_by_id = drops
      .iter()
      .map(|drop| (self.get_id(drop), drop))
      .collect::<HashMap<_, _>>();

    let model_lists = load_all_related::<From, To, PK>(
      self.pk_column,
      drops_by_id.keys().cloned().collect::<Vec<_>>().as_slice(),
      db,
    )
    .await?;

    let preloader_result = populate_db_preloader_results(
      drops,
      model_lists,
      &self.get_id,
      &self.get_value,
      &self.get_once_cell,
    )
    .await?;

    if let Some(get_inverse_once_cell) = &self.get_inverse_once_cell {
      for from_drop in drops_by_id.values() {
        populate_inverse_caches(**from_drop, &preloader_result, get_inverse_once_cell)
      }
    }

    Ok(preloader_result)
  }
}

pub struct EntityRelationPreloaderBuilder<
  FromDrop: ModelBackedDrop,
  ToDrop: ModelBackedDrop,
  Value: ValueView,
> where
  FromDrop: Send + Sync,
  ToDrop: Send + Sync,
  <FromDrop::Model as ModelTrait>::Entity: Related<<ToDrop::Model as ModelTrait>::Entity>,
  <<<FromDrop::Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync,
  Value: Into<DropResult<Value>>,
  <<<FromDrop::Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone
{
  pk_column: <<FromDrop::Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey,
  get_id: Option<Pin<Box<dyn for<'a> GetIdFn<'a, <<<FromDrop::Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType, FromDrop>>>>,
  get_value: Option<
    Pin<
      Box<
        dyn for<'a> GetValueFn<
          'a,
          EntityRelationLoaderResult<
            <FromDrop::Model as ModelTrait>::Entity,
            <ToDrop::Model as ModelTrait>::Entity,
          >,
          Value,
          FromDrop,
        >,
      >,
    >,
  >,
  get_once_cell: Option<Pin<Box<dyn for<'a> GetOnceCellFn<'a, FromDrop, Value>>>>,
  get_inverse_once_cell: Option<Pin<Box<dyn for<'a> GetInverseOnceCellFn<'a, FromDrop, Value>>>>,
  _phantom: PhantomData<(FromDrop, ToDrop, Value)>,
}

impl<
  FromDrop: ModelBackedDrop,
  ToDrop: ModelBackedDrop,
  Value: ValueView,
> PreloaderBuilder for EntityRelationPreloaderBuilder<FromDrop, ToDrop, Value> where
  FromDrop: Send + Sync,
  ToDrop: Send + Sync,
  <FromDrop::Model as ModelTrait>::Entity: Related<<ToDrop::Model as ModelTrait>::Entity>,
  Value: Into<DropResult<Value>>,
  <<<FromDrop::Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync{
    type Preloader = EntityRelationPreloader<
      <FromDrop::Model as ModelTrait>::Entity,
      <ToDrop::Model as ModelTrait>::Entity,
      <<FromDrop::Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey,
      FromDrop,
      Value
    >;
    type Error = IncompleteBuilderError;

  fn finalize(
    self,
  ) -> Result<Self::Preloader, Self::Error> {
    if let (Some(get_id), Some(get_value), Some(get_once_cell)) =
      (self.get_id, self.get_value, self.get_once_cell)
    {
      Ok(EntityRelationPreloader {
        pk_column: self.pk_column.into_column(),
        get_id,
        get_value,
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
  > EntityRelationPreloaderBuilder<FromDrop, ToDrop, Value>
where
  FromDrop: Send + Sync,
  ToDrop: Send + Sync,
  <FromDrop::Model as ModelTrait>::Entity: Related<<ToDrop::Model as ModelTrait>::Entity>,
  Value: Into<DropResult<Value>>,
  <<<FromDrop::Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64> + Send + Sync
{
  pub fn new(pk_column: <<FromDrop::Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey) -> Self {
    EntityRelationPreloaderBuilder {
      pk_column,
      get_id: None,
      get_value: None,
      get_once_cell: None,
      get_inverse_once_cell: None,
      _phantom: Default::default(),
    }
  }

  pub fn with_id_getter<F>(mut self, get_id: F) -> Self
  where
    F: for<'a> GetIdFn<'a, <<<FromDrop::Model as ModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType, FromDrop> + 'static,
  {
    self.get_id = Some(Box::pin(get_id));
    self
  }

  pub fn with_value_getter<F>(mut self, get_value: F) -> Self
  where
    F: for<'a> GetValueFn<
      'a,
      EntityRelationLoaderResult<
        <FromDrop::Model as ModelTrait>::Entity,
        <ToDrop::Model as ModelTrait>::Entity,
      >,
      Value,
      FromDrop,
    > + 'static,
  {
    self.get_value = Some(Box::pin(get_value));
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
    F: for<'a> GetInverseOnceCellFn<'a, FromDrop, Value> + 'static,
  {
    self.get_inverse_once_cell = Some(Box::pin(get_inverse_once_cell));
    self
  }
}
