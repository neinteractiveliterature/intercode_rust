use async_graphql::Union;

use crate::api::objects::{OrganizationRoleType, StaffPositionType};

#[derive(Union)]
#[graphql(name = "PermissionedRole")]
pub enum PermissionedRoleType {
  OrganizationRole(OrganizationRoleType),
  StaffPosition(StaffPositionType),
}
