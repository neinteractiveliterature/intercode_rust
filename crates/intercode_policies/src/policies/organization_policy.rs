use axum::async_trait;
use intercode_entities::organizations;
use sea_orm::DbErr;

use crate::{
  authorization_info::AuthorizationInfo,
  policy::{Policy, ReadManageAction},
  SimpleGuardablePolicy,
};

async fn can_manage_any_organizations(principal: &AuthorizationInfo) -> Result<bool, DbErr> {
  Ok(
    principal
      .organization_permissions_by_organization_id()
      .await?
      .iter()
      .any(|(_organization_id, perms)| perms.contains("manage_organization_access")),
  )
}

pub struct OrganizationPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, organizations::Model> for OrganizationPolicy {
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
    _organization: &organizations::Model,
  ) -> Result<bool, Self::Error> {
    match action {
      ReadManageAction::Read => Ok(
        principal.has_scope("read_organizations")
          && (principal.site_admin_read() || can_manage_any_organizations(principal).await?),
      ),
      ReadManageAction::Manage => {
        Ok(principal.has_scope("manage_organizations") && principal.site_admin_manage())
      }
    }
  }
}

impl SimpleGuardablePolicy<'_, organizations::Model> for OrganizationPolicy {}
