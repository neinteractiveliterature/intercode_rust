use async_trait::async_trait;
use intercode_entities::{events, runs, signups};
use sea_orm::DbErr;

use crate::{AuthorizationInfo, Policy, ReadManageAction};

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
      SignupAction::Read => {
        if !principal.can_act_in_convention(event.convention_id) {
          return Ok(false);
        }

        Ok(
          (principal.has_scope("read_events")
            && (!event.private_signup_list
              && principal
                .active_signups_in_convention_by_event_id(event.convention_id)
                .await?
                .get(&event.id)
                .cloned()
                .unwrap_or_default()
                .iter()
                .any(|signup| signup.run_id == run.id)))
            || SignupPolicy::action_permitted(
              principal,
              &SignupAction::ReadRequestedBucketKey,
              &(event.clone(), run.clone(), signup.clone()),
            )
            .await?,
        )
      }
      SignupAction::ReadRequestedBucketKey => {
        if !principal.can_act_in_convention(event.convention_id) {
          return Ok(false);
        }

        Ok(
          (principal.has_scope("read_signups")
            && (principal
              .user_con_profile_ids()
              .await?
              .contains(&signup.user_con_profile_id)))
            || (principal.has_scope("read_conventions")
              && principal
                .has_convention_permission("read_signup_details", event.convention_id)
                .await?)
            || (principal.has_scope("read_events")
              && principal
                .team_member_event_ids_in_convention(event.convention_id)
                .await?
                .contains(&event.id))
            || principal.site_admin_read(),
        )
      }
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
