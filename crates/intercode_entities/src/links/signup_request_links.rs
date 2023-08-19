use crate::{events, runs, signup_requests, signups};
use sea_orm::{Linked, RelationDef, RelationTrait};

#[derive(Debug, Clone)]
pub struct SignupRequestToResultSignup;

impl Linked for SignupRequestToResultSignup {
  type FromEntity = signup_requests::Entity;
  type ToEntity = signups::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![signup_requests::Relation::Signups1.def()]
  }
}

#[derive(Debug, Clone)]
pub struct SignupRequestToReplaceSignup;

impl Linked for SignupRequestToReplaceSignup {
  type FromEntity = signup_requests::Entity;
  type ToEntity = signups::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![signup_requests::Relation::Signups2.def()]
  }
}

#[derive(Debug, Clone)]
pub struct SignupRequestToEvent;

impl Linked for SignupRequestToEvent {
  type FromEntity = signup_requests::Entity;
  type ToEntity = events::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![
      signup_requests::Relation::Runs.def(),
      runs::Relation::Events.def(),
    ]
  }
}
