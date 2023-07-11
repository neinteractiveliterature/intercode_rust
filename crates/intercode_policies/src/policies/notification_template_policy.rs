use axum::async_trait;
use intercode_entities::{conventions, notification_templates};
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter, QuerySelect};

use crate::{
  authorization_info::AuthorizationInfo,
  policy::{EntityPolicy, Policy, ReadManageAction},
};

pub struct NotificationTemplatePolicy;

#[async_trait]
impl Policy<AuthorizationInfo, notification_templates::Model> for NotificationTemplatePolicy {
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
    resource: &notification_templates::Model,
  ) -> Result<bool, Self::Error> {
    let convention_id = resource.convention_id;
    match action {
      ReadManageAction::Read => Ok(
        principal.has_scope("read_conventions")
          && (principal
            .has_convention_permission("read_notification_templates", convention_id)
            .await?
            || principal.site_admin_read()),
      ),
      ReadManageAction::Manage => Ok(
        principal.has_scope("manage_conventions")
          && (principal
            .has_convention_permission("update_notification_templates", convention_id)
            .await?
            || principal.site_admin_manage()),
      ),
    }
  }
}

impl EntityPolicy<AuthorizationInfo, notification_templates::Model> for NotificationTemplatePolicy {
  type Action = ReadManageAction;

  fn id_column() -> notification_templates::Column {
    notification_templates::Column::Id
  }

  fn accessible_to(
    principal: &AuthorizationInfo,
    action: &Self::Action,
  ) -> sea_orm::Select<notification_templates::Entity> {
    match action {
      ReadManageAction::Read => notification_templates::Entity::find().filter(
        notification_templates::Column::ConventionId.in_subquery(
          QuerySelect::query(
            &mut principal
              .conventions_with_permission("read_notification_templates")
              .select_only()
              .column(conventions::Column::Id),
          )
          .take(),
        ),
      ),
      ReadManageAction::Manage => notification_templates::Entity::find().filter(
        notification_templates::Column::ConventionId.in_subquery(
          QuerySelect::query(
            &mut principal
              .conventions_with_permission("manage_notification_templates")
              .select_only()
              .column(conventions::Column::Id),
          )
          .take(),
        ),
      ),
    }
  }
}
