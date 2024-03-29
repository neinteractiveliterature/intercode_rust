use std::env;
use std::sync::Arc;

use async_graphql::Result;
use axum::error_handling::HandleErrorLayer;
use axum::extract::FromRef;
use axum::BoxError;
use axum::{middleware::from_fn_with_state, routing::IntoMakeService, Extension, Router};
use http::StatusCode;
use sea_orm::DatabaseConnection;
use tower::limit::ConcurrencyLimitLayer;
use tower::ServiceBuilder;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::compression::CompressionLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};

use crate::{
  csrf::{csrf_middleware, CsrfConfig},
  db_sessions::SessionWithDbStoreFromTxLayer,
  request_bound_transaction::request_bound_transaction,
};

pub fn build_app<S, F>(state: S, build_routes: F) -> Result<IntoMakeService<Router>>
where
  S: Clone + Send + Sync + 'static,
  Arc<DatabaseConnection>: FromRef<S>,
  F: FnOnce(Router<S>) -> Router<S>,
{
  let secret_bytes = hex::decode(env::var("SECRET_KEY_BASE")?)?;
  let secret: [u8; 64] = secret_bytes[0..64].try_into().unwrap_or_else(|_| {
    panic!(
      "SECRET_KEY_BASE is {} chars long but must be at least 128",
      secret_bytes.len() * 2
    )
  });

  let csrf_config = CsrfConfig::new(&secret);
  let session_layer = SessionWithDbStoreFromTxLayer::new();

  let app: Router<S> = Router::new();
  let app = build_routes(app);

  let session_service = ServiceBuilder::new()
    .layer(HandleErrorLayer::new(|err: BoxError| async move {
      (StatusCode::BAD_REQUEST, err.to_string())
    }))
    .layer(session_layer);

  let app = app
    .layer(axum::middleware::from_fn(csrf_middleware))
    .layer(Extension(csrf_config))
    .layer(session_service)
    .layer(from_fn_with_state(state.clone(), request_bound_transaction))
    .layer(ConcurrencyLimitLayer::new(
      env::var("MAX_CONCURRENCY")
        .unwrap_or_else(|_| "25".to_string())
        .parse()
        .unwrap_or(25),
    ))
    .layer(CompressionLayer::new())
    .layer(
      TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
    )
    .layer(CatchPanicLayer::new())
    .with_state(state)
    .into_make_service();

  Ok(app)
}
