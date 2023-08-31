use async_graphql::async_trait::async_trait;
use intercode_entities::email_routes;
use sea_orm::{sea_query::Expr, DbErr, EntityTrait, QueryFilter};

use intercode_policies::{
  AuthorizationInfo, EntityPolicy, Policy, ReadManageAction, SimpleGuardablePolicy,
};

pub struct EmailRoutePolicy;

#[async_trait]
impl Policy<AuthorizationInfo, email_routes::Model> for EmailRoutePolicy {
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
    _email_route: &email_routes::Model,
  ) -> Result<bool, Self::Error> {
    match action {
      ReadManageAction::Read => {
        Ok(principal.has_scope("read_email_routing") && principal.site_admin_read())
      }
      ReadManageAction::Manage => {
        Ok(principal.has_scope("manage_email_routing") && principal.site_admin_manage())
      }
    }
  }
}

impl EntityPolicy<AuthorizationInfo, email_routes::Model> for EmailRoutePolicy {
  type Action = ReadManageAction;

  fn id_column() -> email_routes::Column {
    email_routes::Column::Id
  }

  fn accessible_to(
    principal: &AuthorizationInfo,
    action: &Self::Action,
  ) -> sea_orm::Select<<email_routes::Model as sea_orm::ModelTrait>::Entity> {
    let scope = email_routes::Entity::find();

    match action {
      ReadManageAction::Read => {
        if principal.has_scope("read_email_routing") && principal.site_admin_read() {
          return scope;
        }
      }
      ReadManageAction::Manage => {
        if principal.has_scope("manage_email_routing") && principal.site_admin_manage() {
          return scope;
        }
      }
    }

    scope.filter(Expr::cust("1 = 0"))
  }
}

impl SimpleGuardablePolicy<'_, email_routes::Model> for EmailRoutePolicy {}
