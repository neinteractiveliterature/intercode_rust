use async_graphql::*;
use intercode_entities::cms_content_groups;
use intercode_policies::{policies::CmsContentPolicy, AuthorizationInfo, Policy, ReadManageAction};
use seawater::loaders::ExpectModels;

use crate::{load_one_by_model_id, loader_result_to_many, model_backed_type};

use super::{ModelBackedType, PermissionType};
model_backed_type!(CmsContentGroupType, cms_content_groups::Model);

#[Object(name = "CmsContentGroup")]
impl CmsContentGroupType {
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

  async fn name(&self) -> &str {
    &self.model.name
  }

  async fn permissions(&self, ctx: &Context<'_>) -> Result<Vec<PermissionType>> {
    let loader_result = load_one_by_model_id!(cms_content_group_permissions, ctx, self)?;

    Ok(loader_result_to_many!(loader_result, PermissionType))
  }
}
