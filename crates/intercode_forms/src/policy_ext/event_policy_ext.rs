use async_trait::async_trait;
use intercode_entities::{conventions, events, model_ext::form_item_permissions::FormItemRole};
use intercode_policies::{policies::EventPolicy, AuthorizationInfo};

use super::FormResponsePolicy;

#[async_trait]
impl FormResponsePolicy<AuthorizationInfo, (conventions::Model, events::Model)> for EventPolicy {
  async fn form_item_viewer_role(
    principal: &AuthorizationInfo,
    (_convention, form_response): &(conventions::Model, events::Model),
  ) -> FormItemRole {
    if principal
      .has_convention_permission("update_events", form_response.convention_id)
      .await
      .unwrap_or(false)
      || principal.site_admin_manage()
    {
      return FormItemRole::Admin;
    }

    if principal
      .team_member_event_ids_in_convention(form_response.convention_id)
      .await
      .unwrap_or_default()
      .contains(&form_response.id)
    {
      return FormItemRole::TeamMember;
    }

    if principal
      .active_signups_in_convention_by_event_id(form_response.convention_id)
      .await
      .unwrap_or_default()
      .get(&form_response.id)
      .map(|signups| signups.iter().any(|signup| signup.state == "confirmed"))
      .unwrap_or(false)
    {
      return FormItemRole::ConfirmedAttendee;
    }

    FormItemRole::Normal
  }

  async fn form_item_writer_role(
    principal: &AuthorizationInfo,
    resource: &(conventions::Model, events::Model),
  ) -> FormItemRole {
    Self::form_item_viewer_role(principal, resource).await
  }
}
