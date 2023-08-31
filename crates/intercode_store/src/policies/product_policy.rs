use async_trait::async_trait;
use intercode_entities::products;
use intercode_policies::{AuthorizationInfo, EntityPolicy, Policy, ReadManageAction};
use sea_orm::{sea_query::Expr, DbErr, EntityTrait, QueryFilter};

pub struct ProductPolicy;

#[async_trait]
impl Policy<AuthorizationInfo, products::Model> for ProductPolicy {
  type Action = ReadManageAction;
  type Error = DbErr;

  async fn action_permitted(
    principal: &AuthorizationInfo,
    action: &ReadManageAction,
    resource: &products::Model,
  ) -> Result<bool, Self::Error> {
    match action {
      ReadManageAction::Read => Ok(true),
      ReadManageAction::Manage => {
        if principal.has_scope("manage_conventions") {
          let convention_id = resource.convention_id;
          let has_permission = if let Some(convention_id) = convention_id {
            principal
              .has_convention_permission("update_products", convention_id)
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

impl EntityPolicy<AuthorizationInfo, products::Model> for ProductPolicy {
  type Action = ReadManageAction;

  fn id_column() -> products::Column {
    products::Column::Id
  }

  fn accessible_to(
    _principal: &AuthorizationInfo,
    action: &Self::Action,
  ) -> sea_orm::Select<<products::Model as sea_orm::ModelTrait>::Entity> {
    match action {
      ReadManageAction::Read => products::Entity::find(),
      ReadManageAction::Manage => products::Entity::find().filter(Expr::cust("0 = 1")),
    }
  }
}
