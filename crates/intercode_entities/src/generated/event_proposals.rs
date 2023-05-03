//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

use crate::model_ext::event_proposals::EventProposalStatus;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Default)]
#[sea_orm(table_name = "event_proposals")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub convention_id: Option<i64>,
  pub owner_id: Option<i64>,
  pub event_id: Option<i64>,
  pub status: Option<EventProposalStatus>,
  #[sea_orm(column_type = "Text", nullable)]
  pub title: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub email: Option<String>,
  pub length_seconds: Option<i32>,
  #[sea_orm(column_type = "Text", nullable)]
  pub description: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub short_blurb: Option<String>,
  pub registration_policy: Option<Json>,
  pub can_play_concurrently: Option<bool>,
  pub additional_info: Option<Json>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
  pub timeblock_preferences: Option<Json>,
  pub submitted_at: Option<DateTime>,
  #[sea_orm(column_type = "Text", nullable)]
  pub admin_notes: Option<String>,
  pub reminded_at: Option<DateTime>,
  #[sea_orm(column_type = "Text", nullable)]
  pub team_mailing_list_name: Option<String>,
  pub event_category_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::events::Entity",
    from = "Column::EventId",
    to = "super::events::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Events,
  #[sea_orm(
    belongs_to = "super::event_categories::Entity",
    from = "Column::EventCategoryId",
    to = "super::event_categories::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  EventCategories,
  #[sea_orm(
    belongs_to = "super::user_con_profiles::Entity",
    from = "Column::OwnerId",
    to = "super::user_con_profiles::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  UserConProfiles,
  #[sea_orm(
    belongs_to = "super::conventions::Entity",
    from = "Column::ConventionId",
    to = "super::conventions::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Conventions,
}

impl Related<super::events::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Events.def()
  }
}

impl Related<super::event_categories::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::EventCategories.def()
  }
}

impl Related<super::user_con_profiles::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::UserConProfiles.def()
  }
}

impl Related<super::conventions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Conventions.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
