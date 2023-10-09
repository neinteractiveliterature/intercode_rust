use std::sync::Arc;

use axum::{
  debug_handler,
  extract::{OriginalUri, State},
  response::{self, IntoResponse},
};
use intercode_cms::CmsRenderingContext;
use intercode_entities::{cms_parent::CmsParentTrait, events};
use intercode_graphql::{actions::IntercodeSchema, build_intercode_graphql_schema};
use intercode_graphql_core::{schema_data::SchemaData, EmbeddedGraphQLExecutorBuilder};
use intercode_graphql_presend::get_presend_data;
use intercode_liquid_drops::IntercodeLiquidRenderer;
use intercode_server::AuthorizationInfoAndQueryDataFromRequest;
use liquid::object;
use once_cell::sync::Lazy;
use regex::Regex;
use sea_orm::{ColumnTrait, ModelTrait, QueryFilter};
use serde_json::json;

use crate::{liquid_renderer::LiquidRendererRequestDataInjector, server::AppState};

static EVENT_PATH_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("^/events/(\\d+)").unwrap());

#[debug_handler(state = AppState)]
pub async fn single_page_app_entry(
  OriginalUri(url): OriginalUri,
  State(schema): State<IntercodeSchema>,
  State(schema_data): State<SchemaData>,
  AuthorizationInfoAndQueryDataFromRequest(authorization_info, query_data): AuthorizationInfoAndQueryDataFromRequest,
) -> Result<impl IntoResponse, ::http::StatusCode> {
  let db = query_data.db();
  let path = url.path();
  let page_scope = query_data.cms_parent().cms_page_for_path(path);

  let page = if let Some(page_scope) = page_scope {
    page_scope
      .one(db.as_ref())
      .await
      .map_err(|_db_err| ::http::StatusCode::INTERNAL_SERVER_ERROR)?
  } else {
    None
  };

  let event = if let Some(convention) = query_data.convention() {
    if convention.site_mode == "single_event" {
      convention
        .find_related(events::Entity)
        .one(db.as_ref())
        .await
        .map_err(|_db_err| ::http::StatusCode::INTERNAL_SERVER_ERROR)?
    } else if let Some(event_captures) = EVENT_PATH_REGEX.captures(path) {
      let event_id = event_captures.get(1).unwrap().as_str().parse::<i64>();
      if let Ok(event_id) = event_id {
        convention
          .find_related(events::Entity)
          .filter(events::Column::Id.eq(event_id))
          .one(db.as_ref())
          .await
          .map_err(|_db_err| ::http::StatusCode::INTERNAL_SERVER_ERROR)?
      } else {
        None
      }
    } else {
      None
    }
  } else {
    None
  };

  let liquid_renderer = Arc::new(IntercodeLiquidRenderer::new(
    &query_data,
    &schema_data,
    EmbeddedGraphQLExecutorBuilder::new(
      build_intercode_graphql_schema(schema_data.clone()),
      query_data.clone_ref(),
      schema_data.clone(),
      Box::new(LiquidRendererRequestDataInjector::new(
        authorization_info.clone(),
      )),
    ),
  ));

  let cms_rendering_context =
    CmsRenderingContext::new(object!({}), &query_data, liquid_renderer.as_ref());
  // TODO
  let page_title = "TODO";

  let query_data = get_presend_data(
    &schema,
    query_data.clone_ref(),
    liquid_renderer.clone(),
    authorization_info,
    query_data.cms_parent(),
    path,
  )
  .await
  .unwrap_or_else(|_err| json!({}));

  Ok(response::Html(
    cms_rendering_context
      .render_app_root_content(
        &url,
        page_title,
        page.as_ref(),
        event.as_ref(),
        json!({ "queryData": query_data}),
      )
      .await,
  ))
}
