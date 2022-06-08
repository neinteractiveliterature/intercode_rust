#[macro_use]
mod entities_by_id_loader;

use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dataloader::DataLoader;
pub use entities_by_id_loader::*;
use sea_orm::EntityTrait;

use crate::conventions;
use crate::staff_positions;
use crate::team_members;
use crate::users;

impl_to_entity_id_loader!(conventions::Entity, conventions::PrimaryKey::Id);
impl_to_entity_id_loader!(staff_positions::Entity, staff_positions::PrimaryKey::Id);
impl_to_entity_id_loader!(team_members::Entity, team_members::PrimaryKey::Id);
impl_to_entity_id_loader!(users::Entity, users::PrimaryKey::Id);

pub struct LoaderGroup {
  db: Arc<sea_orm::DatabaseConnection>,
  loaders: HashMap<dyn EntityTrait, dyn DataLoader>,
}
