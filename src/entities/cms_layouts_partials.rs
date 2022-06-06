//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "cms_layouts_partials")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub cms_partial_id: i64,
  #[sea_orm(primary_key, auto_increment = false)]
  pub cms_layout_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::cms_partials::Entity",
    from = "Column::CmsPartialId",
    to = "super::cms_partials::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  CmsPartials,
  #[sea_orm(
    belongs_to = "super::cms_layouts::Entity",
    from = "Column::CmsLayoutId",
    to = "super::cms_layouts::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  CmsLayouts,
}

impl Related<super::cms_partials::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::CmsPartials.def()
  }
}

impl Related<super::cms_layouts::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::CmsLayouts.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
