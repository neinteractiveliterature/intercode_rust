use crate::{form_items, form_sections, forms};
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
