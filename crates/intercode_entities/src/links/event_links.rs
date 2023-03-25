use crate::{events, team_members, tickets, user_con_profiles};
use sea_orm::{Linked, RelationDef, RelationTrait};

#[derive(Debug, Clone)]
pub struct EventToProvidedTickets;

impl Linked for EventToProvidedTickets {
  type FromEntity = events::Entity;
  type ToEntity = tickets::Entity;

  fn link(&self) -> Vec<sea_orm::LinkDef> {
    vec![tickets::Relation::Events1.def().rev()]
  }
}

#[derive(Debug, Clone)]
pub struct EventToTeamMemberUserConProfiles;

impl Linked for EventToTeamMemberUserConProfiles {
  type FromEntity = events::Entity;
  type ToEntity = user_con_profiles::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      team_members::Relation::Events.def().rev(),
      team_members::Relation::UserConProfiles.def(),
    ]
  }
}
