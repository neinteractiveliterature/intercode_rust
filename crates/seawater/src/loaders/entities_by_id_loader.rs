use async_graphql::{async_trait, dataloader::Loader};
use sea_orm::{
  sea_query::{IntoValueTuple, ValueTuple},
  EntityTrait, ModelTrait, PrimaryKeyToColumn, PrimaryKeyTrait,
};
use std::{collections::HashMap, sync::Arc};

use crate::ConnectionWrapper;

use super::expect::ExpectModel;

#[derive(Debug, Clone)]
pub struct EntityIdLoaderResult<E: EntityTrait>
where
  <E::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
{
  pub entity: E,
  pub id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType,
  pub model: Option<E::Model>,
}

impl<E: EntityTrait> ExpectModel<E::Model> for EntityIdLoaderResult<E>
where
  <E::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
{
  fn expect_one(&self) -> Result<&E::Model, async_graphql::Error> {
    if let Some(model) = &self.model {
      Ok(model)
    } else {
      Err(async_graphql::Error::new(format!(
        "{} {:?} not found",
        self.entity.table_name(),
        self.id
      )))
    }
  }

  fn try_one(&self) -> Option<&E::Model> {
    self.model.as_ref()
  }
}

impl<E: EntityTrait> ExpectModel<E::Model> for Option<EntityIdLoaderResult<E>>
where
  <E::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone,
{
  fn expect_one(&self) -> Result<&E::Model, async_graphql::Error> {
    if let Some(result) = self {
      result.expect_one()
    } else {
      Err(async_graphql::Error::new(
        "EntityIdLoader did not insert an expected key!  This should never happen; this is a bug in EntityIdLoader.",
      ))
    }
  }

  fn try_one(&self) -> Option<&E::Model> {
    self.as_ref().and_then(|result| result.try_one())
  }
}

#[derive(Debug)]
pub struct EntityIdLoader<E: EntityTrait> {
  pub db: ConnectionWrapper,
  pub primary_key: E::PrimaryKey,
}

impl<E: EntityTrait> EntityIdLoader<E> {
  pub fn new(db: ConnectionWrapper, primary_key: E::PrimaryKey) -> EntityIdLoader<E> {
    EntityIdLoader { db, primary_key }
  }
}

#[async_trait::async_trait]
impl<E: EntityTrait> Loader<<E::PrimaryKey as PrimaryKeyTrait>::ValueType> for EntityIdLoader<E>
where
  <E as sea_orm::EntityTrait>::Model: Sync,
  <E as sea_orm::EntityTrait>::Column: From<<E::PrimaryKey as PrimaryKeyToColumn>::Column>,
  E::PrimaryKey: PrimaryKeyToColumn,
  <E::PrimaryKey as PrimaryKeyTrait>::ValueType:
    Sync + Clone + Eq + std::hash::Hash + IntoValueTuple,
{
  type Value = EntityIdLoaderResult<E>;
  type Error = Arc<sea_orm::DbErr>;

  async fn load(
    &self,
    keys: &[<E::PrimaryKey as PrimaryKeyTrait>::ValueType],
  ) -> Result<
    HashMap<<E::PrimaryKey as PrimaryKeyTrait>::ValueType, EntityIdLoaderResult<E>>,
    Self::Error,
  > {
    use sea_orm::sea_query::FromValueTuple;
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;

    let pk_column = self.primary_key.into_column();
    let pk_values = keys.iter().map(|key| {
      let tuple = key.clone().into_value_tuple();
      if let ValueTuple::One(single_value) = tuple {
        single_value
      } else {
        panic!(
          "EntityIdLoader does not work with composite primary keys (encountered {:?})",
          tuple
        )
      }
    });

    let mut results = E::find()
      .filter(pk_column.is_in(pk_values))
      .all(self.db.as_ref())
      .await?
      .into_iter()
      .map(|model: E::Model| {
        let id =
          <E::PrimaryKey as PrimaryKeyTrait>::ValueType::from_value_tuple(model.get(pk_column));

        (
          id.clone(),
          EntityIdLoaderResult {
            entity: E::default(),
            model: Some(model),
            id,
          },
        )
      })
      .collect::<HashMap<<E::PrimaryKey as PrimaryKeyTrait>::ValueType, EntityIdLoaderResult<E>>>();

    for key in keys.iter() {
      let tuple = key.clone().into_value_tuple();
      let id = <E::PrimaryKey as PrimaryKeyTrait>::ValueType::from_value_tuple(tuple);

      if !results.contains_key(&id) {
        results.insert(
          id.clone(),
          EntityIdLoaderResult {
            entity: E::default(),
            model: None,
            id,
          },
        );
      }
    }

    Ok(results)
  }
}
