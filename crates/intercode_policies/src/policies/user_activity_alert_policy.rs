use axum::async_trait;
use intercode_entities::user_activity_alerts;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter, QuerySelect};

use crate::{
  authorization_info::AuthorizationInfo,
  policy::{EntityPolicy, Policy, ReadManageAction},
};

pub struct UserActivityAlertPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, user_activity_alerts::Model> for UserActivityAlertPolicy {
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
    user_activity_alert: &user_activity_alerts::Model,
  ) -> Result<bool, Self::Error> {
    match action {
      ReadManageAction::Read => Ok(
        principal
          .has_scope_and_convention_permission(
            "read_conventions",
            "update_user_activity_alerts",
            user_activity_alert.convention_id,
          )
          .await?
          || principal.site_admin_read(),
      ),
      ReadManageAction::Manage => Ok(
        principal
          .has_scope_and_convention_permission(
            "manage_conventions",
            "update_user_activity_alerts",
            user_activity_alert.convention_id,
          )
          .await?
          || principal.site_admin_manage(),
      ),
    }
  }
}

impl EntityPolicy<AuthorizationInfo, user_activity_alerts::Model> for UserActivityAlertPolicy {
  type Action = ReadManageAction;

  fn id_column() -> user_activity_alerts::Column {
    user_activity_alerts::Column::Id
  }

  fn accessible_to(
    principal: &AuthorizationInfo,
    _action: &Self::Action,
  ) -> sea_orm::Select<user_activity_alerts::Entity> {
    user_activity_alerts::Entity::find().filter(
      user_activity_alerts::Column::ConventionId.in_subquery(
        QuerySelect::query(
          &mut principal.conventions_with_permission("update_user_activity_alerts"),
        )
        .take(),
      ),
    )
  }
}
