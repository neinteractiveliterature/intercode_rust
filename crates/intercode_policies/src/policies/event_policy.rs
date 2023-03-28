use async_trait::async_trait;
use intercode_entities::{conventions, events, model_ext::form_item_permissions::FormItemRole};
use sea_orm::DbErr;

use crate::{AuthorizationInfo, FormResponsePolicy, Policy, ReadManageAction};

use super::has_schedule_release_permissions;

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
  ProvideTickets,
  OverrideMaximumEventProvidedTickets,
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
impl Policy<AuthorizationInfo, (conventions::Model, events::Model)> for EventPolicy {
  type Action = EventAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &Self::Action,
    (convention, event): &(conventions::Model, events::Model),
  ) -> Result<bool, Self::Error> {
    match action {
      EventAction::Read => Ok(
        (principal.has_scope("read_events")
          && (convention.site_mode == "single_event"
            || principal
              .team_member_event_ids_in_convention(event.convention_id)
              .await?
              .contains(&event.id)
            || (event.status == "active"
              && has_schedule_release_permissions(
                principal,
                convention,
                &convention.show_event_list,
              )
              .await)
            || principal
              .has_convention_permission("read_inactive_events", event.convention_id)
              .await?
            || principal
              .has_convention_permission("update_events", event.convention_id)
              .await?))
          || principal.site_admin_read(),
      ),
      EventAction::ReadSignups => {
        Self::action_permitted(
          principal,
          &EventAction::ReadSignupDetails,
          &(convention.clone(), event.clone()),
        )
        .await
      }
      EventAction::ReadSignupDetails => Ok(
        principal
          .has_scope_and_convention_permission(
            "read_conventions",
            "read_signup_details",
            event.convention_id,
          )
          .await?
          || (principal.has_scope("read_events")
            && principal
              .team_member_event_ids_in_convention(event.convention_id)
              .await?
              .contains(&event.convention_id))
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
            .team_member_event_ids_in_convention(event.convention_id)
            .await?
            .contains(&event.id)
            || principal
              .has_event_category_permission(
                "update_events",
                event.convention_id,
                event.event_category_id,
              )
              .await?
            || principal
              .has_convention_permission("update_events", event.convention_id)
              .await?
            || principal.site_admin_manage()),
      ),
      EventAction::ProvideTickets => todo!(),
      EventAction::OverrideMaximumEventProvidedTickets => Ok(
        principal.has_scope("manage_events")
          && (principal
            .has_convention_permission("override_event_tickets", convention.id)
            .await?
            || principal
              .has_event_category_permission(
                "override_event_tickets",
                convention.id,
                event.event_category_id,
              )
              .await?)
          || principal.site_admin_manage(),
      ),
    }
  }
}

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
