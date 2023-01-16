use async_trait::async_trait;
use futures::try_join;
use intercode_entities::conventions;
use sea_orm::DbErr;
use tuple_conv::RepeatedTuple;

use crate::{AuthorizationInfo, Policy, ReadManageAction};

pub enum ConventionAction {
  Read,
  Update,
  Schedule,
  ListEvents,
  ScheduleWithCounts,
  ViewReports,
  ViewAttendees,
  ViewEventProposals,
}

impl From<ReadManageAction> for ConventionAction {
  fn from(action: ReadManageAction) -> Self {
    match action {
      ReadManageAction::Read => Self::Read,
      ReadManageAction::Manage => Self::Update,
    }
  }
}

async fn has_schedule_release_permissions(
  authorization_info: &AuthorizationInfo,
  convention: &conventions::Model,
  schedule_release_value: &str,
) -> bool {
  match schedule_release_value {
    "yes" => true,
    "gms" => {
      let result = try_join!(
        authorization_info.has_convention_permission("read_prerelease_schedule", convention.id),
        authorization_info
          .has_convention_permission("read_limited_prerelease_schedule", convention.id),
        authorization_info.has_convention_permission("update_events", convention.id),
        authorization_info.team_member_in_convention(convention.id),
      )
      .map(|results| results.to_boxed_slice().iter().any(|result| *result));
      matches!(result, Ok(true))
    }
    "priv" => {
      let result = try_join!(
        authorization_info
          .has_convention_permission("read_limited_prerelease_schedule", convention.id),
        authorization_info.has_convention_permission("update_events", convention.id),
        authorization_info.team_member_in_convention(convention.id),
      )
      .map(|results| results.to_boxed_slice().iter().any(|result| *result));
      matches!(result, Ok(true))
    }
    _ => {
      matches!(
        authorization_info
          .has_convention_permission("update_events", convention.id)
          .await,
        Ok(true)
      )
    }
  }
}

pub struct ConventionPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, conventions::Model> for ConventionPolicy {
  type Action = ConventionAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &Self::Action,
    resource: &conventions::Model,
  ) -> Result<bool, Self::Error> {
    match action {
      ConventionAction::Read => Ok(true),
      ConventionAction::Update => Ok(
        principal
          .has_scope_and_convention_permission(
            "manage_conventions",
            "update_convention",
            resource.id,
          )
          .await?
          || principal.site_admin_manage(),
      ),
      ConventionAction::Schedule => Ok(
        resource.show_schedule == "yes"
          || (principal.has_scope("read_conventions")
            && has_schedule_release_permissions(principal, resource, &resource.show_schedule)
              .await),
      ),
      ConventionAction::ListEvents => Ok(
        resource.show_schedule == "yes"
          || (principal.has_scope("read_conventions")
            && has_schedule_release_permissions(principal, resource, &resource.show_event_list)
              .await),
      ),
      ConventionAction::ScheduleWithCounts => Ok(
        principal
          .has_scope_and_convention_permission(
            "read_conventions",
            "read_schedule_with_counts",
            resource.id,
          )
          .await?
          || principal.site_admin_read(),
      ),
      ConventionAction::ViewReports => Ok(
        principal
          .has_scope_and_convention_permission("read_conventions", "read_reports", resource.id)
          .await?
          || principal.site_admin_read(),
      ),
      ConventionAction::ViewAttendees => Ok(
        principal
          .has_scope_and_convention_permission(
            "read_conventions",
            "read_user_con_profiles",
            resource.id,
          )
          .await?
          || principal.site_admin_read(),
      ),
      ConventionAction::ViewEventProposals => {
        if !principal.can_act_in_convention(resource.id) || !principal.has_scope("read_events") {
          return Ok(false);
        }

        // this is a weird one: does the user have _any_ permission called read_event_proposal
        // in this convention?
        Ok(
          principal
            .all_model_permissions_in_convention(resource.id)
            .await?
            .has_any_permission("read_event_proposals"),
        )
      }
    }
  }
}
