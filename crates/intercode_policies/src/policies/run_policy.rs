use async_trait::async_trait;
use intercode_entities::{conventions, events, runs};
use sea_orm::DbErr;

use crate::{AuthorizationInfo, Policy, ReadManageAction};

use super::{EventAction, EventPolicy};

pub enum RunAction {
  Read,
  Manage,
  SignupSummary,
}

impl From<ReadManageAction> for RunAction {
  fn from(action: ReadManageAction) -> Self {
    match action {
      ReadManageAction::Read => Self::Read,
      ReadManageAction::Manage => Self::Manage,
    }
  }
}

pub struct RunPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, (conventions::Model, events::Model, runs::Model)> for RunPolicy {
  type Action = RunAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &Self::Action,
    (convention, event, run): &(conventions::Model, events::Model, runs::Model),
  ) -> Result<bool, Self::Error> {
    match action {
      RunAction::Read => todo!(),
      RunAction::Manage => Ok(
        principal
          .has_scope_and_convention_permission("manage_events", "update_runs", convention.id)
          .await?
          || principal.site_admin_manage(),
      ),
      RunAction::SignupSummary => {
        if event.private_signup_list {
          return Ok(false);
        }

        if EventPolicy::action_permitted(
          principal,
          &EventAction::ReadSignups,
          &(convention.clone(), event.clone()),
        )
        .await?
        {
          return Ok(true);
        }

        let active_signups = principal
          .active_signups_in_convention_by_event_id(event.convention_id)
          .await?;
        let run_signups = active_signups
          .get(&event.id)
          .map(|signups| {
            signups
              .iter()
              .filter(|signup| signup.run_id == run.id)
              .collect::<Vec<_>>()
          })
          .unwrap_or_default();

        Ok(principal.has_scope("read_signups") && !run_signups.is_empty())
      }
    }
  }
}
