use crate::{event_categories, forms};
use sea_orm::{Linked, RelationDef, RelationTrait};

#[derive(Debug, Clone)]
pub struct EventCategoryToEventForm;

impl Linked for EventCategoryToEventForm {
  type FromEntity = event_categories::Entity;
  type ToEntity = forms::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![event_categories::Relation::EventForm.def()]
  }
}

#[derive(Debug, Clone)]
pub struct EventCategoryToEventProposalForm;

impl Linked for EventCategoryToEventProposalForm {
  type FromEntity = event_categories::Entity;
  type ToEntity = forms::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![event_categories::Relation::EventProposalForm.def()]
  }
}
