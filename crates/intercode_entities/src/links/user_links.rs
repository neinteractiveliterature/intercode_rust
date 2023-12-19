use sea_orm::{Linked, RelationDef, RelationTrait};

use crate::{event_proposals, user_con_profiles, users};

#[derive(Debug, Clone)]
pub struct UserToEventProposals;

impl Linked for UserToEventProposals {
  type FromEntity = users::Entity;
  type ToEntity = event_proposals::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      users::Relation::UserConProfiles.def(),
      user_con_profiles::Relation::EventProposals.def(),
    ]
  }
}
