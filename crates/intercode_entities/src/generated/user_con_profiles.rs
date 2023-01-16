//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Default)]
#[sea_orm(table_name = "user_con_profiles")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub user_id: i64,
  pub convention_id: i64,
  pub created_at: Option<DateTime>,
  pub updated_at: Option<DateTime>,
  pub first_name: String,
  pub last_name: String,
  pub nickname: Option<String>,
  pub birth_date: Option<Date>,
  pub gender: Option<String>,
  pub city: Option<String>,
  pub state: Option<String>,
  pub zipcode: Option<String>,
  pub country: Option<String>,
  pub day_phone: Option<String>,
  pub evening_phone: Option<String>,
  pub best_call_time: Option<String>,
  pub preferred_contact: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub bio: Option<String>,
  pub show_nickname_in_bio: Option<bool>,
  #[sea_orm(column_type = "Text", nullable)]
  pub address: Option<String>,
  pub additional_info: Option<Json>,
  pub receive_whos_free_emails: bool,
  pub gravatar_enabled: bool,
  #[sea_orm(column_type = "Text")]
  pub ical_secret: String,
  pub needs_update: bool,
  pub accepted_clickwrap_agreement: bool,
  pub mobile_phone: Option<String>,
  pub allow_sms: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::users::Entity",
    from = "Column::UserId",
    to = "super::users::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Users,
  #[sea_orm(
    belongs_to = "super::conventions::Entity",
    from = "Column::ConventionId",
    to = "super::conventions::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Conventions,
  #[sea_orm(has_many = "super::event_proposals::Entity")]
  EventProposals,
  #[sea_orm(has_many = "super::event_ratings::Entity")]
  EventRatings,
  #[sea_orm(has_many = "super::form_response_changes::Entity")]
  FormResponseChanges,
  #[sea_orm(has_many = "super::notification_destinations::Entity")]
  NotificationDestinations,
  #[sea_orm(has_many = "super::orders::Entity")]
  Orders,
  #[sea_orm(has_many = "super::signup_changes::Entity")]
  SignupChanges,
  #[sea_orm(has_many = "super::signups::Entity")]
  Signups,
  #[sea_orm(has_many = "super::tickets::Entity")]
  Tickets,
  #[sea_orm(has_many = "super::team_members::Entity")]
  TeamMembers,
  #[sea_orm(has_many = "super::signup_requests::Entity")]
  SignupRequests,
}

impl Related<super::users::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Users.def()
  }
}

impl Related<super::conventions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Conventions.def()
  }
}

impl Related<super::event_proposals::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::EventProposals.def()
  }
}

impl Related<super::event_ratings::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::EventRatings.def()
  }
}

impl Related<super::form_response_changes::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::FormResponseChanges.def()
  }
}

impl Related<super::notification_destinations::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::NotificationDestinations.def()
  }
}

impl Related<super::orders::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Orders.def()
  }
}

impl Related<super::signup_changes::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::SignupChanges.def()
  }
}

impl Related<super::signups::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Signups.def()
  }
}

impl Related<super::tickets::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Tickets.def()
  }
}

impl Related<super::team_members::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::TeamMembers.def()
  }
}

impl Related<super::signup_requests::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::SignupRequests.def()
  }
}

impl Related<super::staff_positions::Entity> for Entity {
  fn to() -> RelationDef {
    super::staff_positions_user_con_profiles::Relation::StaffPositions.def()
  }
  fn via() -> Option<RelationDef> {
    Some(
      super::staff_positions_user_con_profiles::Relation::UserConProfiles
        .def()
        .rev(),
    )
  }
}

impl ActiveModelBehavior for ActiveModel {}
