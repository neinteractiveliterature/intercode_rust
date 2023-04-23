use async_graphql::*;
use intercode_entities::pages;
use intercode_liquid::cms_parent_partial_source::PreloadPartialsStrategy;
use intercode_policies::{policies::CmsContentPolicy, ReadManageAction};
use liquid::object;

use crate::{
  api::objects::model_backed_type::ModelBackedType, cms_rendering_context::CmsRenderingContext,
  model_backed_type, LiquidRenderer, QueryData,
};
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
  async fn current_ability_can_delete(&self, _ctx: &Context<'_>) -> bool {
    // TODO
    false
  }

  #[graphql(name = "current_ability_can_update")]
  async fn current_ability_can_update(&self, _ctx: &Context<'_>) -> bool {
    // TODO
    false
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
