use async_graphql::{async_trait, dataloader::Loader};
use sea_orm::{
  sea_query::{IntoValueTuple, ValueTuple},
  DbErr, EntityTrait, FromQueryResult, Linked, ModelTrait, PrimaryKeyToColumn, PrimaryKeyTrait,
  QuerySelect,
};
use std::{collections::HashMap, marker::PhantomData, sync::Arc};

use crate::ConnectionWrapper;

use super::{expect::ExpectModels, AssociationLoaderResult};

#[derive(FromQueryResult)]
struct ParentModelIdOnly {
  pub parent_model_id: i64,
}

pub async fn load_all_linked<
  From: EntityTrait,
  Link: Linked<FromEntity = From, ToEntity = To>,
  To: EntityTrait,
>(
  pk_column: <From::PrimaryKey as PrimaryKeyToColumn>::Column,
  keys: &[<From::PrimaryKey as PrimaryKeyTrait>::ValueType],
  link: &Link,
  db: &ConnectionWrapper,
) -> Result<
  HashMap<
    <From::PrimaryKey as PrimaryKeyTrait>::ValueType,
    EntityLinkLoaderResult<From::Model, To::Model>,
  >,
  DbErr,
>
where
  Link: Clone,
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

  let mut results = From::find()
    .filter(pk_column.is_in(pk_values))
    .select_only()
    .column_as(pk_column, "parent_model_id")
    .find_also_linked(link.clone())
    .into_model::<ParentModelIdOnly, To::Model>()
    .all(db)
    .await?
    .into_iter()
    .fold(
      HashMap::<
        <From::PrimaryKey as PrimaryKeyTrait>::ValueType,
        EntityLinkLoaderResult<From::Model, To::Model>,
      >::new(),
      |mut acc: HashMap<
        <From::PrimaryKey as PrimaryKeyTrait>::ValueType,
        EntityLinkLoaderResult<From::Model, To::Model>,
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
              EntityLinkLoaderResult::<From::Model, To::Model> {
                from_id: id.into(),
                models: vec![to_model],
              },
            );
          }
        }

        acc
      },
    );

  for id in keys {
    if !results.contains_key(id) {
      results.insert(
        id.to_owned(),
        EntityLinkLoaderResult::<From::Model, To::Model> {
          from_id: id.to_owned(),
          models: vec![],
        },
      );
    }
  }

  Ok(results)
}

#[derive(Debug, Clone)]
pub struct EntityLinkLoaderResult<FromModel: ModelTrait, ToModel: ModelTrait>
where
  <<FromModel::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
{
  from_id: <<FromModel::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
  models: Vec<ToModel>,
}

impl<FromModel: ModelTrait, ToModel: ModelTrait> AssociationLoaderResult<FromModel, ToModel>
  for EntityLinkLoaderResult<FromModel, ToModel>
where
  <<FromModel::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
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

impl<FromModel: ModelTrait, ToModel: ModelTrait> EntityLinkLoaderResult<FromModel, ToModel>
where
  <<FromModel::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
{
  fn expect_one(&self) -> Result<&ToModel, async_graphql::Error> {
    if self.models.len() == 1 {
      Ok(&self.models[0])
    } else {
      Err(async_graphql::Error::new(format!(
        "Expected one model, but there are {}",
        self.models.len()
      )))
    }
  }

  fn try_one(&self) -> Option<&ToModel> {
    if self.models.is_empty() {
      None
    } else {
      Some(&self.models[0])
    }
  }
}

impl<FromModel: ModelTrait, ToModel: ModelTrait> ExpectModels<ToModel>
  for Option<EntityLinkLoaderResult<FromModel, ToModel>>
