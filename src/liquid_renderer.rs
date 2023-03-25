use crate::drops::{DropContext, IntercodeGlobals};
use async_graphql::async_trait::async_trait;
use intercode_graphql::{
  build_partial_compiler, EmbeddedGraphQLExecutorBuilder, LiquidRenderer, QueryData, SchemaData,
};
use intercode_liquid::{build_liquid_parser, cms_parent_partial_source::PreloadPartialsStrategy};
use intercode_policies::AuthorizationInfo;
use std::{fmt::Debug, sync::Arc};

#[derive(Debug, Clone)]
pub struct IntercodeLiquidRenderer {
  query_data: QueryData,
  schema_data: SchemaData,
  authorization_info: AuthorizationInfo,
}

impl IntercodeLiquidRenderer {
  pub fn new(
    query_data: &QueryData,
    schema_data: &SchemaData,
    authorization_info: AuthorizationInfo,
  ) -> Self {
    IntercodeLiquidRenderer {
      query_data: query_data.clone(),
      schema_data: schema_data.clone(),
      authorization_info,
    }
  }
}

#[async_trait]
impl LiquidRenderer for IntercodeLiquidRenderer {
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
    let executor_builder = EmbeddedGraphQLExecutorBuilder::new(
      query_data.clone(),
      schema_data.clone(),
      self.authorization_info.clone(),
    );

    let parser = build_liquid_parser(
      query_data.convention(),
      Arc::downgrade(&schema_data.language_loader),
      query_data.cms_parent(),
      query_data.db().clone(),
      user_signed_in,
      Box::new(executor_builder),
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
