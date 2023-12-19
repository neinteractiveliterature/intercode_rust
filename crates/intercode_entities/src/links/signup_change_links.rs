use sea_orm::{Linked, RelationDef, RelationTrait};

use crate::signup_changes;

#[derive(Debug, Clone)]
pub struct SignupChangeToPreviousSignupChange;

impl Linked for SignupChangeToPreviousSignupChange {
  type FromEntity = signup_changes::Entity;
  type ToEntity = signup_changes::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![signup_changes::Relation::SelfRef.def()]
  }
}
