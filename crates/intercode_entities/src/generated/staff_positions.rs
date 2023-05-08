//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Default)]
#[sea_orm(table_name = "staff_positions")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub convention_id: Option<i64>,
  #[sea_orm(column_type = "Text", nullable)]
  pub name: Option<String>,
  #[sea_orm(column_type = "Text", nullable)]
  pub email: Option<String>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
  pub visible: Option<bool>,
  pub cc_addresses: Vec<String>,
  pub email_aliases: Vec<String>,
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
  #[sea_orm(has_many = "super::notification_destinations::Entity")]
  NotificationDestinations,
  #[sea_orm(has_many = "super::permissions::Entity")]
  Permissions,
}

impl Related<super::conventions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Conventions.def()
  }
}

impl Related<super::notification_destinations::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::NotificationDestinations.def()
  }
}

impl Related<super::permissions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Permissions.def()
  }
}

impl Related<super::user_con_profiles::Entity> for Entity {
  fn to() -> RelationDef {
    super::staff_positions_user_con_profiles::Relation::UserConProfiles.def()
  }
  fn via() -> Option<RelationDef> {
    Some(
      super::staff_positions_user_con_profiles::Relation::StaffPositions
        .def()
        .rev(),
    )
  }
}

impl ActiveModelBehavior for ActiveModel {}
