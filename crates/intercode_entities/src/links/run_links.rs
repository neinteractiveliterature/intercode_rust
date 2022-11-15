use crate::{rooms, rooms_runs, runs};
use sea_orm::{Linked, RelationDef, RelationTrait};

#[derive(Debug, Clone)]
pub struct RunToRooms;

impl Linked for RunToRooms {
  type FromEntity = runs::Entity;
  type ToEntity = rooms::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      runs::Relation::RoomsRuns.def(),
      rooms_runs::Relation::Rooms.def(),
    ]
  }
}
