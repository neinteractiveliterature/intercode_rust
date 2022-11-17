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

#[cfg(test)]
mod tests {
  use intercode_entities::conventions;
  use sea_orm::{ActiveModelTrait, ActiveValue};

  use super::*;
  use crate::test_helpers::with_test_db;

  fn mock_room() -> rooms::Model {
    rooms::Model {
      convention_id: None,
      created_at: Default::default(),
      updated_at: Default::default(),
      id: 0,
      name: Some("A room".to_string()),
    }
  }

  #[tokio::test]
  async fn test_lets_anyone_read_any_room() {
    with_test_db(|db| {
      Box::pin(async move {
        assert!(RoomPolicy::action_permitted(
          &AuthorizationInfo::for_test(db, None, None, None).await,
          ReadManageAction::Read,
          &mock_room()
        )
        .await
        .unwrap())
      })
    })
    .await
  }

  #[tokio::test]
  async fn test_lets_user_with_update_rooms_manage_rooms() {
    with_test_db(|db| {
      Box::pin(async move {
        let convention = conventions::ActiveModel {
          domain: ActiveValue::Set("intercode.test".to_string()),
          email_from: ActiveValue::Set("noreply@intercode.test".to_string()),
          language: ActiveValue::Set("en".to_string()),
          timezone_mode: ActiveValue::Set("user_local".to_string()),
          ticket_mode: ActiveValue::Set("disabled".to_string()),
          ..Default::default()
        };
        convention.insert(db.as_ref()).await.unwrap();
      })
    })
    .await
  }
}