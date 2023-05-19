use axum::async_trait;
use intercode_entities::{organization_roles, organizations};
use sea_orm::DbErr;

use crate::{
  authorization_info::AuthorizationInfo,
  policy::{Policy, ReadManageAction},
};

pub struct OrganizationRolePolicy;

#[async_trait]
impl Policy<AuthorizationInfo, (organizations::Model, organization_roles::Model)>
  for OrganizationRolePolicy
{
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
    (organization, _organization_role): &(organizations::Model, organization_roles::Model),
  ) -> Result<bool, Self::Error> {
    let perms = principal
      .organization_permissions_by_organization_id()
      .await?
      .get(&organization.id)
      .cloned()
      .unwrap_or_default();

    match action {
      ReadManageAction::Read => Ok(
        principal.has_scope("read_organizations")
          && (principal.site_admin_read() || perms.contains("manage_organization_access")),
      ),
      ReadManageAction::Manage => Ok(
        principal.has_scope("manage_organizations")
          && (principal.site_admin_manage() || perms.contains("manage_organization_access")),
      ),
    }
  }
}
