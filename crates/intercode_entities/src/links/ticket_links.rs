use crate::{events, tickets};
use sea_orm::{Linked, RelationDef, RelationTrait};

#[derive(Debug, Clone)]
pub struct TicketToProvidedByEvent;

impl Linked for TicketToProvidedByEvent {
  type FromEntity = tickets::Entity;
  type ToEntity = events::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![tickets::Relation::Events1.def()]
  }
}
