use async_trait::async_trait;
use intercode_liquid::cms_parent_partial_source::PreloadPartialsStrategy;
use std::fmt::Debug;

#[async_trait]
pub trait LiquidRenderer: Send + Sync + Debug {
  async fn render_liquid(
    &self,
    content: &str,
    globals: liquid::Object,
    preload_partials_strategy: Option<PreloadPartialsStrategy<'_>>,
  ) -> Result<String, async_graphql::Error>;

  async fn builtin_globals(
    &self,
  ) -> Result<Box<dyn liquid::ObjectView + Send>, async_graphql::Error>;
}
