use async_graphql::Union;
use intercode_graphql_core::ModelBackedType;
use intercode_graphql_loaders::permissioned_roles_loader::PermissionedRole;

use crate::api::merged_objects::{OrganizationRoleType, StaffPositionType};

#[derive(Union)]
#[graphql(name = "PermissionedRole")]
pub enum PermissionedRoleType {
  OrganizationRole(OrganizationRoleType),
  StaffPosition(StaffPositionType),
}

impl From<PermissionedRole> for PermissionedRoleType {
  fn from(value: PermissionedRole) -> Self {
    match value {
      PermissionedRole::OrganizationRole(role) => {
        PermissionedRoleType::OrganizationRole(OrganizationRoleType::new(role))
      }
      PermissionedRole::StaffPosition(role) => {
        PermissionedRoleType::StaffPosition(StaffPositionType::new(role))
      }
    }
  }
}
