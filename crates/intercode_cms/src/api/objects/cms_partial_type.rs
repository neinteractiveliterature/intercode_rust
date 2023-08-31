use async_graphql::*;
use intercode_entities::cms_partials;
use intercode_graphql_core::model_backed_type;
use intercode_policies::{
  AuthorizationInfo, ModelBackedTypeGuardablePolicy, Policy, ReadManageAction,
};

use crate::api::policies::CmsPartialPolicy;

model_backed_type!(CmsPartialType, cms_partials::Model);

#[Object(name = "CmsPartial")]
impl CmsPartialType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(
    name = "admin_notes",
    guard = "CmsPartialPolicy::model_guard(ReadManageAction::Manage, self)"
  )]
  async fn admin_notes(&self) -> Option<&str> {
    self.model.admin_notes.as_deref()
  }

  async fn content(&self) -> Option<&str> {
    self.model.content.as_deref()
  }

  #[graphql(name = "current_ability_can_delete")]
  async fn current_ability_can_delete(&self, ctx: &Context<'_>) -> Result<bool> {
    let authorization_info = ctx.data::<AuthorizationInfo>()?;

    Ok(
      CmsPartialPolicy::action_permitted(
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
      CmsPartialPolicy::action_permitted(
        authorization_info,
        &ReadManageAction::Manage,
        &self.model,
      )
      .await?,
    )
  }

  async fn name(&self) -> &str {
    &self.model.name
  }
}
