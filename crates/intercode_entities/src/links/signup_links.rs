use crate::{events, runs, signups};
use sea_orm::{Linked, RelationDef, RelationTrait};

#[derive(Debug, Clone)]
pub struct SignupToEvent;

impl Linked for SignupToEvent {
  type FromEntity = signups::Entity;
  type ToEntity = events::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![signups::Relation::Runs.def(), runs::Relation::Events.def()]
  }
}
