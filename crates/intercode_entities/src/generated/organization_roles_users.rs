//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0
#![allow(clippy::derive_partial_eq_without_eq)]

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "organization_roles_users")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub organization_role_id: i64,
  #[sea_orm(primary_key, auto_increment = false)]
  pub user_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::users::Entity",
    from = "Column::UserId",
    to = "super::users::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Users,
  #[sea_orm(
    belongs_to = "super::organization_roles::Entity",
    from = "Column::OrganizationRoleId",
    to = "super::organization_roles::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  OrganizationRoles,
}

impl Related<super::users::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Users.def()
  }
}

impl Related<super::organization_roles::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OrganizationRoles.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
