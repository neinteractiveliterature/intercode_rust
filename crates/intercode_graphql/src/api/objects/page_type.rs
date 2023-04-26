use async_graphql::*;
use intercode_entities::pages;
use intercode_liquid::cms_parent_partial_source::PreloadPartialsStrategy;
use intercode_policies::{policies::CmsContentPolicy, AuthorizationInfo, Policy, ReadManageAction};
use liquid::object;
use seawater::loaders::ExpectModel;

use crate::{
  api::objects::model_backed_type::ModelBackedType, cms_rendering_context::CmsRenderingContext,
  load_one_by_model_id, model_backed_type, LiquidRenderer, QueryData,
};

use super::CmsLayoutType;
model_backed_type!(PageType, pages::Model);

#[Object(name = "Page")]
impl PageType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(
    name = "admin_notes",
    guard = "self.simple_policy_guard::<CmsContentPolicy<pages::Model>>(ReadManageAction::Manage)"
  )]
  async fn admin_notes(&self) -> Option<&str> {
    self.model.admin_notes.as_deref()
  }

  #[graphql(name = "cms_layout")]
  async fn cms_layout(&self, ctx: &Context<'_>) -> Result<Option<CmsLayoutType>> {
    let loader_result = load_one_by_model_id!(pages_cms_layouts, ctx, self)?;
    Ok(loader_result.try_one().cloned().map(CmsLayoutType::new))
  }

  async fn content(&self) -> Option<&str> {
    self.model.content.as_deref()
  }

  #[graphql(name = "content_html")]
  async fn content_html(&self, ctx: &Context<'_>) -> Result<String, Error> {
    if let Some(content) = &self.model.content {
      let query_data = ctx.data::<QueryData>()?;
      let liquid_renderer = ctx.data::<Box<dyn LiquidRenderer>>()?;
      let cms_rendering_context =
        CmsRenderingContext::new(object!({}), query_data, liquid_renderer.as_ref());

      cms_rendering_context
        .render_liquid(
          content.as_str(),
          Some(PreloadPartialsStrategy::ByPage(&self.model)),
        )
        .await
    } else {
      Ok("".to_string())
    }
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

  #[graphql(name = "hidden_from_search")]
  async fn hidden_from_search(&self) -> bool {
    self.model.hidden_from_search
  }

  async fn name(&self) -> &Option<String> {
    &self.model.name
  }

  #[graphql(name = "skip_clickwrap_agreement")]
  async fn skip_clickwrap_agreement(&self) -> bool {
    self.model.skip_clickwrap_agreement
  }

  async fn slug(&self) -> &Option<String> {
    &self.model.slug
  }
}
