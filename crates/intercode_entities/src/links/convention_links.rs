use crate::{
  conventions, events, orders, runs, signup_requests, signups, staff_positions, user_con_profiles,
};
use sea_orm::{
  sea_query::{Expr, IntoCondition},
  Linked, RelationDef, RelationTrait,
};

#[derive(Debug, Clone)]
pub struct ConventionToCatchAllStaffPosition;

impl Linked for ConventionToCatchAllStaffPosition {
  type FromEntity = conventions::Entity;
  type ToEntity = staff_positions::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![conventions::Relation::StaffPositions.def()]
  }
}

#[derive(Debug, Clone)]
pub struct ConventionToOrders;

impl Linked for ConventionToOrders {
  type FromEntity = conventions::Entity;
  type ToEntity = orders::Entity;

  fn link(&self) -> Vec<sea_orm::LinkDef> {
    vec![
      conventions::Relation::UserConProfiles.def(),
      user_con_profiles::Relation::Orders.def(),
    ]
  }
}

#[derive(Debug, Clone)]
pub struct ConventionToStaffPositions;

impl Linked for ConventionToStaffPositions {
  type FromEntity = conventions::Entity;
  type ToEntity = staff_positions::Entity;

  fn link(&self) -> Vec<RelationDef> {
    vec![staff_positions::Relation::Conventions.def().rev()]
  }
}

#[derive(Debug, Clone)]
pub struct ConventionToSignups;

impl Linked for ConventionToSignups {
  type FromEntity = conventions::Entity;
  type ToEntity = signups::Entity;

  fn link(&self) -> Vec<sea_orm::LinkDef> {
    vec![
      conventions::Relation::Events.def(),
      events::Relation::Runs.def(),
      runs::Relation::Signups.def(),
    ]
  }
}

#[derive(Debug, Clone)]
pub struct ConventionToSignupRequests;

impl Linked for ConventionToSignupRequests {
  type FromEntity = conventions::Entity;
  type ToEntity = signup_requests::Entity;

  fn link(&self) -> Vec<sea_orm::LinkDef> {
    vec![
      conventions::Relation::Events.def(),
      events::Relation::Runs.def(),
      runs::Relation::SignupRequests.def(),
    ]
  }
}

#[derive(Debug, Clone)]
pub struct ConventionToSingleEvent;

impl Linked for ConventionToSingleEvent {
  type FromEntity = conventions::Entity;
  type ToEntity = events::Entity;

  fn link(&self) -> Vec<sea_orm::LinkDef> {
    vec![conventions::Relation::Events
      .def()
      .on_condition(|conventions_table, _events_table| {
        Expr::col((conventions_table, conventions::Column::SiteMode))
          .eq("single_event")
          .into_condition()
      })]
  }
}
