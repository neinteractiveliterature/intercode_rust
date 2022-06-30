use async_graphql::*;
use intercode_entities::pages;
use intercode_liquid::cms_parent_partial_source::PreloadPartialsStrategy;
use liquid::object;

use crate::{cms_rendering_context::CmsRenderingContext, model_backed_type, QueryData, SchemaData};
model_backed_type!(PageType, pages::Model);

#[Object]
impl PageType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "content_html")]
  async fn content_html(&self, ctx: &Context<'_>) -> Result<String, Error> {
    if let Some(content) = &self.model.content {
      let schema_data = ctx.data::<SchemaData>()?;
      let query_data = ctx.data::<QueryData>()?;
      let cms_rendering_context = CmsRenderingContext::new(object!({}), schema_data, query_data);

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
