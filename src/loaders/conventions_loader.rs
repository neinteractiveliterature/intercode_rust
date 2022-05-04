use std::sync::Arc;

use crate::conventions;
use crate::loaders::{EntityIdLoader, ToEntityIdLoader};
use sea_orm;

impl ToEntityIdLoader<conventions::PrimaryKey> for conventions::Entity {
  type EntityIdLoaderType = EntityIdLoader<conventions::Entity, conventions::PrimaryKey>;

  fn to_entity_id_loader(
    self: &Self,
    db: Arc<sea_orm::DatabaseConnection>,
  ) -> Self::EntityIdLoaderType {
    EntityIdLoader::<conventions::Entity, conventions::PrimaryKey>::new(
      db,
      conventions::PrimaryKey::Id,
    )
  }
}
