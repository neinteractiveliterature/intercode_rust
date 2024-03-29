//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "form_response_changes")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub user_con_profile_id: i64,
  pub field_identifier: String,
  pub previous_value: Option<Json>,
  pub new_value: Option<Json>,
  pub notified_at: Option<DateTime>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
  pub response_type: Option<String>,
  pub response_id: Option<i64>,
  pub compacted: bool,
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
}

impl Related<super::user_con_profiles::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::UserConProfiles.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
