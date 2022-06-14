use crate::{
  cms_parent::PreloadPartialsStrategy, liquid_extensions::parse_and_render_in_graphql_context,
  pages,
};
use async_graphql::*;

use crate::model_backed_type;
model_backed_type!(PageType, pages::Model);

#[Object]
impl PageType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn name(&self) -> &Option<String> {
    &self.model.name
  }

  #[graphql(name = "content_html")]
  async fn content_html(&self, ctx: &Context<'_>) -> Result<String, Error> {
    if let Some(content) = &self.model.content {
      parse_and_render_in_graphql_context(
        ctx,
        content.as_str(),
        Some(PreloadPartialsStrategy::ByPage(&self.model)),
      )
      .await
    } else {
      Ok("".to_string())
    }
  }
}
