#[macro_use]
mod entities_by_id_loader;

use std::sync::Arc;

use async_graphql::dataloader::DataLoader;
pub use entities_by_id_loader::*;

use crate::conventions;
use crate::staff_positions;
use crate::team_members;
use crate::users;

impl_to_entity_id_loader!(conventions::Entity, conventions::PrimaryKey::Id);
impl_to_entity_id_loader!(staff_positions::Entity, staff_positions::PrimaryKey::Id);
impl_to_entity_id_loader!(team_members::Entity, team_members::PrimaryKey::Id);
impl_to_entity_id_loader!(users::Entity, users::PrimaryKey::Id);

pub struct LoaderManager {
  db: Arc<sea_orm::DatabaseConnection>,
  pub conventions_by_id: DataLoader<EntityIdLoader<conventions::Entity, conventions::PrimaryKey>>,
  pub staff_positions_by_id:
    DataLoader<EntityIdLoader<staff_positions::Entity, staff_positions::PrimaryKey>>,
  pub team_members_by_id:
    DataLoader<EntityIdLoader<team_members::Entity, team_members::PrimaryKey>>,
  pub users_by_id: DataLoader<EntityIdLoader<users::Entity, users::PrimaryKey>>,
}

impl LoaderManager {
  pub fn new(db: &Arc<sea_orm::DatabaseConnection>) -> Self {
    LoaderManager {
      db: db.clone(),
      conventions_by_id: DataLoader::new(
        conventions::Entity.to_entity_id_loader(db.clone()),
        tokio::spawn,
      ),
      staff_positions_by_id: DataLoader::new(
        staff_positions::Entity.to_entity_id_loader(db.clone()),
        tokio::spawn,
      ),
      team_members_by_id: DataLoader::new(
        team_members::Entity.to_entity_id_loader(db.clone()),
        tokio::spawn,
      ),
      users_by_id: DataLoader::new(users::Entity.to_entity_id_loader(db.clone()), tokio::spawn),
    }
  }
}

impl std::fmt::Debug for LoaderManager {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // DataLoader doesn't implement Debug, so we're just going to exclude the loaders from the debug output
    f.debug_struct("LoaderManager")
      .field("db", &self.db)
      .finish_non_exhaustive()
  }
}

impl Clone for LoaderManager {
  fn clone(&self) -> Self {
    LoaderManager::new(&self.db)
  }
}
