use async_graphql::*;
use intercode_entities::cms_variables;
use intercode_graphql_core::{model_backed_type, scalars::JsonScalar};
use intercode_policies::{AuthorizationInfo, Policy, ReadManageAction};

use crate::api::policies::CmsContentPolicy;

model_backed_type!(CmsVariableType, cms_variables::Model);

#[Object(name = "CmsVariable")]
impl CmsVariableType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "current_ability_can_delete")]
  async fn current_ability_can_delete(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;

    Ok(
      CmsContentPolicy::action_permitted(
        authorization_info,
        &ReadManageAction::Manage,
        self.model.as_ref(),
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
        self.model.as_ref(),
      )
      .await?,
    )
  }

  async fn key(&self) -> &str {
    &self.model.key
  }

  #[graphql(name = "value_json")]
  async fn value_json(&self) -> String {
    self
      .model
      .value
      .clone()
      .map(JsonScalar)
      .unwrap_or_default()
      .0
      .to_string()
  }
}
