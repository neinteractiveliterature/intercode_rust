//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "pages")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[sea_orm(column_type = "Text", nullable)]
  pub name: Option<String>,
  pub slug: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub content: Option<String>,
  pub parent_id: Option<i32>,
  pub parent_type: Option<String>,
  pub created_at: Option<DateTime>,
  pub updated_at: Option<DateTime>,
  pub cms_layout_id: Option<i64>,
  #[sea_orm(column_type = "Text", nullable)]
  pub admin_notes: Option<String>,
  pub invariant: bool,
  pub skip_clickwrap_agreement: bool,
  pub hidden_from_search: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::cms_layouts::Entity",
    from = "Column::CmsLayoutId",
    to = "super::cms_layouts::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  CmsLayouts,
  #[sea_orm(has_many = "super::cms_navigation_items::Entity")]
  CmsNavigationItems,
  #[sea_orm(has_many = "super::conventions::Entity")]
  Conventions,
  #[sea_orm(has_many = "super::root_sites::Entity")]
  RootSites,
}

impl Related<super::cms_layouts::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::CmsLayouts.def()
  }
}

impl Related<super::cms_navigation_items::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::CmsNavigationItems.def()
  }
}

impl Related<super::conventions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Conventions.def()
  }
}

impl Related<super::root_sites::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::RootSites.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
