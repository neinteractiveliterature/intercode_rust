use axum::async_trait;
use intercode_entities::rooms;
use sea_orm::{sea_query::Expr, DbErr, EntityTrait, QueryFilter};

use crate::{
  authorization_info::AuthorizationInfo,
  policy::{EntityPolicy, Policy, ReadManageAction},
  SimpleGuardablePolicy,
};

pub struct RoomPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, rooms::Model> for RoomPolicy {
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
    resource: &rooms::Model,
  ) -> Result<bool, Self::Error> {
    match action {
      ReadManageAction::Read => Ok(true),
      ReadManageAction::Manage => {
        if principal.has_scope("manage_conventions") {
          let convention_id = resource.convention_id;
          let has_permission = if let Some(convention_id) = convention_id {
            principal
              .has_convention_permission("update_rooms", convention_id)
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

impl EntityPolicy<AuthorizationInfo, rooms::Model> for RoomPolicy {
  type Action = ReadManageAction;

  fn id_column() -> rooms::Column {
    rooms::Column::Id
  }

  fn accessible_to(
    _principal: &AuthorizationInfo,
    action: &Self::Action,
  ) -> sea_orm::Select<rooms::Entity> {
    match action {
      ReadManageAction::Read => rooms::Entity::find(),
      ReadManageAction::Manage => rooms::Entity::find().filter(Expr::cust("0 = 1")),
    }
  }
}

impl SimpleGuardablePolicy<'_, rooms::Model> for RoomPolicy {}

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
          &ReadManageAction::Read,
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
