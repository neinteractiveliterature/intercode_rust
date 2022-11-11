use axum::async_trait;
use intercode_entities::rooms;
use sea_orm::{DbErr, EntityTrait};

use crate::{
  authorization_info::AuthorizationInfo,
  policy::{EntityPolicy, Policy, ReadManageAction},
};

pub struct RoomPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, ReadManageAction, rooms::Model> for RoomPolicy {
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: ReadManageAction,
    resource: &rooms::Model,
  ) -> Result<bool, Self::Error> {
    match action {
      ReadManageAction::Read => Ok(true),
      ReadManageAction::Manage => {
        if principal.has_scope("manage_conventions") {
          let convention_id = resource.convention_id;
          let has_permission = if let Some(convention_id) = convention_id {
            let perms = principal
              .all_model_permissions_in_convention(convention_id)
              .await?;

            perms.has_convention_permission(convention_id, "update_rooms")
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

impl EntityPolicy<AuthorizationInfo, ReadManageAction, rooms::Model> for RoomPolicy {
  fn accessible_to(
    _principal: &AuthorizationInfo,
  ) -> sea_orm::Select<<rooms::Model as sea_orm::ModelTrait>::Entity> {
    rooms::Entity::find()
  }
}