where
  <<FromModel::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
{
  fn expect_models(&self) -> Result<&Vec<ToModel>, async_graphql::Error> {
    if let Some(result) = self {
      Ok(&result.models)
    } else {
      Err(async_graphql::Error::new(
        "EntityLinkLoader did not insert an expected key!  This should never happen; this is a bug in EntityLinkLoader.",
      ))
    }
  }

  fn expect_one(&self) -> Result<&ToModel, async_graphql::Error> {
    if let Some(result) = self {
      result.expect_one()
    } else {
      Err(async_graphql::Error::new(
        "EntityLinkLoader did not insert an expected key!  This should never happen; this is a bug in EntityLinkLoader.",
      ))
    }
  }

  fn try_one(&self) -> Option<&ToModel> {
    self.as_ref().and_then(|result| result.try_one())
  }
}

impl<FromModel: ModelTrait, ToModel: ModelTrait> ExpectModels<ToModel>
  for Option<&EntityLinkLoaderResult<FromModel, ToModel>>
where
  <<FromModel::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
{
  fn expect_models(&self) -> Result<&Vec<ToModel>, async_graphql::Error> {
    if let Some(result) = self {
      Ok(&result.models)
    } else {
      Err(async_graphql::Error::new(
        "EntityLinkLoader did not insert an expected key!  This should never happen; this is a bug in EntityLinkLoader.",
      ))
    }
  }

  fn expect_one(&self) -> Result<&ToModel, async_graphql::Error> {
    if let Some(result) = self {
      result.expect_one()
    } else {
      Err(async_graphql::Error::new(
        "EntityLinkLoader did not insert an expected key!  This should never happen; this is a bug in EntityLinkLoader.",
      ))
    }
  }

  fn try_one(&self) -> Option<&ToModel> {
    self.as_ref().and_then(|result| result.try_one())
  }
}

#[derive(Debug)]
pub struct EntityLinkLoader<
  From: EntityTrait,
  Link: Linked<FromEntity = From, ToEntity = To>,
  To: EntityTrait,
> {
  pub db: ConnectionWrapper,
  pub primary_key: From::PrimaryKey,
  pub link: Link,
  _from: PhantomData<From>,
  _to: PhantomData<To>,
}

impl<From: EntityTrait, Link: Linked<FromEntity = From, ToEntity = To>, To: EntityTrait>
  EntityLinkLoader<From, Link, To>
{
  pub fn new(
    db: ConnectionWrapper,
    link: Link,
    primary_key: From::PrimaryKey,
  ) -> EntityLinkLoader<From, Link, To> {
    EntityLinkLoader::<From, Link, To> {
      db,
      link,
      primary_key,
      _from: PhantomData::<From>,
      _to: PhantomData::<To>,
    }
  }
}

#[async_trait::async_trait]
impl<From: EntityTrait, Link: Linked<FromEntity = From, ToEntity = To>, To: EntityTrait>
  Loader<<From::PrimaryKey as PrimaryKeyTrait>::ValueType> for EntityLinkLoader<From, Link, To>
where
  <From as sea_orm::EntityTrait>::Model: Sync,
  <To as sea_orm::EntityTrait>::Model: Sync,
  Link: 'static + Send + Sync + Clone,
  From::PrimaryKey: PrimaryKeyToColumn,
  <From::PrimaryKey as PrimaryKeyTrait>::ValueType:
    Sync + Clone + Eq + std::hash::Hash + IntoValueTuple + std::convert::From<i64>,
{
  type Value = EntityLinkLoaderResult<From::Model, To::Model>;
  type Error = Arc<sea_orm::DbErr>;

  async fn load(
    &self,
    keys: &[<From::PrimaryKey as PrimaryKeyTrait>::ValueType],
  ) -> Result<
    HashMap<
      <From::PrimaryKey as PrimaryKeyTrait>::ValueType,
      EntityLinkLoaderResult<From::Model, To::Model>,
    >,
    Self::Error,
  > {
    let pk_column = self.primary_key.into_column();

    Ok(load_all_linked(pk_column, keys, &self.link, self.db.as_ref()).await?)
  }
}
