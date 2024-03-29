//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "oauth_access_tokens")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub resource_owner_id: Option<i64>,
  pub application_id: Option<i64>,
  pub token: String,
  pub refresh_token: Option<String>,
  pub expires_in: Option<i32>,
  pub revoked_at: Option<DateTime>,
  pub created_at: DateTime,
  pub scopes: Option<String>,
  pub previous_refresh_token: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::oauth_applications::Entity",
    from = "Column::ApplicationId",
    to = "super::oauth_applications::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  OauthApplications,
}

impl Related<super::oauth_applications::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OauthApplications.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
