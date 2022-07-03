use async_graphql::{async_trait::async_trait, EmptyMutation, EmptySubscription};
use i18n_embed::fluent::FluentLanguageLoader;
use intercode_entities::{cms_parent::CmsParent, conventions, user_con_profiles, users};
use intercode_liquid::{
  cms_parent_partial_source::{LazyCmsPartialSource, PreloadPartialsStrategy},
  GraphQLExecutor,
};
use liquid::partials::LazyCompiler;
use sea_orm::DatabaseConnection;
use std::{fmt::Debug, future::Future, sync::Arc};

pub mod api;
pub mod cms_rendering_context;
pub mod entity_relay_connection;
pub mod loaders;

#[derive(Debug, Clone)]
pub struct SchemaData {
  pub db: Arc<DatabaseConnection>,
  pub language_loader: Arc<FluentLanguageLoader>,
  pub loaders: loaders::LoaderManager,
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
  pub timezone: chrono_tz::Tz,
  pub user_con_profile: Arc<Option<user_con_profiles::Model>>,
}

#[derive(Clone, Debug)]
pub struct EmbeddedGraphQLExecutor {
  schema_data: SchemaData,
  query_data: QueryData,
}

impl GraphQLExecutor for EmbeddedGraphQLExecutor {
  fn execute(
    &self,
    request: impl Into<async_graphql::Request>,
  ) -> std::pin::Pin<Box<dyn Future<Output = async_graphql::Response> + Send + '_>> {
    let schema = async_graphql::Schema::build(api::QueryRoot, EmptyMutation, EmptySubscription)
      .extension(async_graphql::extensions::Tracing)
      .data(self.schema_data.clone())
      .finish();

    let request: async_graphql::Request = request.into();
    let request = request.data(self.query_data.clone());
    let response_future = async move { schema.execute(request).await };

    Box::pin(response_future)
  }
}

impl QueryData {
  pub fn new(
    cms_parent: Arc<CmsParent>,
    current_user: Arc<Option<users::Model>>,
    convention: Arc<Option<conventions::Model>>,
    timezone: chrono_tz::Tz,
    user_con_profile: Arc<Option<user_con_profiles::Model>>,
  ) -> QueryData {
    QueryData {
      cms_parent,
      current_user,
      convention,
      timezone,
      user_con_profile,
    }
  }

  pub async fn build_partial_compiler<'a>(
    &self,
    schema_data: &SchemaData,
    preload_partials_strategy: Option<PreloadPartialsStrategy<'a>>,
  ) -> Result<LazyCompiler<LazyCmsPartialSource>, liquid_core::Error> {
    let source = LazyCmsPartialSource::new(self.cms_parent.clone(), schema_data.db.clone());

    if let Some(strategy) = preload_partials_strategy {
      source
        .preload(schema_data.db.as_ref(), self.cms_parent.as_ref(), strategy)
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
