use async_graphql::{async_trait, dataloader::Loader};
use sea_orm::{
  sea_query::{IntoValueTuple, ValueTuple},
  EntityTrait, ModelTrait, PrimaryKeyToColumn, PrimaryKeyTrait,
};
use std::{collections::HashMap, marker::PhantomData, sync::Arc};

pub trait ToEntityIdLoader<PK: PrimaryKeyTrait>
where
  Self: EntityTrait<PrimaryKey = PK>,
  <Self as sea_orm::EntityTrait>::Model: Sync,
  PK::ValueType: Sync + Clone + Eq + std::hash::Hash + IntoValueTuple,
{
  type EntityIdLoaderType: Loader<PK::ValueType, Value = Self::Model, Error = Arc<sea_orm::DbErr>>;

  fn to_entity_id_loader(
    self: &Self,
    db: Arc<sea_orm::DatabaseConnection>,
  ) -> Self::EntityIdLoaderType;
}

pub struct EntityIdLoader<E: EntityTrait<PrimaryKey = PK>, PK: PrimaryKeyTrait> {
  pub db: Arc<sea_orm::DatabaseConnection>,
  pub primary_key: PK,
  _marker: PhantomData<E>,
}

impl<E: EntityTrait<PrimaryKey = PK>, PK: PrimaryKeyTrait> EntityIdLoader<E, PK> {
  pub fn new(db: Arc<sea_orm::DatabaseConnection>, primary_key: PK) -> EntityIdLoader<E, PK> {
    EntityIdLoader::<E, PK> {
      db,
      primary_key,
      _marker: PhantomData::<E>,
    }
  }
}

#[async_trait::async_trait]
impl<E: EntityTrait<PrimaryKey = PK>, PK: PrimaryKeyTrait> Loader<PK::ValueType>
  for EntityIdLoader<E, PK>
where
  <E as sea_orm::EntityTrait>::Model: Sync,
  <E as sea_orm::EntityTrait>::Column: From<<PK as PrimaryKeyToColumn>::Column>,
  PK: PrimaryKeyToColumn,
  PK::ValueType: Sync + Clone + Eq + std::hash::Hash + IntoValueTuple,
{
  type Value = <E as sea_orm::EntityTrait>::Model;
  type Error = Arc<sea_orm::DbErr>;

  async fn load(
    &self,
    keys: &[PK::ValueType],
  ) -> Result<HashMap<PK::ValueType, Self::Value>, Self::Error> {
    use sea_orm::sea_query::FromValueTuple;
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;

    let pk_column = self.primary_key.into_column();
    let pk_values = keys.into_iter().map(|key| {
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

    Ok(
      E::find()
        .filter(pk_column.is_in(pk_values))
        .all(self.db.as_ref())
        .await?
        .into_iter()
        .map(|model: E::Model| {
          (
            PK::ValueType::from_value_tuple(model.get(pk_column.into())),
            model,
          )
        })
        .collect(),
    )
  }
}
