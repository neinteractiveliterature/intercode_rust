use axum::async_trait;
use intercode_entities::{conventions, events, maximum_event_provided_tickets_overrides};
use sea_orm::DbErr;

use crate::{
  authorization_info::AuthorizationInfo,
  policy::{Policy, ReadManageAction},
};

use super::{EventAction, EventPolicy};

pub struct MaximumEventProvidedTicketsOverridePolicy;

#[async_trait]
impl
  Policy<
    AuthorizationInfo,
    (
      conventions::Model,
      events::Model,
      maximum_event_provided_tickets_overrides::Model,
    ),
  > for MaximumEventProvidedTicketsOverridePolicy
{
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
    (convention, event, _mepto): &(
      conventions::Model,
      events::Model,
      maximum_event_provided_tickets_overrides::Model,
    ),
  ) -> Result<bool, Self::Error> {
    match action {
      ReadManageAction::Read => Ok(
        principal.has_scope("read_events")
          && (principal
            .has_convention_permission("override_event_tickets", convention.id)
            .await?
            || principal
              .has_event_category_permission(
                "override_event_tickets",
                convention.id,
                event.event_category_id,
              )
              .await?
            || principal
              .team_member_event_ids_in_convention(convention.id)
              .await?
              .contains(&event.id))
          || principal.site_admin_read(),
      ),
      ReadManageAction::Manage => {
        EventPolicy::action_permitted(
          principal,
          &EventAction::OverrideMaximumEventProvidedTickets,
          &(convention.clone(), event.clone()),
        )
        .await
      }
    }
  }
}
