use async_graphql::{async_trait, dataloader::Loader};
use sea_orm::{
  sea_query::{IntoValueTuple, ValueTuple},
  DbErr, EntityTrait, FromQueryResult, ModelTrait, PrimaryKeyToColumn, PrimaryKeyTrait,
  QuerySelect, Related, RelationDef,
};
use std::{collections::HashMap, marker::PhantomData, sync::Arc};

use crate::ConnectionWrapper;

use super::{expect::ExpectModels, AssociationLoaderResult};

#[derive(FromQueryResult, Debug)]
struct ParentModelIdOnly {
  pub parent_model_id: i64,
}

pub async fn load_all_related<From: EntityTrait + Related<To>, To: EntityTrait>(
  pk_column: <From::PrimaryKey as PrimaryKeyToColumn>::Column,
  keys: &[<From::PrimaryKey as PrimaryKeyTrait>::ValueType],
  db: &ConnectionWrapper,
) -> Result<
  HashMap<
    <From::PrimaryKey as PrimaryKeyTrait>::ValueType,
    EntityRelationLoaderResult<From::Model, To::Model>,
  >,
  Arc<DbErr>,
>
where
  <From::PrimaryKey as PrimaryKeyTrait>::ValueType:
    Eq + std::hash::Hash + Clone + std::convert::From<i64>,
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
    HashMap::<
      <From::PrimaryKey as PrimaryKeyTrait>::ValueType,
      EntityRelationLoaderResult<From::Model, To::Model>,
    >::new(),
    |mut acc: HashMap<
      <From::PrimaryKey as PrimaryKeyTrait>::ValueType,
      EntityRelationLoaderResult<From::Model, To::Model>,
    >,
     (from_model, to_model): (ParentModelIdOnly, Option<To::Model>)| {
      if let Some(to_model) = to_model {
        let id = from_model.parent_model_id;
        let result = acc.get_mut(&id.into());
        if let Some(result) = result {
          result.models.push(to_model);
        } else {
          acc.insert(
            id.into(),
            EntityRelationLoaderResult::<From::Model, To::Model> {
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
        EntityRelationLoaderResult::<From::Model, To::Model> {
          from_id: id.to_owned(),
          models: vec![],
        },
      );
    }
  }

  Ok(results)
}

#[derive(Debug, Clone)]
pub struct EntityRelationLoaderResult<FromModel: ModelTrait, ToModel: ModelTrait>
where
  <<FromModel::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
  FromModel::Entity: Related<ToModel::Entity>,
{
  pub from_id: <<FromModel::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
  pub models: Vec<ToModel>,
}

impl<FromModel: ModelTrait, ToModel: ModelTrait> AssociationLoaderResult<FromModel, ToModel>
  for EntityRelationLoaderResult<FromModel, ToModel>
where
  <<FromModel::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
  FromModel::Entity: Related<ToModel::Entity>,
{
  fn get_from_id(
    &self,
  ) -> <<FromModel::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType {
    self.from_id
  }

  fn get_models(&self) -> &Vec<ToModel> {
    &self.models
  }
}

impl<FromModel: ModelTrait, ToModel: ModelTrait> EntityRelationLoaderResult<FromModel, ToModel>
where
  <<FromModel::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
  FromModel::Entity: Related<ToModel::Entity>,
{
  pub fn relation_def(&self) -> RelationDef {
    <FromModel::Entity as Related<ToModel::Entity>>::to()
  }

  pub fn expect_one(&self) -> Result<&ToModel, async_graphql::Error> {
    if self.models.len() == 1 {
      Ok(&self.models[0])
    } else {
      Err(async_graphql::Error::new(format!(
        "Expected one model, but there are {}",
        self.models.len()
      )))
    }
  }

  pub fn try_one(&self) -> Option<&ToModel> {
    if self.models.is_empty() {
      None
    } else {
      Some(&self.models[0])
    }
  }
}

impl<FromModel: ModelTrait, ToModel: ModelTrait> ExpectModels<ToModel>
  for Option<EntityRelationLoaderResult<FromModel, ToModel>>
where
  <<FromModel::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
  FromModel::Entity: Related<ToModel::Entity>,
{
  fn expect_models(&self) -> Result<&Vec<ToModel>, async_graphql::Error> {
    if let Some(result) = self {
      Ok(&result.models)
    } else {
      Err(async_graphql::Error::new(
        "EntityRelationLoader did not insert an expected key!  This should never happen; this is a bug in EntityRelationLoader.",
      ))
    }
  }

  fn expect_one(&self) -> Result<&ToModel, async_graphql::Error> {
    if let Some(result) = self {
      result.expect_one()
    } else {
      Err(async_graphql::Error::new(
        "EntityRelationLoader did not insert an expected key!  This should never happen; this is a bug in EntityRelationLoader.",
      ))
    }
  }

  fn try_one(&self) -> Option<&ToModel> {
    self.as_ref().and_then(|result| result.try_one())
  }
}

impl<FromModel: ModelTrait, ToModel: ModelTrait> ExpectModels<ToModel>
  for Option<&EntityRelationLoaderResult<FromModel, ToModel>>
where
  <<FromModel::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
  FromModel::Entity: Related<ToModel::Entity>,
{
  fn expect_models(&self) -> Result<&Vec<ToModel>, async_graphql::Error> {
    if let Some(result) = self {
      Ok(&result.models)
    } else {
      Err(async_graphql::Error::new(
        "EntityRelationLoader did not insert an expected key!  This should never happen; this is a bug in EntityRelationLoader.",
      ))
    }
  }

  fn expect_one(&self) -> Result<&ToModel, async_graphql::Error> {
    if let Some(result) = self {
      result.expect_one()
    } else {
      Err(async_graphql::Error::new(
        "EntityRelationLoader did not insert an expected key!  This should never happen; this is a bug in EntityRelationLoader.",
      ))
    }
  }

  fn try_one(&self) -> Option<&ToModel> {
    self.as_ref().and_then(|result| result.try_one())
  }
}

#[derive(Debug)]
pub struct EntityRelationLoader<From: EntityTrait + Related<To>, To: EntityTrait> {
  pub db: ConnectionWrapper,
  pub primary_key: From::PrimaryKey,
  _from: PhantomData<From>,
  _to: PhantomData<To>,
}

impl<From: EntityTrait + Related<To>, To: EntityTrait> EntityRelationLoader<From, To> {
  pub fn new(
    db: ConnectionWrapper,
    primary_key: From::PrimaryKey,
  ) -> EntityRelationLoader<From, To> {
    EntityRelationLoader::<From, To> {
      db,
      primary_key,
      _from: PhantomData::<From>,
      _to: PhantomData::<To>,
    }
  }
}

#[async_trait::async_trait]
impl<From: EntityTrait + Related<To>, To: EntityTrait>
  Loader<<From::PrimaryKey as PrimaryKeyTrait>::ValueType> for EntityRelationLoader<From, To>
where
  <From as sea_orm::EntityTrait>::Model: Sync,
  <To as sea_orm::EntityTrait>::Model: Sync,
  <From::PrimaryKey as PrimaryKeyTrait>::ValueType:
    Sync + Clone + Eq + std::hash::Hash + IntoValueTuple + std::convert::From<i64>,
{
  type Value = EntityRelationLoaderResult<From::Model, To::Model>;
  type Error = Arc<sea_orm::DbErr>;

  async fn load(
    &self,
    keys: &[<From::PrimaryKey as PrimaryKeyTrait>::ValueType],
  ) -> Result<
    HashMap<
      <From::PrimaryKey as PrimaryKeyTrait>::ValueType,
      EntityRelationLoaderResult<From::Model, To::Model>,
    >,
    Self::Error,
  > {
    let pk_column = self.primary_key.into_column();

    load_all_related::<From, To>(pk_column, keys, self.db.as_ref()).await
  }
}
