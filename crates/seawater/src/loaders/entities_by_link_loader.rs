use async_graphql::{async_trait, dataloader::Loader};
use sea_orm::{
  sea_query::{IntoValueTuple, ValueTuple},
  DbErr, EntityTrait, FromQueryResult, Linked, PrimaryKeyToColumn, PrimaryKeyTrait, QuerySelect,
};
use std::{collections::HashMap, sync::Arc};

use crate::ConnectionWrapper;

use super::expect::ExpectModels;

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
  HashMap<<From::PrimaryKey as PrimaryKeyTrait>::ValueType, EntityLinkLoaderResult<From, To>>,
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
      HashMap::<<From::PrimaryKey as PrimaryKeyTrait>::ValueType, EntityLinkLoaderResult<From, To>>::new(),
      |mut acc: HashMap<<From::PrimaryKey as PrimaryKeyTrait>::ValueType, EntityLinkLoaderResult<From, To>>,
       (from_model, to_model): (ParentModelIdOnly, Option<To::Model>)| {
        if let Some(to_model) = to_model {
          let id = from_model.parent_model_id;
          let result = acc.get_mut(&id.into());
          if let Some(result) = result {
            result.models.push(to_model);
          } else {
            acc.insert(
              id.into(),
              EntityLinkLoaderResult::<From, To> {
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
        EntityLinkLoaderResult::<From, To> {
          from_id: id.to_owned(),
          models: vec![],
        },
      );
    }
  }

  Ok(results)
}

#[derive(Debug, Clone)]
pub struct EntityLinkLoaderResult<From: EntityTrait, To: EntityTrait>
where
  <<From as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
{
  pub from_id: <<From as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
  pub models: Vec<To::Model>,
}

impl<From: EntityTrait, To: EntityTrait> EntityLinkLoaderResult<From, To>
where
  <<From as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
{
  fn expect_one(&self) -> Result<&To::Model, async_graphql::Error> {
    if self.models.len() == 1 {
      Ok(&self.models[0])
    } else {
      Err(async_graphql::Error::new(format!(
        "Expected one model, but there are {}",
        self.models.len()
      )))
    }
  }

  fn try_one(&self) -> Option<&To::Model> {
    if self.models.is_empty() {
      None
    } else {
      Some(&self.models[0])
    }
  }
}

impl<From: EntityTrait, To: EntityTrait> ExpectModels<To::Model>
  for Option<EntityLinkLoaderResult<From, To>>
where
  <<From as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
{
  fn expect_models(&self) -> Result<&Vec<To::Model>, async_graphql::Error> {
    if let Some(result) = self {
      Ok(&result.models)
    } else {
      Err(async_graphql::Error::new(
        "EntityLinkLoader did not insert an expected key!  This should never happen; this is a bug in EntityLinkLoader.",
      ))
    }
  }

  fn expect_one(&self) -> Result<&To::Model, async_graphql::Error> {
    if let Some(result) = self {
      result.expect_one()
    } else {
      Err(async_graphql::Error::new(
        "EntityLinkLoader did not insert an expected key!  This should never happen; this is a bug in EntityLinkLoader.",
      ))
    }
  }

  fn try_one(&self) -> Option<&To::Model> {
    self.as_ref().and_then(|result| result.try_one())
  }
}

impl<From: EntityTrait, To: EntityTrait> ExpectModels<To::Model>
  for Option<&EntityLinkLoaderResult<From, To>>
where
  <<From as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
{
  fn expect_models(&self) -> Result<&Vec<To::Model>, async_graphql::Error> {
    if let Some(result) = self {
      Ok(&result.models)
    } else {
      Err(async_graphql::Error::new(
        "EntityLinkLoader did not insert an expected key!  This should never happen; this is a bug in EntityLinkLoader.",
      ))
    }
  }

  fn expect_one(&self) -> Result<&To::Model, async_graphql::Error> {
    if let Some(result) = self {
      result.expect_one()
    } else {
      Err(async_graphql::Error::new(
        "EntityLinkLoader did not insert an expected key!  This should never happen; this is a bug in EntityLinkLoader.",
      ))
    }
  }

  fn try_one(&self) -> Option<&To::Model> {
    self.as_ref().and_then(|result| result.try_one())
  }
}

#[derive(Debug)]
pub struct EntityLinkLoader<Link: Linked> {
  pub db: ConnectionWrapper,
  pub primary_key: <Link::FromEntity as EntityTrait>::PrimaryKey,
  pub link: Link,
}

impl<Link: Linked> EntityLinkLoader<Link> {
  pub fn new(
    db: ConnectionWrapper,
    link: Link,
    primary_key: <Link::FromEntity as EntityTrait>::PrimaryKey,
  ) -> EntityLinkLoader<Link> {
    EntityLinkLoader {
      db,
      link,
      primary_key,
    }
  }
}

#[async_trait::async_trait]
impl<Link: Linked>
  Loader<<<Link::FromEntity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>
  for EntityLinkLoader<Link>
where
  <Link::FromEntity as EntityTrait>::Model: Sync,
  <Link::ToEntity as EntityTrait>::Model: Sync,
  Link: 'static + Send + Sync + Clone,
  <Link::FromEntity as EntityTrait>::PrimaryKey: PrimaryKeyToColumn,
  <<Link::FromEntity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType:
    Sync + Clone + Eq + std::hash::Hash + IntoValueTuple + std::convert::From<i64>,
{
  type Value = EntityLinkLoaderResult<Link::FromEntity, Link::ToEntity>;
  type Error = Arc<sea_orm::DbErr>;

  async fn load(
    &self,
    keys: &[<<Link::FromEntity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType],
  ) -> Result<
    HashMap<
      <<Link::FromEntity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
      EntityLinkLoaderResult<Link::FromEntity, Link::ToEntity>,
    >,
    Self::Error,
  > {
    let pk_column = self.primary_key.into_column();

    Ok(load_all_linked(pk_column, keys, &self.link, self.db.as_ref()).await?)
  }
}
