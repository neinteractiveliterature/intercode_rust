use async_graphql::{async_trait, dataloader::Loader};
use sea_orm::{
  sea_query::{IntoValueTuple, ValueTuple},
  EntityTrait, FromQueryResult, Linked, PrimaryKeyToColumn, PrimaryKeyTrait, QuerySelect,
};
use std::{collections::HashMap, marker::PhantomData, sync::Arc};

use super::expect::ExpectModels;

#[derive(FromQueryResult)]
struct ParentModelIdOnly {
  pub parent_model_id: i64,
}

pub trait ToEntityLinkLoader<
  To: EntityTrait,
  Link: Linked<FromEntity = Self, ToEntity = To>,
  PK: PrimaryKeyTrait + PrimaryKeyToColumn,
> where
  Self: EntityTrait<PrimaryKey = PK>,
  <<Self as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType:
    Eq + Clone + std::hash::Hash + Sync,
{
  type EntityLinkLoaderType: Loader<
    <<Self as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
    Value = EntityLinkLoaderResult<Self, To>,
    Error = Arc<sea_orm::DbErr>,
  >;

  fn to_entity_link_loader(
    self: &Self,
    link: Link,
    db: Arc<sea_orm::DatabaseConnection>,
  ) -> Self::EntityLinkLoaderType;
}

#[macro_export]
macro_rules! impl_to_entity_link_loader {
  ($from: ty, $link: path, $to: ty, $pk: path) => {
    impl
      crate::loaders::entities_by_link_loader::ToEntityLinkLoader<
        $to,
        $link,
        <$from as sea_orm::EntityTrait>::PrimaryKey,
      > for $from
    {
      type EntityLinkLoaderType = crate::loaders::entities_by_link_loader::EntityLinkLoader<
        $from,
        $link,
        $to,
        <$from as sea_orm::EntityTrait>::PrimaryKey,
      >;

      fn to_entity_link_loader(
        self: &Self,
        link: $link,
        db: std::sync::Arc<sea_orm::DatabaseConnection>,
      ) -> Self::EntityLinkLoaderType {
        crate::loaders::entities_by_link_loader::EntityLinkLoader::<
          $from,
          $link,
          $to,
          <$from as sea_orm::EntityTrait>::PrimaryKey,
        >::new(db, link, $pk)
      }
    }
  };
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
  pub fn expect_one(self: &Self) -> Result<&To::Model, async_graphql::Error> {
    if self.models.len() == 1 {
      Ok(&self.models[0])
    } else {
      Err(async_graphql::Error::new(format!(
        "Expected one model, but there are {}",
        self.models.len()
      )))
    }
  }
}

impl<From: EntityTrait, To: EntityTrait> ExpectModels<To::Model>
  for Option<EntityLinkLoaderResult<From, To>>
where
  <<From as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
{
  fn expect_models(self: &Self) -> Result<&Vec<To::Model>, async_graphql::Error> {
    if let Some(result) = self {
      Ok(&result.models)
    } else {
      Err(async_graphql::Error::new(
        "EntityLinkLoader did not insert an expected key!  This should never happen; this is a bug in EntityLinkLoader.",
      ))
    }
  }

  fn expect_one(self: &Self) -> Result<&To::Model, async_graphql::Error> {
    if let Some(result) = self {
      result.expect_one()
    } else {
      Err(async_graphql::Error::new(
        "EntityLinkLoader did not insert an expected key!  This should never happen; this is a bug in EntityLinkLoader.",
      ))
    }
  }
}

#[derive(Debug)]
pub struct EntityLinkLoader<
  From: EntityTrait<PrimaryKey = PK>,
  Link: Linked<FromEntity = From, ToEntity = To>,
  To: EntityTrait,
  PK: PrimaryKeyTrait + PrimaryKeyToColumn,
> {
  pub db: Arc<sea_orm::DatabaseConnection>,
  pub primary_key: PK,
  pub link: Link,
  _from: PhantomData<From>,
  _to: PhantomData<To>,
}

impl<
    From: EntityTrait<PrimaryKey = PK>,
    Link: Linked<FromEntity = From, ToEntity = To>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn,
  > EntityLinkLoader<From, Link, To, PK>
{
  pub fn new(
    db: Arc<sea_orm::DatabaseConnection>,
    link: Link,
    primary_key: PK,
  ) -> EntityLinkLoader<From, Link, To, PK> {
    EntityLinkLoader::<From, Link, To, PK> {
      db,
      link,
      primary_key,
      _from: PhantomData::<From>,
      _to: PhantomData::<To>,
    }
  }
}

#[async_trait::async_trait]
impl<
    From: EntityTrait<PrimaryKey = PK>,
    Link: Linked<FromEntity = From, ToEntity = To>,
    To: EntityTrait,
    PK: PrimaryKeyTrait + PrimaryKeyToColumn<Column = From::Column>,
  > Loader<PK::ValueType> for EntityLinkLoader<From, Link, To, PK>
where
  <From as sea_orm::EntityTrait>::Model: Sync,
  <To as sea_orm::EntityTrait>::Model: Sync,
  Link: 'static + Send + Sync + Clone,
  PK::ValueType: Sync + Clone + Eq + std::hash::Hash + IntoValueTuple + std::convert::From<i64>,
{
  type Value = EntityLinkLoaderResult<From, To>;
  type Error = Arc<sea_orm::DbErr>;

  async fn load(
    &self,
    keys: &[PK::ValueType],
  ) -> Result<HashMap<PK::ValueType, EntityLinkLoaderResult<From, To>>, Self::Error> {
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;

    let pk_column = self.primary_key.into_column();
    let pk_values = keys.into_iter().map(|key| {
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
      .find_also_linked(self.link.clone())
      .into_model::<ParentModelIdOnly, To::Model>()
      .all(self.db.as_ref())
      .await?
      .into_iter()
      .fold(
        HashMap::<PK::ValueType, EntityLinkLoaderResult<From, To>>::new(),
        |mut acc: HashMap<PK::ValueType, EntityLinkLoaderResult<From, To>>,
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

    for id in keys.into_iter() {
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
}
