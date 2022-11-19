use async_trait::async_trait;
use intercode_entities::conventions;
use sea_orm::DbErr;

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
      ConventionAction::Schedule => todo!(),
      ConventionAction::ListEvents => todo!(),
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
      ConventionAction::ViewEventProposals => todo!(),
    }
  }
}
