use async_graphql::Union;

use crate::api::{merged_objects::StaffPositionType, objects::OrganizationRoleType};

#[derive(Union)]
#[graphql(name = "PermissionedRole")]
pub enum PermissionedRoleType {
  OrganizationRole(OrganizationRoleType),
  StaffPosition(StaffPositionType),
}
