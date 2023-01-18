use async_trait::async_trait;
use intercode_entities::{events, model_ext::form_item_permissions::FormItemRole};
use sea_orm::DbErr;

use crate::{AuthorizationInfo, FormResponsePolicy, Policy, ReadManageAction};

pub enum EventAction {
  Read,
  ReadAdminNotes,
  ReadSignups,
  ReadSignupDetails,
  UpdateAdminNotes,
  Drop,
  Create,
  Restore,
  Update,
}

impl From<ReadManageAction> for EventAction {
  fn from(action: ReadManageAction) -> Self {
    match action {
      ReadManageAction::Read => Self::Read,
      ReadManageAction::Manage => Self::Update,
    }
  }
}

pub struct EventPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, events::Model> for EventPolicy {
  type Action = EventAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &Self::Action,
    resource: &events::Model,
  ) -> Result<bool, Self::Error> {
    match action {
      EventAction::Read => todo!(),
      EventAction::ReadSignups => {
        Self::action_permitted(principal, &EventAction::ReadSignupDetails, resource).await
      }
      EventAction::ReadSignupDetails => Ok(
        principal
          .has_scope_and_convention_permission(
            "read_conventions",
            "read_signup_details",
            resource.convention_id,
          )
          .await?
          || (principal.has_scope("read_events")
            && principal
              .team_member_event_ids_in_convention(resource.convention_id)
              .await?
              .contains(&resource.convention_id))
          || principal.site_admin_read(),
      ),
      EventAction::ReadAdminNotes => todo!(),
      EventAction::UpdateAdminNotes => todo!(),
      EventAction::Drop => todo!(),
      EventAction::Create => todo!(),
      EventAction::Restore => todo!(),
      EventAction::Update => Ok(
        principal.has_scope("manage_events")
          && (principal
            .team_member_event_ids_in_convention(resource.convention_id)
            .await?
            .contains(&resource.id)
            || principal
              .has_event_category_permission(
                "update_events",
                resource.convention_id,
                resource.event_category_id,
              )
              .await?
            || principal
              .has_convention_permission("update_events", resource.convention_id)
              .await?
            || principal.site_admin_manage()),
      ),
    }
  }
}

#[async_trait]
impl FormResponsePolicy<AuthorizationInfo, events::Model> for EventPolicy {
  async fn form_item_viewer_role(
    principal: &AuthorizationInfo,
    form_response: &events::Model,
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
    form_response: &events::Model,
  ) -> FormItemRole {
    Self::form_item_viewer_role(principal, form_response).await
  }
}
