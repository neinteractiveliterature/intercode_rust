//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "ahoy_events")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub visit_id: Option<i64>,
  pub user_id: Option<i64>,
  pub name: Option<String>,
  pub properties: Option<Json>,
  pub time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
  fn def(&self) -> RelationDef {
    panic!("No RelationDef")
  }
}

impl ActiveModelBehavior for ActiveModel {}
