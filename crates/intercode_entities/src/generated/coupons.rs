//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0
#![allow(clippy::derive_partial_eq_without_eq)]

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "coupons")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub convention_id: i64,
  #[sea_orm(column_type = "Text")]
  pub code: String,
  pub provides_product_id: Option<i64>,
  pub fixed_amount_cents: Option<i32>,
  pub fixed_amount_currency: Option<String>,
  pub percent_discount: Option<Decimal>,
  pub usage_limit: Option<i32>,
  pub expires_at: Option<DateTime>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::products::Entity",
    from = "Column::ProvidesProductId",
    to = "super::products::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Products,
  #[sea_orm(
    belongs_to = "super::conventions::Entity",
    from = "Column::ConventionId",
    to = "super::conventions::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Conventions,
  #[sea_orm(has_many = "super::coupon_applications::Entity")]
  CouponApplications,
}

impl Related<super::products::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Products.def()
  }
}

impl Related<super::conventions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Conventions.def()
  }
}

impl Related<super::coupon_applications::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::CouponApplications.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
