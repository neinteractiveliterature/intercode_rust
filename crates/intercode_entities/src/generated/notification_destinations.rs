//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0
#![allow(clippy::derive_partial_eq_without_eq)]

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "notification_destinations")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub source_type: String,
  pub source_id: i64,
  pub staff_position_id: Option<i64>,
  pub user_con_profile_id: Option<i64>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::staff_positions::Entity",
    from = "Column::StaffPositionId",
    to = "super::staff_positions::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  StaffPositions,
  #[sea_orm(
    belongs_to = "super::user_con_profiles::Entity",
    from = "Column::UserConProfileId",
    to = "super::user_con_profiles::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  UserConProfiles,
}

impl Related<super::staff_positions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::StaffPositions.def()
  }
}

impl Related<super::user_con_profiles::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::UserConProfiles.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
