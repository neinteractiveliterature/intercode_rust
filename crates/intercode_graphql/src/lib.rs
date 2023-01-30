use api::QueryRoot;
use async_graphql::{async_trait::async_trait, EmptyMutation, EmptySubscription, Schema};
use i18n_embed::fluent::FluentLanguageLoader;
use intercode_entities::{cms_parent::CmsParent, conventions, user_con_profiles, users};
use intercode_liquid::{
  cms_parent_partial_source::{LazyCmsPartialSource, PreloadPartialsStrategy},
  tags::GraphQLExecutorBuilder,
  GraphQLExecutor,
};
use intercode_policies::AuthorizationInfo;
use liquid::partials::LazyCompiler;
use loaders::LoaderManager;
use seawater::ConnectionWrapper;
use std::{fmt::Debug, future::Future, sync::Arc};

pub mod api;
pub mod cms_rendering_context;
pub mod entity_relay_connection;
pub(crate) mod filter_utils;
mod lax_id;
pub mod loaders;
mod policy_guard;
mod presenters;

#[derive(Clone, Debug)]
pub struct SchemaData {
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

  async fn builtin_globals(
    &self,
  ) -> Result<Box<dyn liquid::ObjectView + Send>, async_graphql::Error>;
}

pub trait QueryDataContainer: Sync + Send + Debug {
  fn clone_ref(&self) -> Box<dyn QueryDataContainer>;
  fn cms_parent(&self) -> &CmsParent;
  fn current_user(&self) -> Option<&users::Model>;
  fn convention(&self) -> Option<&conventions::Model>;
  fn db(&self) -> &ConnectionWrapper;
  fn loaders(&self) -> &LoaderManager;
  fn timezone(&self) -> &chrono_tz::Tz;
  fn user_con_profile(&self) -> Option<&user_con_profiles::Model>;
}

pub type QueryData = Box<dyn QueryDataContainer>;

impl Clone for QueryData {
  fn clone(&self) -> Self {
    self.clone_ref()
  }
}

#[derive(Debug)]
pub struct OwnedQueryData {
  pub cms_parent: CmsParent,
  pub current_user: Option<users::Model>,
  pub convention: Option<conventions::Model>,
  pub db: ConnectionWrapper,
  pub loaders: LoaderManager,
  pub timezone: chrono_tz::Tz,
  pub user_con_profile: Option<user_con_profiles::Model>,
}

impl OwnedQueryData {
  pub fn new(
    cms_parent: CmsParent,
    current_user: Option<users::Model>,
    convention: Option<conventions::Model>,
    db: ConnectionWrapper,
    timezone: chrono_tz::Tz,
    user_con_profile: Option<user_con_profiles::Model>,
  ) -> Self {
    OwnedQueryData {
      cms_parent,
      current_user,
      convention,
      db: db.clone(),
      loaders: LoaderManager::new(db),
      timezone,
      user_con_profile,
    }
  }
}

#[derive(Debug)]
pub struct ArcQueryData {
  owned_query_data: Arc<OwnedQueryData>,
}

impl ArcQueryData {
  pub fn new(owned_query_data: OwnedQueryData) -> Self {
    ArcQueryData {
      owned_query_data: Arc::new(owned_query_data),
    }
  }
}

impl QueryDataContainer for ArcQueryData {
  fn clone_ref(&self) -> Box<dyn QueryDataContainer> {
    Box::new(ArcQueryData {
      owned_query_data: self.owned_query_data.clone(),
    })
  }

  fn cms_parent(&self) -> &CmsParent {
    &self.owned_query_data.cms_parent
  }

  fn current_user(&self) -> Option<&users::Model> {
    self.owned_query_data.current_user.as_ref()
  }

  fn convention(&self) -> Option<&conventions::Model> {
    self.owned_query_data.convention.as_ref()
  }

  fn db(&self) -> &ConnectionWrapper {
    &self.owned_query_data.db
  }

  fn loaders(&self) -> &LoaderManager {
    &self.owned_query_data.loaders
  }

  fn timezone(&self) -> &chrono_tz::Tz {
    &self.owned_query_data.timezone
  }

  fn user_con_profile(&self) -> Option<&user_con_profiles::Model> {
    self.owned_query_data.user_con_profile.as_ref()
  }
}

#[derive(Debug)]
pub struct EmbeddedGraphQLExecutor {
  schema_data: SchemaData,
  query_data: QueryData,
  authorization_info: Arc<AuthorizationInfo>,
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
    request: async_graphql::Request,
  ) -> std::pin::Pin<Box<dyn Future<Output = async_graphql::Response> + Send + '_>> {
    let schema = build_intercode_graphql_schema(self.schema_data.clone());

    let request = request
      .data(self.query_data.clone_ref())
      .data(self.authorization_info.clone());
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
      authorization_info: Arc::new(self.authorization_info.clone()),
    })
  }
}
