use crate::{conventions, events, runs, signups, staff_positions};
use sea_orm::{Linked, RelationDef, RelationTrait};

#[derive(Debug, Clone)]
pub struct ConventionToCatchAllStaffPosition;

impl Linked for ConventionToCatchAllStaffPosition {
  type FromEntity = conventions::Entity;
  type ToEntity = staff_positions::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![conventions::Relation::StaffPositions.def()]
  }
}

#[derive(Debug, Clone)]
pub struct ConventionToStaffPositions;

impl Linked for ConventionToStaffPositions {
  type FromEntity = conventions::Entity;
  type ToEntity = staff_positions::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![staff_positions::Relation::Conventions.def().rev()]
  }
}

#[derive(Debug, Clone)]
pub struct ConventionToSignups;

impl Linked for ConventionToSignups {
  type FromEntity = conventions::Entity;
  type ToEntity = signups::Entity;

  fn link(&self) -> Vec<sea_orm::LinkDef> {
    vec![
      conventions::Relation::Events.def(),
      events::Relation::Runs.def(),
      runs::Relation::Signups.def(),
    ]
  }
}
