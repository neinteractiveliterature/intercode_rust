use async_graphql::*;
use intercode_entities::cms_graphql_queries;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_policies::{AuthorizationInfo, Policy, ReadManageAction};

use crate::api::policies::CmsContentPolicy;

model_backed_type!(CmsGraphqlQueryType, cms_graphql_queries::Model);

#[Object(name = "CmsGraphqlQuery")]
impl CmsGraphqlQueryType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(
    name = "admin_notes",
    guard = "self.simple_policy_guard::<CmsContentPolicy<cms_graphql_queries::Model>>(ReadManageAction::Manage)"
  )]
  async fn admin_notes(&self) -> Option<&str> {
    self.model.admin_notes.as_deref()
  }

  #[graphql(name = "current_ability_can_delete")]
  async fn current_ability_can_delete(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;

    Ok(
      CmsContentPolicy::action_permitted(
        authorization_info,
        &ReadManageAction::Manage,
        &self.model,
      )
      .await?,
    )
  }

  #[graphql(name = "current_ability_can_update")]
  async fn current_ability_can_update(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;

    Ok(
      CmsContentPolicy::action_permitted(
        authorization_info,
        &ReadManageAction::Manage,
        &self.model,
      )
      .await?,
    )
  }

  async fn identifier(&self) -> Option<&str> {
    self.model.identifier.as_deref()
  }

  async fn query(&self) -> Option<&str> {
    self.model.query.as_deref()
  }
}
