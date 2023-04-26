use async_graphql::*;
use intercode_entities::cms_content_groups;
use intercode_policies::{policies::CmsContentPolicy, AuthorizationInfo, Policy, ReadManageAction};
use seawater::loaders::ExpectModels;

use crate::{
  load_one_by_model_id, loader_result_to_many,
  loaders::cms_content_group_contents_loader::CmsContentGroupItem, model_backed_type, QueryData,
};

use super::{
  CmsContentType, CmsLayoutType, CmsPartialType, ModelBackedType, PageType, PermissionType,
};
model_backed_type!(CmsContentGroupType, cms_content_groups::Model);

#[Object(name = "CmsContentGroup")]
impl CmsContentGroupType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn contents(&self, ctx: &Context<'_>) -> Result<Vec<CmsContentType>> {
    let loader_result = ctx
      .data::<QueryData>()?
      .loaders()
      .cms_content_group_contents
      .load_one(self.model.id)
      .await?
      .unwrap_or_default();

    Ok(
      loader_result
        .into_iter()
        .map(|item| match item {
          CmsContentGroupItem::Page(page) => CmsContentType::Page(PageType::new(page)),
          CmsContentGroupItem::CmsLayout(cms_layout) => {
            CmsContentType::Layout(CmsLayoutType::new(cms_layout))
          }
          CmsContentGroupItem::CmsPartial(cms_partial) => {
            CmsContentType::Partial(CmsPartialType::new(cms_partial))
          }
        })
        .collect(),
    )
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
