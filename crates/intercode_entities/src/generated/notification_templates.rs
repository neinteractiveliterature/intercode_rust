//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Default)]
#[sea_orm(table_name = "notification_templates")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub convention_id: i64,
  pub event_key: String,
  #[sea_orm(column_type = "Text", nullable)]
  pub subject: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub body_html: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub body_text: Option<String>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
  #[sea_orm(column_type = "Text", nullable)]
  pub body_sms: Option<String>,
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
}

impl Related<super::conventions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Conventions.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
