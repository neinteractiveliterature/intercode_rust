use async_graphql::*;
use intercode_entities::pages;
use intercode_liquid::cms_parent_partial_source::PreloadPartialsStrategy;
use liquid::object;

use crate::{model_backed_type, QueryData, SchemaData};
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
      let schema_data = ctx.data::<SchemaData>()?;
      let query_data = ctx.data::<QueryData>()?;
      query_data
        .render_liquid(
          schema_data,
          content.as_str(),
          object!({}),
          Some(PreloadPartialsStrategy::ByPage(&self.model)),
        )
        .await
    } else {
      Ok("".to_string())
    }
  }
}
