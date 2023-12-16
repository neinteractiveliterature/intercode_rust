use std::sync::Arc;

use crate::api;
use async_graphql::{
  http::{playground_source, GraphQLPlaygroundConfig},
  EmptySubscription, Schema,
};
use async_graphql_axum::{GraphQLBatchRequest, GraphQLResponse};
use axum::response::{self, IntoResponse};
use intercode_graphql_core::liquid_renderer::{LiquidRenderer, LiquidRendererFromRequest};
use intercode_graphql_loaders::LoaderManager;
use intercode_server::AuthorizationInfoAndQueryDataFromRequest;

pub type IntercodeSchema = Schema<api::QueryRoot, api::MutationRoot, EmptySubscription>;

pub async fn graphql_handler_inner(
  schema: IntercodeSchema,
  AuthorizationInfoAndQueryDataFromRequest(authorization_info, query_data): AuthorizationInfoAndQueryDataFromRequest,
  LiquidRendererFromRequest(liquid_renderer): LiquidRendererFromRequest,
  req: GraphQLBatchRequest,
) -> GraphQLResponse {
  let loader_manager = Arc::new(LoaderManager::new(query_data.db().clone()));
  let req = req
    .into_inner()
    .data(query_data)
    .data(loader_manager)
    .data::<Arc<dyn LiquidRenderer>>(liquid_renderer)
    .data(authorization_info);

  let req = intercode_store::inject_request_data(req);
  let req = intercode_notifiers::inject_request_data(req);

  schema.execute_batch(req).await.into()
}

pub async fn graphql_playground() -> impl IntoResponse {
  response::Html(playground_source(
    GraphQLPlaygroundConfig::new("/graphql").with_setting("schema.polling.interval", 10000),
  ))
}
