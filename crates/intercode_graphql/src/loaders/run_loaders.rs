use intercode_entities::{events, rooms, rooms_runs, runs};
use sea_orm::{Linked, RelationDef, RelationTrait};

impl_to_entity_id_loader!(runs::Entity, runs::PrimaryKey::Id);

impl_to_entity_relation_loader!(runs::Entity, events::Entity, runs::PrimaryKey::Id);

#[derive(Debug, Clone)]
pub struct RunToRooms;

impl Linked for RunToRooms {
  type FromEntity = runs::Entity;
  type ToEntity = rooms::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      rooms_runs::Relation::Runs.def().rev(),
      rooms_runs::Relation::Rooms.def(),
    ]
  }
}

impl_to_entity_link_loader!(
  runs::Entity,
  RunToRooms,
  rooms::Entity,
  runs::PrimaryKey::Id
);
