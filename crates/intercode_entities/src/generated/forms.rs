//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Default)]
#[sea_orm(table_name = "forms")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[sea_orm(column_type = "Text", nullable)]
  pub title: Option<String>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
  pub convention_id: Option<i64>,
  pub form_type: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::conventions::Entity",
    from = "Column::ConventionId",
    to = "super::conventions::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Conventions,
  #[sea_orm(has_many = "super::form_sections::Entity")]
  FormSections,
}

impl Related<super::conventions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Conventions.def()
  }
}

impl Related<super::form_sections::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::FormSections.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
