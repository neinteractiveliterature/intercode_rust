//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0
#![allow(clippy::derive_partial_eq_without_eq)]

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "cms_files")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub parent_id: Option<i64>,
  pub uploader_id: Option<i64>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
  pub parent_type: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::users::Entity",
    from = "Column::UploaderId",
    to = "super::users::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Users,
  #[sea_orm(has_many = "super::cms_files_layouts::Entity")]
  CmsFilesLayouts,
  #[sea_orm(has_many = "super::cms_files_pages::Entity")]
  CmsFilesPages,
}

impl Related<super::users::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Users.def()
  }
}

impl Related<super::cms_files_layouts::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::CmsFilesLayouts.def()
  }
}

impl Related<super::cms_files_pages::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::CmsFilesPages.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
