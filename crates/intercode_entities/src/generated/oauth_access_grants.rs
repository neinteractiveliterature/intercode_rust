//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "oauth_access_grants")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub resource_owner_id: i64,
  pub application_id: i64,
  pub token: String,
  pub expires_in: i32,
  #[sea_orm(column_type = "Text")]
  pub redirect_uri: String,
  pub created_at: DateTime,
  pub revoked_at: Option<DateTime>,
  pub scopes: Option<String>,
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
  #[sea_orm(has_many = "super::oauth_openid_requests::Entity")]
  OauthOpenidRequests,
}

impl Related<super::oauth_applications::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OauthApplications.def()
  }
}

impl Related<super::oauth_openid_requests::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OauthOpenidRequests.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}