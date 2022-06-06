//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "deprecated_graph_ql_usages")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[sea_orm(column_type = "Text", nullable)]
  pub operation_name: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub graphql_type: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub field_name: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub argument_name: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub user_agent: Option<String>,
  #[sea_orm(column_type = "Custom(\"inet\".to_owned())", nullable)]
  pub client_address: Option<String>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
  fn def(&self) -> RelationDef {
    panic!("No RelationDef")
  }
}

impl ActiveModelBehavior for ActiveModel {}
