use async_trait::async_trait;
use intercode_entities::{events, runs, signups};
use sea_orm::DbErr;

use crate::{AuthorizationInfo, Policy, ReadManageAction};

use super::{EventAction, EventPolicy};

pub enum SignupAction {
  Read,
  ReadRequestedBucketKey,
  Manage,
  Create,
  Withdraw,
  ForceConfirm,
  UpdateCounted,
  UpdateBucket,
}

impl From<ReadManageAction> for SignupAction {
  fn from(action: ReadManageAction) -> Self {
    match action {
      ReadManageAction::Read => Self::Read,
      ReadManageAction::Manage => Self::Manage,
    }
  }
}

pub struct SignupPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, (events::Model, runs::Model, signups::Model)> for SignupPolicy {
  type Action = SignupAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &Self::Action,
    (event, run, signup): &(events::Model, runs::Model, signups::Model),
  ) -> Result<bool, Self::Error> {
    match action {
      SignupAction::Read => todo!(),
      SignupAction::ReadRequestedBucketKey => todo!(),
      SignupAction::Manage => todo!(),
      SignupAction::Create => todo!(),
      SignupAction::Withdraw => todo!(),
      SignupAction::ForceConfirm | SignupAction::UpdateCounted | SignupAction::UpdateBucket => Ok(
        (principal.has_scope("manage_events")
          && principal
            .team_member_event_ids_in_convention(event.convention_id)
            .await?
            .contains(&event.id))
          || (principal
            .has_scope_and_convention_permission(
              "manage_conventions",
              "update_signups",
              event.convention_id,
            )
            .await?)
          || principal.site_admin_manage(),
      ),
    }
  }
}
