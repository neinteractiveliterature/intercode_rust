//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "team_members")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub event_id: Option<i64>,
  pub updated_at: Option<DateTime>,
  pub updated_by_id: Option<i64>,
  pub display: Option<bool>,
  pub show_email: Option<bool>,
  pub receive_con_email: Option<bool>,
  pub created_at: Option<DateTime>,
  pub user_con_profile_id: i64,
  pub receive_signup_email: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::user_con_profiles::Entity",
    from = "Column::UserConProfileId",
    to = "super::user_con_profiles::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  UserConProfiles,
  #[sea_orm(
    belongs_to = "super::events::Entity",
    from = "Column::EventId",
    to = "super::events::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Events,
}

impl Related<super::user_con_profiles::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::UserConProfiles.def()
  }
}

impl Related<super::events::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Events.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
