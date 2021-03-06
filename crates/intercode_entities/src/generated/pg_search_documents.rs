//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "pg_search_documents")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[sea_orm(column_type = "Text", nullable)]
  pub content: Option<String>,
  pub convention_id: Option<i64>,
  pub searchable_type: Option<String>,
  pub searchable_id: Option<i64>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
  #[sea_orm(column_type = "Custom(\"tsvector\".to_owned())", nullable)]
  pub content_vector: Option<String>,
  pub hidden_from_search: bool,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
  fn def(&self) -> RelationDef {
    panic!("No RelationDef")
  }
}

impl ActiveModelBehavior for ActiveModel {}
