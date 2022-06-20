//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "cms_content_groups")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub name: String,
  pub parent_type: Option<String>,
  pub parent_id: Option<i64>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::cms_content_group_associations::Entity")]
  CmsContentGroupAssociations,
  #[sea_orm(has_many = "super::permissions::Entity")]
  Permissions,
}

impl Related<super::cms_content_group_associations::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::CmsContentGroupAssociations.def()
  }
}

impl Related<super::permissions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Permissions.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}