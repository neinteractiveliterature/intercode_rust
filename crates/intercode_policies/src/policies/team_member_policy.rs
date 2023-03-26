use axum::async_trait;
use intercode_entities::{conventions, events, team_members};
use sea_orm::DbErr;

use crate::{
  authorization_info::AuthorizationInfo,
  policy::{Policy, ReadManageAction},
};

use super::{EventAction, EventPolicy};

pub struct TeamMemberPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, (conventions::Model, events::Model, team_members::Model)>
  for TeamMemberPolicy
{
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
    (convention, event, _team_member): &(conventions::Model, events::Model, team_members::Model),
  ) -> Result<bool, Self::Error> {
    if !principal.can_act_in_convention(event.convention_id) {
      return Ok(false);
    }

    match action {
      ReadManageAction::Read => Ok(
        (principal.has_scope("read_events")
          && EventPolicy::action_permitted(
            principal,
            &EventAction::Read,
            &(convention.clone(), event.clone()),
          )
          .await?)
          || (principal.has_scope("read_conventions")
            && (principal
              .has_convention_permission("update_event_team_members", event.convention_id)
              .await?
              || EventPolicy::action_permitted(
                principal,
                &EventAction::Read,
                &(convention.clone(), event.clone()),
              )
              .await?))
          || principal.site_admin_read(),
      ),
      ReadManageAction::Manage => Ok(
        (principal.has_scope("read_events")
          && EventPolicy::action_permitted(
            principal,
            &EventAction::Read,
            &(convention.clone(), event.clone()),
          )
          .await?)
          || (principal.has_scope("read_conventions")
            && (principal
              .has_convention_permission("update_event_team_members", event.convention_id)
              .await?))
          || principal.site_admin_manage(),
      ),
    }
  }
}
