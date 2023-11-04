use axum::async_trait;
use intercode_entities::oauth_applications;
use intercode_policies::{AuthorizationInfo, Policy, ReadManageAction};
use sea_orm::DbErr;

pub struct OAuthApplicationPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, oauth_applications::Model> for OAuthApplicationPolicy {
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
    _resource: &oauth_applications::Model,
  ) -> Result<bool, Self::Error> {
    // Only accessible by site admins, and only with a real cookie session (so no oauth_scope)
    if principal.oauth_scope.is_some() {
      return Ok(false);
    }

    match action {
      ReadManageAction::Read => Ok(principal.site_admin_read()),
      ReadManageAction::Manage => Ok(principal.site_admin_manage()),
    }
  }
}
