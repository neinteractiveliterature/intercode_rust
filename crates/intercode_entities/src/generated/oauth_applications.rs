//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Default)]
#[sea_orm(table_name = "oauth_applications")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub name: String,
  pub uid: String,
  pub secret: String,
  #[sea_orm(column_type = "Text")]
  pub redirect_uri: String,
  pub scopes: String,
  pub confidential: bool,
  pub created_at: DateTime,
  pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::oauth_access_grants::Entity")]
  OauthAccessGrants,
  #[sea_orm(has_many = "super::oauth_access_tokens::Entity")]
  OauthAccessTokens,
}

impl Related<super::oauth_access_grants::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OauthAccessGrants.def()
  }
}

impl Related<super::oauth_access_tokens::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OauthAccessTokens.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
