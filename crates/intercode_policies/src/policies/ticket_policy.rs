use async_trait::async_trait;
use intercode_entities::{conventions, tickets, user_con_profiles};
use sea_orm::DbErr;

use crate::{AuthorizationInfo, Policy, ReadManageAction};

pub enum TicketAction {
  Read,
  Manage,
  Provide,
}

impl From<ReadManageAction> for TicketAction {
  fn from(action: ReadManageAction) -> Self {
    match action {
      ReadManageAction::Read => Self::Read,
      ReadManageAction::Manage => Self::Manage,
    }
  }
}

pub struct TicketPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, (conventions::Model, user_con_profiles::Model, tickets::Model)>
  for TicketPolicy
{
  type Action = TicketAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &TicketAction,
    (convention, user_con_profile, _ticket): &(
      conventions::Model,
      user_con_profiles::Model,
      tickets::Model,
    ),
  ) -> Result<bool, Self::Error> {
    if !principal.can_act_in_convention(convention.id) {
      return Ok(false);
    }

    match action {
      TicketAction::Read => Ok(
        (principal
          .has_scope_and_convention_permission("read_conventions", "read_tickets", convention.id)
          .await?)
          || (principal.has_scope("read_events")
            && principal.team_member_in_convention(convention.id).await?)
          || {
            principal.has_scope("read_profile")
              && principal
                .user
                .as_ref()
                .map(|u| u.id == user_con_profile.id)
                .unwrap_or(false)
          }
          || principal.site_admin_read(),
      ),
      TicketAction::Manage => {
        principal
          .has_scope_and_convention_permission(
            "manage_conventions",
            "update_tickets",
            convention.id,
          )
          .await
      }
      TicketAction::Provide => todo!(),
    }
  }
}
