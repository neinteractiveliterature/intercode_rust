use crate::{conventions, staff_positions};
use sea_orm::{Linked, RelationDef, RelationTrait};

#[derive(Debug, Clone)]
pub struct ConventionToStaffPositions;

impl Linked for ConventionToStaffPositions {
  type FromEntity = conventions::Entity;
  type ToEntity = staff_positions::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![staff_positions::Relation::Conventions.def().rev()]
  }
}
