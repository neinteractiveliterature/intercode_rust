//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "form_items")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub form_section_id: Option<i64>,
  pub position: i32,
  #[sea_orm(column_type = "Text", nullable)]
  pub identifier: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub item_type: Option<String>,
  pub properties: Option<Json>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
  #[sea_orm(column_type = "Text", nullable)]
  pub admin_description: Option<String>,
  pub default_value: Option<Json>,
  #[sea_orm(column_type = "Text", nullable)]
  pub public_description: Option<String>,
  pub visibility: String,
  pub writeability: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::form_sections::Entity",
    from = "Column::FormSectionId",
    to = "super::form_sections::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  FormSections,
}

impl Related<super::form_sections::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::FormSections.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}