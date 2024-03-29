use crate::api::{
  objects::{CmsContentType, CmsLayoutType, CmsPartialType, PageType},
  policies::CmsContentPolicy,
};
use std::sync::Arc;

use async_graphql::*;
use intercode_entities::{cms_content_groups, permissions};
use intercode_graphql_core::{load_one_by_model_id, model_backed_type, ModelBackedType};
use intercode_graphql_loaders::{
  cms_content_group_contents_loader::CmsContentGroupItem, LoaderManager,
};
use intercode_policies::{AuthorizationInfo, Policy, ReadManageAction};
use seawater::loaders::ExpectModels;

model_backed_type!(CmsContentGroupCmsFields, cms_content_groups::Model);

impl CmsContentGroupCmsFields {
  pub async fn permissions(&self, ctx: &Context<'_>) -> Result<Vec<permissions::Model>> {
    let loader_result = load_one_by_model_id!(cms_content_group_permissions, ctx, self)?;
    loader_result.expect_models().cloned()
  }
}

#[Object]
impl CmsContentGroupCmsFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn contents(&self, ctx: &Context<'_>) -> Result<Vec<CmsContentType>> {
    let loader_result = ctx
      .data::<Arc<LoaderManager>>()?
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

  async fn name(&self) -> &str {
    &self.model.name
  }
}
