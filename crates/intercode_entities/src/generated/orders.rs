//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Default)]
#[sea_orm(table_name = "orders")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub user_con_profile_id: i64,
  pub status: String,
  pub charge_id: Option<String>,
  pub payment_amount_cents: Option<i32>,
  pub payment_amount_currency: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub payment_note: Option<String>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
  pub submitted_at: Option<DateTime>,
  pub paid_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::user_con_profiles::Entity",
    from = "Column::UserConProfileId",
    to = "super::user_con_profiles::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  UserConProfiles,
  #[sea_orm(has_many = "super::coupon_applications::Entity")]
  CouponApplications,
  #[sea_orm(has_many = "super::order_entries::Entity")]
  OrderEntries,
}

impl Related<super::user_con_profiles::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::UserConProfiles.def()
  }
}

impl Related<super::coupon_applications::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::CouponApplications.def()
  }
}

impl Related<super::order_entries::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OrderEntries.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
