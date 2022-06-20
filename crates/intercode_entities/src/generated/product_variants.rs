//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "product_variants")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub product_id: Option<i64>,
  #[sea_orm(column_type = "Text", nullable)]
  pub name: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub description: Option<String>,
  pub image: Option<String>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
  pub position: Option<i32>,
  pub override_pricing_structure: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::products::Entity",
    from = "Column::ProductId",
    to = "super::products::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Products,
  #[sea_orm(has_many = "super::order_entries::Entity")]
  OrderEntries,
}

impl Related<super::products::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Products.def()
  }
}

impl Related<super::order_entries::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OrderEntries.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}