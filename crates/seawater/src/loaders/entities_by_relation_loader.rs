use async_graphql::{async_trait, dataloader::Loader};
use sea_orm::{
  sea_query::{IntoValueTuple, ValueTuple},
  DbErr, EntityTrait, FromQueryResult, PrimaryKeyToColumn, PrimaryKeyTrait, QuerySelect, Related,
  RelationDef,
};
use std::{collections::HashMap, marker::PhantomData, sync::Arc};

use crate::ConnectionWrapper;

use super::expect::ExpectModels;

#[derive(FromQueryResult, Debug)]
struct ParentModelIdOnly {
  pub parent_model_id: i64,
}

pub async fn load_all_related<
  From: EntityTrait<PrimaryKey = PK> + Related<To>,
  To: EntityTrait,
  PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
>(
  pk_column: PK::Column,
  keys: &[PK::ValueType],
  db: &ConnectionWrapper,
) -> Result<HashMap<PK::ValueType, EntityRelationLoaderResult<From, To>>, Arc<DbErr>>
where
  PK::ValueType: Eq + std::hash::Hash + Clone + std::convert::From<i64>,
{
  use sea_orm::{ColumnTrait, QueryFilter};

  let pk_values = keys.iter().map(|key| {
    let tuple = key.clone().into_value_tuple();
    if let ValueTuple::One(single_value) = tuple {
      single_value
    } else {
      panic!(
        "EntityRelationshipLoader does not work with composite primary keys (encountered {:?})",
        tuple
      )
    }
  });

  let query_results = From::find()
    .filter(pk_column.is_in(pk_values))
    .select_only()
    .column_as(pk_column, "parent_model_id")
    .find_also_related(To::default())
    .into_model::<ParentModelIdOnly, To::Model>()
    .all(db)
    .await?;

  let mut results = query_results.into_iter().fold(
    HashMap::<PK::ValueType, EntityRelationLoaderResult<From, To>>::new(),
    |mut acc: HashMap<PK::ValueType, EntityRelationLoaderResult<From, To>>,
     (from_model, to_model): (ParentModelIdOnly, Option<To::Model>)| {
      if let Some(to_model) = to_model {
        let id = from_model.parent_model_id;
        let result = acc.get_mut(&id.into());
        if let Some(result) = result {
          result.models.push(to_model);
        } else {
          acc.insert(
            id.into(),
            EntityRelationLoaderResult::<From, To> {
              from_id: id.into(),
              models: vec![to_model],
            },
          );
        }
      }

      acc
    },
  );

  for id in keys.iter() {
    if !results.contains_key(id) {
      results.insert(
        id.to_owned(),
        EntityRelationLoaderResult::<From, To> {
          from_id: id.to_owned(),
          models: vec![],
        },
      );
    }
  }

  Ok(results)
}

#[derive(Debug, Clone)]
pub struct EntityRelationLoaderResult<From: EntityTrait + Related<To>, To: EntityTrait>
where
  <<From as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
{
  pub from_id: <<From as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
  pub models: Vec<To::Model>,
}

impl<From: EntityTrait + Related<To>, To: EntityTrait> EntityRelationLoaderResult<From, To>
where
  <<From as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
{
  pub fn relation_def(&self) -> RelationDef {
    <From as Related<To>>::to()
  }

  pub fn expect_one(&self) -> Result<&To::Model, async_graphql::Error> {
    if self.models.len() == 1 {
      Ok(&self.models[0])
    } else {
      Err(async_graphql::Error::new(format!(
        "Expected one model, but there are {}",
        self.models.len()
      )))
    }
  }

  pub fn try_one(&self) -> Option<&To::Model> {
    if self.models.is_empty() {
      None
    } else {
      Some(&self.models[0])
    }
  }
}

impl<From: EntityTrait + Related<To>, To: EntityTrait> ExpectModels<To::Model>
  for Option<EntityRelationLoaderResult<From, To>>
where
  <<From as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
{
  fn expect_models(&self) -> Result<&Vec<To::Model>, async_graphql::Error> {
    if let Some(result) = self {
      Ok(&result.models)
    } else {
      Err(async_graphql::Error::new(
        "EntityRelationLoader did not insert an expected key!  This should never happen; this is a bug in EntityRelationLoader.",
      ))
    }
  }

  fn expect_one(&self) -> Result<&To::Model, async_graphql::Error> {
    if let Some(result) = self {
      result.expect_one()
    } else {
      Err(async_graphql::Error::new(
        "EntityRelationLoader did not insert an expected key!  This should never happen; this is a bug in EntityRelationLoader.",
      ))
    }
  }

  fn try_one(&self) -> Option<&To::Model> {
    self.as_ref().and_then(|result| result.try_one())
  }
}

impl<From: EntityTrait + Related<To>, To: EntityTrait> ExpectModels<To::Model>
  for Option<&EntityRelationLoaderResult<From, To>>
where
  <<From as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
{
  fn expect_models(&self) -> Result<&Vec<To::Model>, async_graphql::Error> {
    if let Some(result) = self {
      Ok(&result.models)
    } else {
      Err(async_graphql::Error::new(
        "EntityRelationLoader did not insert an expected key!  This should never happen; this is a bug in EntityRelationLoader.",
      ))
    }
  }

  fn expect_one(&self) -> Result<&To::Model, async_graphql::Error> {
    if let Some(result) = self {
      result.expect_one()
    } else {
      Err(async_graphql::Error::new(
        "EntityRelationLoader did not insert an expected key!  This should never happen; this is a bug in EntityRelationLoader.",
      ))
    }
  }

  fn try_one(&self) -> Option<&To::Model> {
    self.as_ref().and_then(|result| result.try_one())
  }
}

#[derive(Debug)]
pub struct EntityRelationLoader<
  From: EntityTrait<PrimaryKey = PK> + Related<To>,
  To: EntityTrait,
  PK: PrimaryKeyTrait + PrimaryKeyToColumn,
> {
  pub db: ConnectionWrapper,
  pub primary_key: PK,
  _from: PhantomData<From>,
  _to: PhantomData<To>,
}

impl<
    From: EntityTrait<PrimaryKey = PK> + Related<To>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn,
  > EntityRelationLoader<From, To, PK>
{
  pub fn new(db: ConnectionWrapper, primary_key: PK) -> EntityRelationLoader<From, To, PK> {
    EntityRelationLoader::<From, To, PK> {
      db,
      primary_key,
      _from: PhantomData::<From>,
      _to: PhantomData::<To>,
    }
  }
}

#[async_trait::async_trait]
impl<
    From: EntityTrait<PrimaryKey = PK> + Related<To>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
  > Loader<PK::ValueType> for EntityRelationLoader<From, To, PK>
where
  <From as sea_orm::EntityTrait>::Model: Sync,
  <To as sea_orm::EntityTrait>::Model: Sync,
  PK::ValueType: Sync + Clone + Eq + std::hash::Hash + IntoValueTuple + std::convert::From<i64>,
{
  type Value = EntityRelationLoaderResult<From, To>;
  type Error = Arc<sea_orm::DbErr>;

  async fn load(
    &self,
    keys: &[PK::ValueType],
  ) -> Result<HashMap<PK::ValueType, EntityRelationLoaderResult<From, To>>, Self::Error> {
    let pk_column = self.primary_key.into_column();

    load_all_related::<From, To, PK>(pk_column, keys, self.db.as_ref()).await
  }
}
