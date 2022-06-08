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
  type EntityIdLoaderType: Loader<
    PK::ValueType,
    Value = EntityIdLoaderResult<Self, PK>,
    Error = Arc<sea_orm::DbErr>,
  >;

  fn to_entity_id_loader(
    self: &Self,
    db: Arc<sea_orm::DatabaseConnection>,
  ) -> Self::EntityIdLoaderType;
}

#[macro_export]
macro_rules! impl_to_entity_id_loader {
  ($e: path, $pk: path) => {
    impl ToEntityIdLoader<<$e as sea_orm::EntityTrait>::PrimaryKey> for $e {
      type EntityIdLoaderType = EntityIdLoader<$e, <$e as sea_orm::EntityTrait>::PrimaryKey>;

      fn to_entity_id_loader(
        self: &Self,
        db: std::sync::Arc<sea_orm::DatabaseConnection>,
      ) -> Self::EntityIdLoaderType {
        EntityIdLoader::<$e, <$e as sea_orm::EntityTrait>::PrimaryKey>::new(db, $pk)
      }
    }
  };
}

pub trait ExpectModel<M: ModelTrait> {
  fn expect_model(self: &Self) -> Result<M, async_graphql::Error>;
}

#[derive(Debug, Clone)]
pub struct EntityIdLoaderResult<E: EntityTrait<PrimaryKey = PK>, PK: PrimaryKeyTrait> {
  pub entity: E,
  pub id: PK::ValueType,
  pub model: Option<E::Model>,
}

impl<E: EntityTrait<PrimaryKey = PK>, PK: PrimaryKeyTrait> ExpectModel<E::Model>
  for EntityIdLoaderResult<E, PK>
{
  fn expect_model(self: &Self) -> Result<E::Model, async_graphql::Error> {
    if let Some(model) = &self.model {
      Ok(model.to_owned())
    } else {
      Err(async_graphql::Error::new(format!(
        "{} {:?} not found",
        self.entity.table_name(),
        self.id
      )))
    }
  }
}

impl<E: EntityTrait<PrimaryKey = PK>, PK: PrimaryKeyTrait> ExpectModel<E::Model>
  for Option<EntityIdLoaderResult<E, PK>>
{
  fn expect_model(self: &Self) -> Result<E::Model, async_graphql::Error> {
    if let Some(result) = self {
      result.expect_model()
    } else {
      Err(async_graphql::Error::new(
        "EntityIdLoader did not insert an expected key!  This should never happen; this is a bug in EntityIdLoader.",
      ))
    }
  }
}

#[derive(Debug)]
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
  type Value = EntityIdLoaderResult<E, PK>;
  type Error = Arc<sea_orm::DbErr>;

  async fn load(
    &self,
    keys: &[PK::ValueType],
  ) -> Result<HashMap<PK::ValueType, EntityIdLoaderResult<E, PK>>, Self::Error> {
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

    let mut results = E::find()
      .filter(pk_column.is_in(pk_values))
      .all(self.db.as_ref())
      .await?
      .into_iter()
      .map(|model: E::Model| {
        let id = PK::ValueType::from_value_tuple(model.get(pk_column.into()));

        (
          id.clone(),
          EntityIdLoaderResult {
            entity: E::default(),
            model: Some(model),
            id,
          },
        )
      })
      .collect::<HashMap<PK::ValueType, EntityIdLoaderResult<E, PK>>>();

    for key in keys.into_iter() {
      let tuple = key.clone().into_value_tuple();
      let id = PK::ValueType::from_value_tuple(tuple);

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
