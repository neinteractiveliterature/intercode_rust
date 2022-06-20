//! SeaORM Entity. Generated by sea-orm-codegen 0.7.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "organization_roles")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub organization_id: Option<i64>,
  #[sea_orm(column_type = "Text", nullable)]
  pub name: Option<String>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::organizations::Entity",
    from = "Column::OrganizationId",
    to = "super::organizations::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Organizations,
  #[sea_orm(has_many = "super::organization_roles_users::Entity")]
  OrganizationRolesUsers,
  #[sea_orm(has_many = "super::permissions::Entity")]
  Permissions,
}

impl Related<super::organizations::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Organizations.def()
  }
}

impl Related<super::organization_roles_users::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OrganizationRolesUsers.def()
  }
}

impl Related<super::permissions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Permissions.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}