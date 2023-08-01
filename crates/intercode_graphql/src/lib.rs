use api::{MutationRoot, QueryRoot};
use async_graphql::{EmptySubscription, Schema};
use intercode_entities::cms_parent::CmsParent;
use intercode_graphql_core::{query_data::QueryData, schema_data::SchemaData};
use intercode_graphql_loaders::LoaderManager;
use intercode_liquid::{
  cms_parent_partial_source::{LazyCmsPartialSource, PreloadPartialsStrategy},
  tags::GraphQLExecutorBuilder,
  GraphQLExecutor,
};
use intercode_policies::AuthorizationInfo;
use liquid::partials::LazyCompiler;
use seawater::ConnectionWrapper;
use std::{fmt::Debug, future::Future, sync::Arc};

pub mod api;

#[derive(Debug)]
pub struct EmbeddedGraphQLExecutor {
  schema_data: SchemaData,
  query_data: QueryData,
  authorization_info: AuthorizationInfo,
}

pub fn build_intercode_graphql_schema(
  schema_data: SchemaData,
) -> Schema<QueryRoot, MutationRoot, EmptySubscription> {
  async_graphql::Schema::build(
    api::QueryRoot::default(),
    api::MutationRoot,
    EmptySubscription,
  )
  .extension(async_graphql::extensions::Tracing)
  .data(schema_data)
  .finish()
}

impl GraphQLExecutor for EmbeddedGraphQLExecutor {
  fn execute(
    &self,
    request: async_graphql::Request,
  ) -> std::pin::Pin<Box<dyn Future<Output = async_graphql::Response> + Send + '_>> {
    let schema = build_intercode_graphql_schema(self.schema_data.clone());
    let loader_manager = Arc::new(LoaderManager::new(self.query_data.db().clone()));

    let request = request
      .data(self.query_data.clone_ref())
      .data(self.authorization_info.clone())
      .data(loader_manager);
    let response_future = async move { schema.execute(request).await };

    Box::pin(response_future)
  }
}

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

#[derive(Debug, Clone)]
pub struct EmbeddedGraphQLExecutorBuilder {
  query_data: QueryData,
  schema_data: SchemaData,
  authorization_info: AuthorizationInfo,
}

impl EmbeddedGraphQLExecutorBuilder {
  pub fn new(
    query_data: QueryData,
    schema_data: SchemaData,
    authorization_info: AuthorizationInfo,
  ) -> Self {
    EmbeddedGraphQLExecutorBuilder {
      query_data,
      schema_data,
      authorization_info,
    }
  }
}

impl GraphQLExecutorBuilder for EmbeddedGraphQLExecutorBuilder {
  fn build_executor(&self) -> Box<dyn GraphQLExecutor> {
    Box::new(EmbeddedGraphQLExecutor {
      query_data: self.query_data.clone_ref(),
      schema_data: self.schema_data.clone(),
      authorization_info: self.authorization_info.clone(),
    })
  }
}
