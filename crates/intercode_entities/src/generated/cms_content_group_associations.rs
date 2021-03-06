//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "cms_content_group_associations")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub content_type: String,
  pub content_id: i64,
  pub cms_content_group_id: i64,
  pub created_at: DateTime,
  pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::cms_content_groups::Entity",
    from = "Column::CmsContentGroupId",
    to = "super::cms_content_groups::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  CmsContentGroups,
}

impl Related<super::cms_content_groups::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::CmsContentGroups.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
