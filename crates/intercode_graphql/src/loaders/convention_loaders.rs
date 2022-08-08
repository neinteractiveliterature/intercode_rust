use intercode_entities::*;
use sea_orm::{Linked, RelationDef, RelationTrait};

impl_to_entity_relation_loader!(
  conventions::Entity,
  event_categories::Entity,
  conventions::PrimaryKey::Id
);

#[derive(Debug, Clone)]
pub struct ConventionToStaffPositions;

impl Linked for ConventionToStaffPositions {
  type FromEntity = conventions::Entity;
  type ToEntity = staff_positions::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![staff_positions::Relation::Conventions.def().rev()]
  }
}

impl_to_entity_link_loader!(
  conventions::Entity,
  ConventionToStaffPositions,
  staff_positions::Entity,
  conventions::PrimaryKey::Id
);

impl_to_entity_relation_loader!(
  conventions::Entity,
  ticket_types::Entity,
  conventions::PrimaryKey::Id
);

impl_to_entity_id_loader!(conventions::Entity, conventions::PrimaryKey::Id);
