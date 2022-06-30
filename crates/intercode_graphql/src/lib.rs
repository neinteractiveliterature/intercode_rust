use async_graphql::{EmptyMutation, EmptySubscription};
use i18n_embed::fluent::FluentLanguageLoader;
use intercode_entities::{cms_parent::CmsParent, conventions, user_con_profiles, users};
use intercode_liquid::{
  build_liquid_parser,
  cms_parent_partial_source::{LazyCmsPartialSource, PreloadPartialsStrategy},
  GraphQLExecutor,
};
use liquid::{object, partials::LazyCompiler};
use sea_orm::DatabaseConnection;
use std::{future::Future, sync::Arc};

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

  pub async fn render_liquid(
    &self,
    schema_data: &SchemaData,
    content: &str,
    globals: liquid::Object,
    preload_partials_strategy: Option<PreloadPartialsStrategy<'_>>,
  ) -> Result<String, async_graphql::Error> {
    let schema_data: SchemaData = schema_data.clone();
    let query_data: QueryData = self.clone();

    let partial_compiler = query_data
      .build_partial_compiler(&schema_data, preload_partials_strategy)
      .await?;
    let convention = query_data.convention.clone();
    let language_loader = schema_data.language_loader.clone();
    let cms_parent = query_data.cms_parent.clone();
    let db = schema_data.db.clone();
    let user_signed_in = query_data.current_user.is_some();
    let executor = query_data.build_embedded_graphql_executor(&schema_data);

    let parser = build_liquid_parser(
      &convention,
      &language_loader,
      &cms_parent,
      &db,
      user_signed_in,
      executor,
      partial_compiler,
    )?;

    let mut all_globals = object!({
      "convention": self.convention,
      "user_con_profile": self.user_con_profile
    });
    all_globals.extend(globals);

    let template = parser.parse(content)?;
    let result = template.render(&all_globals);

    match result {
      Ok(content) => Ok(content),
      Err(error) => Err(async_graphql::Error::new(error.to_string())),
    }
  }
}
