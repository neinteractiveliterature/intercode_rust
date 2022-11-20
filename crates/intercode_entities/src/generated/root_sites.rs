//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "root_sites")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[sea_orm(column_type = "Text", nullable)]
  pub site_name: Option<String>,
  pub root_page_id: Option<i64>,
  pub default_layout_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::pages::Entity",
    from = "Column::RootPageId",
    to = "super::pages::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Pages,
  #[sea_orm(
    belongs_to = "super::cms_layouts::Entity",
    from = "Column::DefaultLayoutId",
    to = "super::cms_layouts::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  CmsLayouts,
}

impl Related<super::pages::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Pages.def()
  }
}

impl Related<super::cms_layouts::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::CmsLayouts.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
