use api::QueryRoot;
use async_graphql::{async_trait::async_trait, EmptyMutation, EmptySubscription, Schema};
use i18n_embed::fluent::FluentLanguageLoader;
use intercode_entities::{cms_parent::CmsParent, conventions, user_con_profiles, users};
use intercode_liquid::{
  cms_parent_partial_source::{LazyCmsPartialSource, PreloadPartialsStrategy},
  GraphQLExecutor,
};
use liquid::partials::LazyCompiler;
use loaders::LoaderManager;
use sea_orm::DatabaseConnection;
use seawater::ConnectionWrapper;
use std::{fmt::Debug, future::Future, sync::Arc};

pub mod api;
pub mod cms_rendering_context;
pub mod entity_relay_connection;
pub mod loaders;
mod policy_guard;

#[derive(Clone, Debug)]
pub struct SchemaData {
  pub db_conn: Arc<DatabaseConnection>,
  pub language_loader: Arc<FluentLanguageLoader>,
}

#[async_trait]
pub trait LiquidRenderer: Send + Sync + Debug {
  async fn render_liquid(
    &self,
    content: &str,
    globals: liquid::Object,
    preload_partials_strategy: Option<PreloadPartialsStrategy<'_>>,
  ) -> Result<String, async_graphql::Error>;
}

#[derive(Debug, Clone)]
pub struct QueryData {
  pub cms_parent: Arc<CmsParent>,
  pub current_user: Arc<Option<users::Model>>,
  pub convention: Arc<Option<conventions::Model>>,
  pub db: ConnectionWrapper,
  pub loaders: LoaderManager,
  // pub session_handle: Arc<RwLock<Session>>,
  pub timezone: chrono_tz::Tz,
  pub user_con_profile: Arc<Option<user_con_profiles::Model>>,
}

#[derive(Clone, Debug)]
pub struct EmbeddedGraphQLExecutor {
  schema_data: SchemaData,
  query_data: QueryData,
}

pub fn build_intercode_graphql_schema(
  schema_data: SchemaData,
) -> Schema<QueryRoot, EmptyMutation, EmptySubscription> {
  async_graphql::Schema::build(api::QueryRoot, EmptyMutation, EmptySubscription)
    .extension(async_graphql::extensions::Tracing)
    .data(schema_data)
    .finish()
}

impl GraphQLExecutor for EmbeddedGraphQLExecutor {
  fn execute(
    &self,
    request: impl Into<async_graphql::Request>,
  ) -> std::pin::Pin<Box<dyn Future<Output = async_graphql::Response> + Send + '_>> {
    let schema = build_intercode_graphql_schema(self.schema_data.clone());

    let request: async_graphql::Request = request.into();
    let request = request.data(self.query_data.clone());
    let response_future = async move { schema.execute(request).await };

    Box::pin(response_future)
  }
}

impl QueryData {
  pub async fn build_partial_compiler<'a>(
    &self,
    db: ConnectionWrapper,
    preload_partials_strategy: Option<PreloadPartialsStrategy<'a>>,
  ) -> Result<LazyCompiler<LazyCmsPartialSource>, liquid_core::Error> {
    let source = LazyCmsPartialSource::new(self.cms_parent.clone(), db.clone());

    if let Some(strategy) = preload_partials_strategy {
      source
        .preload(db.as_ref(), self.cms_parent.as_ref(), strategy)
        .await
        .map_err(|db_err| {
          liquid_core::Error::with_msg(format!("Error preloading partials: {}", db_err))
        })?;
    }

    Ok(LazyCompiler::new(source))
  }

  pub fn build_embedded_graphql_executor(
    &self,
    schema_data: &SchemaData,
  ) -> EmbeddedGraphQLExecutor {
    EmbeddedGraphQLExecutor {
      query_data: self.clone(),
      schema_data: schema_data.clone(),
    }
  }
}
