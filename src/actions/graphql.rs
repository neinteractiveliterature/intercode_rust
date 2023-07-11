use std::sync::Arc;

use async_graphql::{
  http::{playground_source, GraphQLPlaygroundConfig},
  EmptySubscription, Schema,
};
use async_graphql_axum::{GraphQLBatchRequest, GraphQLResponse};
use axum::{
  debug_handler,
  extract::State,
  response::{self, IntoResponse},
};
use intercode_graphql::{api, SchemaData};
use intercode_graphql_core::liquid_renderer::LiquidRenderer;
use intercode_graphql_loaders::LoaderManager;

use crate::{
  liquid_renderer::IntercodeLiquidRenderer, middleware::AuthorizationInfoAndQueryDataFromRequest,
};

#[allow(unused_imports)]
use crate::server::AppState;

pub type IntercodeSchema = Schema<api::QueryRoot, api::MutationRoot, EmptySubscription>;

#[debug_handler(state = AppState)]
pub async fn graphql_handler(
  State(schema): State<IntercodeSchema>,
  State(schema_data): State<SchemaData>,
  AuthorizationInfoAndQueryDataFromRequest(authorization_info, query_data): AuthorizationInfoAndQueryDataFromRequest,
  req: GraphQLBatchRequest,
) -> GraphQLResponse {
  let liquid_renderer =
    IntercodeLiquidRenderer::new(&query_data, &schema_data, authorization_info.clone());
  let loader_manager = Arc::new(LoaderManager::new(query_data.db().clone()));
  let req = req
    .into_inner()
    .data(query_data)
    .data(loader_manager)
    .data::<Arc<dyn LiquidRenderer>>(Arc::new(liquid_renderer))
    .data(authorization_info);

  schema.execute_batch(req).await.into()
}

pub async fn graphql_playground() -> impl IntoResponse {
  response::Html(playground_source(
    GraphQLPlaygroundConfig::new("/graphql").with_setting("schema.polling.interval", 10000),
  ))
}
