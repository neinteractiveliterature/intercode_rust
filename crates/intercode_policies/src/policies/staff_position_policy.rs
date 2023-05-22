use axum::async_trait;
use intercode_entities::staff_positions;
use sea_orm::{DbErr, EntityTrait};

use crate::{
  authorization_info::AuthorizationInfo,
  policy::{EntityPolicy, Policy, ReadManageAction},
};

pub struct StaffPositionPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, staff_positions::Model> for StaffPositionPolicy {
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
    resource: &staff_positions::Model,
  ) -> Result<bool, Self::Error> {
    match action {
      ReadManageAction::Read => Ok(true),
      ReadManageAction::Manage => {
        if principal.has_scope("manage_conventions") {
          let convention_id = resource.convention_id;
          let has_permission = if let Some(convention_id) = convention_id {
            principal
              .has_convention_permission("update_staff_positions", convention_id)
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

impl EntityPolicy<AuthorizationInfo, staff_positions::Model> for StaffPositionPolicy {
  type Action = ReadManageAction;

  fn id_column() -> staff_positions::Column {
    staff_positions::Column::Id
  }

  fn accessible_to(
    _principal: &AuthorizationInfo,
    _action: &Self::Action,
  ) -> sea_orm::Select<<staff_positions::Model as sea_orm::ModelTrait>::Entity> {
    staff_positions::Entity::find()
  }
}
