use crate::{rooms, rooms_runs, runs};
use sea_orm::{Linked, RelationDef, RelationTrait};

#[derive(Debug, Clone)]
pub struct RoomToRuns;

impl Linked for RoomToRuns {
  type FromEntity = rooms::Entity;
  type ToEntity = runs::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      rooms::Relation::RoomsRuns.def(),
      rooms_runs::Relation::Runs.def(),
    ]
  }
}
