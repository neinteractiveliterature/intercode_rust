use crate::actions;
use crate::database::connect_database;
use axum::body::HttpBody;
use axum::extract::FromRef;
use axum::routing::{get, post, IntoMakeService};
use axum::Router;
use intercode_graphql::actions::IntercodeSchema;
use intercode_graphql::build_intercode_graphql_schema;
use intercode_graphql_core::schema_data::SchemaData;
use intercode_server::i18n::build_language_loader;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone, FromRef)]
pub struct AppState {
  schema: IntercodeSchema,
  schema_data: SchemaData,
  db_conn: Arc<DatabaseConnection>,
}

pub async fn bootstrap_app<B>() -> Result<IntoMakeService<Router<(), B>>, async_graphql::Error>
where
  axum::body::Bytes: From<<B as HttpBody>::Data>,
  B: HttpBody + Send + Sync + Unpin + 'static,
  <B as HttpBody>::Data: Send + Sync,
  <B as HttpBody>::Error: std::error::Error + Send + Sync,
{
  let db_conn = Arc::new(connect_database().await?);
  let language_loader_arc = Arc::new(build_language_loader()?);
  let schema_data = SchemaData {
    language_loader: language_loader_arc,
  };
  let graphql_schema = build_intercode_graphql_schema(schema_data.clone());

  let app_state = AppState {
    schema: graphql_schema,
    schema_data,
    db_conn,
  };

  intercode_server::build_app(app_state, |router| {
    router
      .route(
        "/graphql",
        get(intercode_graphql::actions::graphql_playground)
          .post(intercode_graphql::actions::graphql_handler),
      )
      .route(
        "/authenticity_tokens",
        get(intercode_server::actions::authenticity_tokens),
      )
      .route("/users/sign_in", post(intercode_users::actions::sign_in))
      .route(
        "/reports/user_con_profiles/:user_con_profile_id",
        get(intercode_reporting::actions::single_user_printable),
      )
      .route(
        "/calendars/user_schedule/:ical_secret",
        get(intercode_signups::actions::user_schedule),
      )
      .fallback(actions::single_page_app_entry::single_page_app_entry)
  })
}
