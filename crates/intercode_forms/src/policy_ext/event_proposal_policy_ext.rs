use async_trait::async_trait;
use intercode_entities::{
  conventions, event_proposals, model_ext::form_item_permissions::FormItemRole,
};
use intercode_policies::{
  policies::{is_non_draft_event_proposal, EventProposalPolicy},
  AuthorizationInfo,
};

use super::FormResponsePolicy;

#[async_trait]
impl FormResponsePolicy<AuthorizationInfo, (conventions::Model, event_proposals::Model)>
  for EventProposalPolicy
{
  async fn form_item_viewer_role(
    principal: &AuthorizationInfo,
    (convention, form_response): &(conventions::Model, event_proposals::Model),
  ) -> FormItemRole {
    if is_non_draft_event_proposal(form_response)
      && principal
        .has_convention_permission("update_event_proposals", convention.id)
        .await
        .unwrap_or(false)
      || principal.site_admin_manage()
    {
      return FormItemRole::Admin;
    }

    if let Some(owner_id) = form_response.owner_id {
      if principal
        .user_con_profile_ids()
        .await
        .cloned()
        .unwrap_or_default()
        .contains(&owner_id)
      {
        return FormItemRole::TeamMember;
      }
    }

    FormItemRole::Normal
  }

  async fn form_item_writer_role(
    principal: &AuthorizationInfo,
    resource: &(conventions::Model, event_proposals::Model),
  ) -> FormItemRole {
    EventProposalPolicy::form_item_viewer_role(principal, resource).await
  }
}
