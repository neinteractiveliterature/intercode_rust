//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "cms_layouts")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub parent_type: Option<String>,
  pub parent_id: Option<i64>,
  #[sea_orm(column_type = "Text", nullable)]
  pub name: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub content: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub navbar_classes: Option<String>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
  #[sea_orm(column_type = "Text", nullable)]
  pub admin_notes: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::cms_files_layouts::Entity")]
  CmsFilesLayouts,
  #[sea_orm(has_many = "super::conventions::Entity")]
  Conventions,
  #[sea_orm(has_many = "super::pages::Entity")]
  Pages,
  #[sea_orm(has_many = "super::root_sites::Entity")]
  RootSites,
  #[sea_orm(has_many = "super::cms_layouts_partials::Entity")]
  CmsLayoutsPartials,
}

impl Related<super::cms_files_layouts::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::CmsFilesLayouts.def()
  }
}

impl Related<super::conventions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Conventions.def()
  }
}

impl Related<super::pages::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Pages.def()
  }
}

impl Related<super::root_sites::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::RootSites.def()
  }
}

impl Related<super::cms_layouts_partials::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::CmsLayoutsPartials.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}