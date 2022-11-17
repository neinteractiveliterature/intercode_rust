use std::sync::Arc;

use async_graphql::*;
use intercode_entities::cms_layouts;
use intercode_liquid::{cms_parent_partial_source::PreloadPartialsStrategy, react_component_tag};
use liquid::object;
use serde_json::json;

use crate::{
  cms_rendering_context::CmsRenderingContext, model_backed_type, LiquidRenderer, QueryData,
  SchemaData,
};
model_backed_type!(CmsLayoutType, cms_layouts::Model);

const DEFAULT_NAVBAR_CLASSES: &str =
  "navbar-fixed-top navbar-expand-md mb-4 navbar-dark bg-intercode-blue";

#[Object(name = "CmsLayout")]
impl CmsLayoutType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "content_html")]
  #[allow(unused_variables)]
  async fn content_html(&self, ctx: &Context<'_>, path: Option<String>) -> Result<String, Error> {
    let schema_data = ctx.data::<SchemaData>()?;
    let query_data = ctx.data::<QueryData>()?;
    let liquid_renderer = ctx.data::<Arc<dyn LiquidRenderer>>()?;

    let cms_rendering_context = CmsRenderingContext::new(
      object!({
        "content_for_head": "",
        "content_for_navbar": react_component_tag("NavigationBar", json!({
          "navbarClasses": self.model.navbar_classes.as_deref().unwrap_or(DEFAULT_NAVBAR_CLASSES)
        })),
        "content_for_layout": react_component_tag("AppRouter", json!({}))
      }),
      query_data,
      liquid_renderer.clone(),
    );

    cms_rendering_context
      .render_liquid(
        self.model.content.as_deref().unwrap_or(""),
        Some(PreloadPartialsStrategy::ByLayout(&self.model)),
      )
      .await
  }
}
