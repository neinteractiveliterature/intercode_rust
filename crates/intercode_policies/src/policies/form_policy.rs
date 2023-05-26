use axum::async_trait;
use intercode_entities::forms;
use sea_orm::{sea_query::Expr, DbErr, EntityTrait, QueryFilter};

use crate::{
  authorization_info::AuthorizationInfo,
  policy::{EntityPolicy, Policy, ReadManageAction},
};

pub struct FormPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, forms::Model> for FormPolicy {
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
    resource: &forms::Model,
  ) -> Result<bool, Self::Error> {
    match action {
      ReadManageAction::Read => Ok(true),
      ReadManageAction::Manage => {
        if principal.has_scope("manage_conventions") {
          let convention_id = resource.convention_id;
          let has_permission = if let Some(convention_id) = convention_id {
            principal
              .has_convention_permission("update_forms", convention_id)
              .await?
          } else {
            false
          };
          Ok(has_permission || principal.site_admin_manage())
        } else {
          Ok(false)
        }
      }
    }
  }
}

impl EntityPolicy<AuthorizationInfo, forms::Model> for FormPolicy {
  type Action = ReadManageAction;

  fn id_column() -> forms::Column {
    forms::Column::Id
  }

  fn accessible_to(
    _principal: &AuthorizationInfo,
    action: &Self::Action,
  ) -> sea_orm::Select<forms::Entity> {
    match action {
      ReadManageAction::Read => forms::Entity::find(),
      ReadManageAction::Manage => forms::Entity::find().filter(Expr::cust("0 = 1")),
    }
  }
}
