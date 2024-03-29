//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Default)]
#[sea_orm(table_name = "events")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub title: String,
  pub author: Option<String>,
  pub email: Option<String>,
  pub organization: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub url: Option<String>,
  pub length_seconds: i32,
  pub can_play_concurrently: bool,
  pub con_mail_destination: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub description: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub short_blurb: Option<String>,
  pub updated_by_id: Option<i64>,
  pub created_at: Option<DateTime>,
  pub updated_at: Option<DateTime>,
  pub convention_id: i64,
  pub owner_id: Option<i64>,
  pub status: String,
  pub registration_policy: Option<Json>,
  #[sea_orm(column_type = "Text", nullable)]
  pub participant_communications: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub age_restrictions_description: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub content_warnings: Option<String>,
  pub additional_info: Option<Json>,
  #[sea_orm(column_type = "Text", nullable)]
  pub admin_notes: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub team_mailing_list_name: Option<String>,
  pub private_signup_list: bool,
  pub event_category_id: i64,
  pub minimum_age: Option<i32>,
  #[sea_orm(ignore)]
  pub title_vector: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::conventions::Entity",
    from = "Column::ConventionId",
    to = "super::conventions::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Conventions,
  #[sea_orm(
    belongs_to = "super::users::Entity",
    from = "Column::UpdatedById",
    to = "super::users::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Users2,
  #[sea_orm(
    belongs_to = "super::event_categories::Entity",
    from = "Column::EventCategoryId",
    to = "super::event_categories::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  EventCategories,
  #[sea_orm(
    belongs_to = "super::users::Entity",
    from = "Column::OwnerId",
    to = "super::users::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Users1,
  #[sea_orm(has_many = "super::event_proposals::Entity")]
  EventProposals,
  #[sea_orm(has_many = "super::event_ratings::Entity")]
  EventRatings,
  #[sea_orm(has_many = "super::runs::Entity")]
  Runs,
  #[sea_orm(has_many = "super::ticket_types::Entity")]
  TicketTypes,
  #[sea_orm(has_many = "super::team_members::Entity")]
  TeamMembers,
  #[sea_orm(has_many = "super::maximum_event_provided_tickets_overrides::Entity")]
  MaximumEventProvidedTicketsOverrides,
}

impl Related<super::conventions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Conventions.def()
  }
}

impl Related<super::event_categories::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::EventCategories.def()
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

impl Related<super::runs::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Runs.def()
  }
}

impl Related<super::ticket_types::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::TicketTypes.def()
  }
}

impl Related<super::team_members::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::TeamMembers.def()
  }
}

impl Related<super::maximum_event_provided_tickets_overrides::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::MaximumEventProvidedTicketsOverrides.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
