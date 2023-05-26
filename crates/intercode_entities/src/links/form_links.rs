use crate::{conventions, event_categories, form_items, form_sections, forms};
use sea_orm::{Linked, RelationDef, RelationTrait};

#[derive(Debug, Clone)]
pub struct FormToFormItems;

impl Linked for FormToFormItems {
  type FromEntity = forms::Entity;
  type ToEntity = form_items::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      form_sections::Relation::Forms.def().rev(),
      form_sections::Relation::FormItems.def(),
    ]
  }
}

#[derive(Debug, Clone)]
pub struct FormToEventCategories;

impl Linked for FormToEventCategories {
  type FromEntity = forms::Entity;
  type ToEntity = event_categories::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![event_categories::Relation::EventForm.def().rev()]
  }
}

#[derive(Debug, Clone)]
pub struct FormToProposalEventCategories;

impl Linked for FormToProposalEventCategories {
  type FromEntity = forms::Entity;
  type ToEntity = event_categories::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![event_categories::Relation::EventProposalForm.def().rev()]
  }
}

#[derive(Debug, Clone)]
pub struct FormToUserConProfileConventions;

impl Linked for FormToUserConProfileConventions {
  type FromEntity = forms::Entity;
  type ToEntity = conventions::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![conventions::Relation::Forms.def().rev()]
  }
}
