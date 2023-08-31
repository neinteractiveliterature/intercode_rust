use async_graphql::async_trait::async_trait;
use intercode_entities::event_categories;
use intercode_policies::{AuthorizationInfo, EntityPolicy, Policy, ReadManageAction};
use sea_orm::{sea_query::Expr, DbErr, EntityTrait, QueryFilter};

pub struct EventCategoryPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, event_categories::Model> for EventCategoryPolicy {
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
    resource: &event_categories::Model,
  ) -> Result<bool, Self::Error> {
    match action {
      ReadManageAction::Read => Ok(true),
      ReadManageAction::Manage => Ok(
        principal
          .has_scope_and_convention_permission(
            "manage_conventions",
            "update_event_categories",
            resource.convention_id,
          )
          .await?
          || principal.site_admin_manage(),
      ),
    }
  }
}

impl EntityPolicy<AuthorizationInfo, event_categories::Model> for EventCategoryPolicy {
  type Action = ReadManageAction;

  fn id_column() -> event_categories::Column {
    event_categories::Column::Id
  }

  fn accessible_to(
    _principal: &AuthorizationInfo,
    action: &Self::Action,
  ) -> sea_orm::Select<event_categories::Entity> {
    match action {
      ReadManageAction::Read => event_categories::Entity::find(),
      ReadManageAction::Manage => event_categories::Entity::find().filter(Expr::cust("0 = 1")),
    }
  }
}
