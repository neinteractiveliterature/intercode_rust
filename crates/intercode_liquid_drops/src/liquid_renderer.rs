use crate::drops::{DropContext, IntercodeGlobals};
use async_graphql::async_trait::async_trait;
use intercode_entities::cms_parent::CmsParent;
use intercode_graphql_core::{
  liquid_renderer::LiquidRenderer, query_data::QueryData, schema_data::SchemaData,
};
use intercode_liquid::{
  build_liquid_parser,
  cms_parent_partial_source::{LazyCmsPartialSource, PreloadPartialsStrategy},
  tags::GraphQLExecutorBuilder,
};
use liquid_core::partials::LazyCompiler;
use seawater::ConnectionWrapper;
use std::{fmt::Debug, sync::Arc};

pub async fn build_partial_compiler(
  cms_parent: CmsParent,
  db: ConnectionWrapper,
  preload_partials_strategy: Option<PreloadPartialsStrategy<'_>>,
) -> Result<LazyCompiler<LazyCmsPartialSource>, liquid_core::Error> {
  let source = LazyCmsPartialSource::new(cms_parent.clone(), db.clone());

  if let Some(strategy) = preload_partials_strategy {
    source
      .preload(db.as_ref(), strategy)
      .await
      .map_err(|db_err| {
        liquid_core::Error::with_msg(format!("Error preloading partials: {}", db_err))
      })?;
  }

  Ok(LazyCompiler::new(source))
}

#[derive(Clone)]
pub struct IntercodeLiquidRenderer<B: GraphQLExecutorBuilder> {
  query_data: QueryData,
  schema_data: SchemaData,
  graphql_executor_builder: B,
}

impl<B: GraphQLExecutorBuilder> Debug for IntercodeLiquidRenderer<B> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("IntercodeLiquidRenderer")
      .field("query_data", &self.query_data)
      .field("schema_data", &self.schema_data)
      .finish_non_exhaustive()
  }
}

impl<B: GraphQLExecutorBuilder> IntercodeLiquidRenderer<B> {
  pub fn new(
    query_data: &QueryData,
    schema_data: &SchemaData,
    graphql_executor_builder: B,
  ) -> Self {
    IntercodeLiquidRenderer {
      query_data: query_data.clone(),
      schema_data: schema_data.clone(),
      graphql_executor_builder,
    }
  }
}

#[async_trait]
impl<B: GraphQLExecutorBuilder + Clone + 'static> LiquidRenderer for IntercodeLiquidRenderer<B> {
  async fn builtin_globals(
    &self,
  ) -> Result<Box<dyn liquid::ObjectView + Send>, async_graphql::Error> {
    todo!()
  }

  async fn render_liquid(
    &self,
    content: &str,
    globals: liquid::Object,
    preload_partials_strategy: Option<PreloadPartialsStrategy<'_>>,
  ) -> Result<String, async_graphql::Error> {
    let schema_data: SchemaData = self.schema_data.clone();
    let query_data: QueryData = self.query_data.clone();
    let cms_parent = query_data.cms_parent().clone();

    let partial_compiler = build_partial_compiler(
      cms_parent,
      query_data.db().clone(),
      preload_partials_strategy,
    )
    .await?;
    let user_signed_in = query_data.current_user().is_some();

    let parser = build_liquid_parser(
      query_data.convention(),
      Arc::downgrade(&schema_data.language_loader),
      query_data.cms_parent(),
      query_data.db().clone(),
      user_signed_in,
      Box::new(self.graphql_executor_builder.clone()),
      partial_compiler,
    )?;

    let renderer = seawater::Renderer::new(
      parser,
      move |store| DropContext::new(schema_data.clone(), query_data.clone(), store),
      IntercodeGlobals::new,
    );
    renderer.render_liquid(content, globals).await
  }
}
