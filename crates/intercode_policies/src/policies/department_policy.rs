use axum::async_trait;
use intercode_entities::departments;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter, QuerySelect};

use crate::{
  authorization_info::AuthorizationInfo,
  policy::{EntityPolicy, Policy, ReadManageAction},
};

pub struct DepartmentPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, departments::Model> for DepartmentPolicy {
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
    user_activity_alert: &departments::Model,
  ) -> Result<bool, Self::Error> {
    match action {
      ReadManageAction::Read => Ok(
        principal
          .has_scope_and_convention_permission(
            "read_conventions",
            "read_departments",
            user_activity_alert.convention_id,
          )
          .await?
          || principal.site_admin_read(),
      ),
      ReadManageAction::Manage => Ok(
        principal
          .has_scope_and_convention_permission(
            "manage_conventions",
            "update_departments",
            user_activity_alert.convention_id,
          )
          .await?
          || principal.site_admin_manage(),
      ),
    }
  }
}

impl EntityPolicy<AuthorizationInfo, departments::Model> for DepartmentPolicy {
  type Action = ReadManageAction;

  fn id_column() -> departments::Column {
    departments::Column::Id
  }

  fn accessible_to(
    principal: &AuthorizationInfo,
    action: &Self::Action,
  ) -> sea_orm::Select<departments::Entity> {
    match action {
      ReadManageAction::Read => {
        departments::Entity::find().filter(departments::Column::ConventionId.in_subquery(
          QuerySelect::query(&mut principal.conventions_with_permission("read_departments")).take(),
        ))
      }
      ReadManageAction::Manage => departments::Entity::find().filter(
        departments::Column::ConventionId.in_subquery(
          QuerySelect::query(&mut principal.conventions_with_permission("update_departments"))
            .take(),
        ),
      ),
    }
  }
}
