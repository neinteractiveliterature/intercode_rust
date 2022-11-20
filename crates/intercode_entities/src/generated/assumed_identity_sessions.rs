//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "assumed_identity_sessions")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub assumed_profile_id: i64,
  pub assumer_profile_id: i64,
  #[sea_orm(column_type = "Text")]
  pub justification: String,
  pub started_at: DateTime,
  pub finished_at: Option<DateTime>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::user_con_profiles::Entity",
    from = "Column::AssumedProfileId",
    to = "super::user_con_profiles::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  UserConProfiles2,
  #[sea_orm(
    belongs_to = "super::user_con_profiles::Entity",
    from = "Column::AssumerProfileId",
    to = "super::user_con_profiles::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  UserConProfiles1,
  #[sea_orm(has_many = "super::assumed_identity_request_logs::Entity")]
  AssumedIdentityRequestLogs,
}

impl Related<super::assumed_identity_request_logs::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::AssumedIdentityRequestLogs.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
