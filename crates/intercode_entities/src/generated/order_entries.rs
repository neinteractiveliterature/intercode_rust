//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "order_entries")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub order_id: i64,
  pub product_id: i64,
  pub product_variant_id: Option<i64>,
  pub quantity: Option<i32>,
  pub price_per_item_cents: Option<i32>,
  pub price_per_item_currency: Option<String>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::product_variants::Entity",
    from = "Column::ProductVariantId",
    to = "super::product_variants::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  ProductVariants,
  #[sea_orm(
    belongs_to = "super::orders::Entity",
    from = "Column::OrderId",
    to = "super::orders::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Orders,
  #[sea_orm(
    belongs_to = "super::products::Entity",
    from = "Column::ProductId",
    to = "super::products::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Products,
  #[sea_orm(has_many = "super::tickets::Entity")]
  Tickets,
}

impl Related<super::product_variants::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::ProductVariants.def()
  }
}

impl Related<super::orders::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Orders.def()
  }
}

impl Related<super::products::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Products.def()
  }
}

impl Related<super::tickets::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Tickets.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
