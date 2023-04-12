use axum::{
  debug_handler,
  extract::{OriginalUri, State},
  response::{self, IntoResponse},
};
use intercode_entities::{cms_parent::CmsParentTrait, events};
use intercode_graphql::{cms_rendering_context::CmsRenderingContext, SchemaData};
use liquid::object;
use once_cell::sync::Lazy;
use regex::Regex;
use sea_orm::{ColumnTrait, ModelTrait, QueryFilter};

use crate::{
  liquid_renderer::IntercodeLiquidRenderer, middleware::AuthorizationInfoAndQueryDataFromRequest,
};

static EVENT_PATH_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("^/events/(\\d+)").unwrap());

#[debug_handler]
pub async fn single_page_app_entry(
  OriginalUri(url): OriginalUri,
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

  let liquid_renderer = IntercodeLiquidRenderer::new(&query_data, &schema_data, authorization_info);

  let cms_rendering_context = CmsRenderingContext::new(object!({}), &query_data, &liquid_renderer);
  let page_title = "TODO";

  Ok(response::Html(
    cms_rendering_context
      .render_app_root_content(&url, page_title, page.as_ref(), event.as_ref())
      .await,
  ))
}
