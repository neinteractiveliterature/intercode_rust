use std::sync::Arc;

use crate::loaders::{EntityIdLoader, ToEntityIdLoader};
use crate::users;
use sea_orm;

impl ToEntityIdLoader<users::PrimaryKey> for users::Entity {
  type EntityIdLoaderType = EntityIdLoader<users::Entity, users::PrimaryKey>;

  fn to_entity_id_loader(
    self: &Self,
    db: Arc<sea_orm::DatabaseConnection>,
  ) -> Self::EntityIdLoaderType {
    EntityIdLoader::<users::Entity, users::PrimaryKey>::new(db, users::PrimaryKey::Id)
  }
}
