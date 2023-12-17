use crate::{conventions, events, runs, ticket_types, tickets};
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

#[derive(Debug, Clone)]
pub struct TicketToConvention;

impl Linked for TicketToConvention {
  type FromEntity = tickets::Entity;
  type ToEntity = conventions::Entity;

  fn link(&self) -> Vec<sea_orm::LinkDef> {
    vec![
      tickets::Relation::TicketTypes.def(),
      ticket_types::Relation::Conventions.def(),
    ]
  }
}

#[derive(Debug, Clone)]
pub struct TicketToRun;

impl Linked for TicketToRun {
  type FromEntity = tickets::Entity;
  type ToEntity = runs::Entity;

  fn link(&self) -> Vec<sea_orm::LinkDef> {
    vec![tickets::Relation::Runs.def()]
  }
}
