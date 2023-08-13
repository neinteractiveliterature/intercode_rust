use crate::server::AppState;
use async_graphql::{async_trait::async_trait, Request};
use axum::extract::{FromRequestParts, State};
use http::request::Parts;
use intercode_graphql::build_intercode_graphql_schema;
use intercode_graphql_core::{
  liquid_renderer::LiquidRendererFromRequest, query_data::QueryData, schema_data::SchemaData,
  EmbeddedGraphQLExecutorBuilder, RequestDataInjector,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_liquid_drops::IntercodeLiquidRenderer;
use intercode_policies::AuthorizationInfo;
use intercode_server::AuthorizationInfoAndQueryDataFromRequest;
use std::sync::Arc;

#[derive(Clone)]
pub struct LiquidRendererRequestDataInjector {
  authorization_info: AuthorizationInfo,
}

impl LiquidRendererRequestDataInjector {
  pub fn new(authorization_info: AuthorizationInfo) -> Self {
    Self { authorization_info }
  }
}

impl RequestDataInjector for LiquidRendererRequestDataInjector {
  fn inject_data(&self, request: Request, query_data: &QueryData) -> Request {
    let loader_manager = Arc::new(LoaderManager::new(query_data.db().clone()));

    request
      .data(loader_manager)
      .data(self.authorization_info.clone())
  }
}

#[async_trait]
impl FromRequestParts<AppState> for LiquidRendererFromRequest {
  type Rejection = http::StatusCode;

  async fn from_request_parts(
    parts: &mut Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    let State::<SchemaData>(schema_data) = State::from_request_parts(parts, state).await.unwrap();
    let AuthorizationInfoAndQueryDataFromRequest(authorization_info, query_data) =
      AuthorizationInfoAndQueryDataFromRequest::from_request_parts(parts, state).await?;

    let graphql_executor_builder = EmbeddedGraphQLExecutorBuilder::new(
      build_intercode_graphql_schema(schema_data.clone()),
      query_data.clone_ref(),
      schema_data.clone(),
      Box::new(LiquidRendererRequestDataInjector { authorization_info }),
    );

    let liquid_renderer =
      IntercodeLiquidRenderer::new(&query_data, &schema_data, graphql_executor_builder);
    Ok(Self(Arc::new(liquid_renderer)))
  }
}
