//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
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

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
