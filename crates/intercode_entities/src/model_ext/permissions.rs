use std::fmt::Display;

use crate::permissions;

pub struct ExclusiveArcMissingReference;

impl Display for ExclusiveArcMissingReference {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!(
      "Exclusive arc has no non-null value on any of its reference fields"
    ))
  }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum PermissionedModelRef {
  CmsContentGroup(i64),
  Convention(i64),
  EventCategory(i64),
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum PermissionedRoleRef {
  OrganizationRole(i64),
  StaffPosition(i64),
}

impl TryFrom<&permissions::Model> for PermissionedModelRef {
  type Error = ExclusiveArcMissingReference;

  fn try_from(value: &permissions::Model) -> Result<Self, Self::Error> {
    if let Some(cms_content_group_id) = value.cms_content_group_id {
      Ok(PermissionedModelRef::CmsContentGroup(cms_content_group_id))
    } else if let Some(convention_id) = value.convention_id {
      Ok(PermissionedModelRef::Convention(convention_id))
    } else if let Some(event_category_id) = value.event_category_id {
      Ok(PermissionedModelRef::EventCategory(event_category_id))
    } else {
      Err(ExclusiveArcMissingReference)
    }
  }
}

impl TryFrom<&permissions::Model> for PermissionedRoleRef {
  type Error = ExclusiveArcMissingReference;

  fn try_from(value: &permissions::Model) -> Result<Self, Self::Error> {
    if let Some(organization_role_id) = value.organization_role_id {
      Ok(PermissionedRoleRef::OrganizationRole(organization_role_id))
    } else if let Some(staff_position_id) = value.staff_position_id {
      Ok(PermissionedRoleRef::StaffPosition(staff_position_id))
    } else {
      Err(ExclusiveArcMissingReference)
    }
  }
}
