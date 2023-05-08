use async_trait::async_trait;
use intercode_entities::{conventions, events, runs, signups};
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
impl
  Policy<
    AuthorizationInfo,
    (
      conventions::Model,
      events::Model,
      runs::Model,
      signups::Model,
    ),
  > for SignupPolicy
{
  type Action = SignupAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &Self::Action,
    (convention, event, run, signup): &(
      conventions::Model,
      events::Model,
      runs::Model,
      signups::Model,
    ),
  ) -> Result<bool, Self::Error> {
    if !principal.can_act_in_convention(convention.id) {
      return Ok(false);
    }

    match action {
      SignupAction::Read => Ok(
        (principal.has_scope("read_events")
          && (!event.private_signup_list
            && principal
              .active_signups_in_convention_by_event_id(convention.id)
              .await?
              .get(&event.id)
              .cloned()
              .unwrap_or_default()
              .iter()
              .any(|signup| signup.run_id == run.id)))
          || SignupPolicy::action_permitted(
            principal,
            &SignupAction::ReadRequestedBucketKey,
            &(
              convention.clone(),
              event.clone(),
              run.clone(),
              signup.clone(),
            ),
          )
          .await?,
      ),
      SignupAction::ReadRequestedBucketKey => {
        if !principal.can_act_in_convention(convention.id) {
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
                .has_convention_permission("read_signup_details", convention.id)
                .await?)
            || (principal.has_scope("read_events")
              && principal
                .team_member_event_ids_in_convention(convention.id)
                .await?
                .contains(&event.id))
            || principal.site_admin_read(),
        )
      }
      SignupAction::Manage => Ok(
        (convention.signup_mode == "moderated"
          && principal
            .has_scope_and_convention_permission(
              "manage_conventions",
              "update_signups",
              convention.id,
            )
            .await?)
          || principal.site_admin_manage(),
      ),
      SignupAction::Create => todo!(),
      SignupAction::Withdraw => todo!(),
      SignupAction::ForceConfirm | SignupAction::UpdateCounted | SignupAction::UpdateBucket => Ok(
        (principal.has_scope("manage_events")
          && principal
            .team_member_event_ids_in_convention(convention.id)
            .await?
            .contains(&event.id))
          || (principal
            .has_scope_and_convention_permission(
              "manage_conventions",
              "update_signups",
              convention.id,
            )
            .await?)
          || principal.site_admin_manage(),
      ),
    }
  }
}
