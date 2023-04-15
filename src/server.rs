use crate::actions;
use crate::actions::graphql::IntercodeSchema;
use crate::csrf::{csrf_middleware, CsrfConfig};
use crate::db_sessions::SessionWithDbStoreFromTxLayer;
use crate::request_bound_transaction::request_bound_transaction;
use crate::Localizations;
use axum::body::HttpBody;
use axum::extract::FromRef;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, post, IntoMakeService};
use axum::{Extension, Router};
use axum_server::tls_rustls::{RustlsAcceptor, RustlsConfig};
use axum_server::{Handle, Server};
use axum_sessions::SameSite;
use csrf::ChaCha20Poly1305CsrfProtection;
use i18n_embed::fluent::{fluent_language_loader, FluentLanguageLoader};
use i18n_embed::{I18nEmbedError, LanguageLoader};
use intercode_graphql::{build_intercode_graphql_schema, SchemaData};
use opentelemetry::global::shutdown_tracer_provider;
use sea_orm::DatabaseConnection;
use std::borrow::Cow;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::compression::CompressionLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::log::*;

#[derive(Debug)]
struct FatalDatabaseError {
  #[allow(dead_code)]
  db_err: sea_orm::DbErr,
}

#[derive(Clone, FromRef)]
pub struct AppState {
  schema: IntercodeSchema,
  schema_data: SchemaData,
  db_conn: Arc<DatabaseConnection>,
}

pub fn build_language_loader() -> Result<FluentLanguageLoader, I18nEmbedError> {
  let language_loader = fluent_language_loader!();
  language_loader.load_languages(&Localizations, &[language_loader.fallback_language()])?;

  Ok(language_loader)
}

async fn build_axum_server(
  addr: SocketAddr,
  handle: Handle,
) -> Result<Server<RustlsAcceptor>, Server> {
  if let (Ok(cert_path), Ok(key_path)) = (env::var("TLS_CERT_PATH"), env::var("TLS_KEY_PATH")) {
    let config = RustlsConfig::from_pem_file(cert_path, key_path)
      .await
      .map_err(|err| {
        warn!(
          "Falling back to unencrypted HTTP because of error reading cert and/or key: {}",
          err
        );
        axum_server::bind(addr).handle(handle.clone())
      })?;

    Ok(axum_server::bind_rustls(addr, config).handle(handle))
  } else {
    warn!(
      "TLS_CERT_PATH and/or TLS_KEY_PATH not present in env.  Falling back to unencrypted HTTP."
    );
    Err(axum_server::bind(addr).handle(handle))
  }
}

fn build_app<B: HttpBody + Send + Sync + Unpin + 'static>(
  db_conn: Arc<DatabaseConnection>,
) -> Result<IntoMakeService<Router<(), B>>, async_graphql::Error>
where
  axum::body::Bytes: From<<B as HttpBody>::Data>,
  <B as HttpBody>::Data: Send + Sync,
  <B as HttpBody>::Error: std::error::Error + Send + Sync,
{
  let language_loader_arc = Arc::new(build_language_loader()?);
  let schema_data = SchemaData {
    stripe_client: stripe::Client::new(env::var("STRIPE_SECRET_KEY")?),
    language_loader: language_loader_arc,
  };
  let graphql_schema = build_intercode_graphql_schema(schema_data.clone());

  let secret_bytes = hex::decode(env::var("SECRET_KEY_BASE")?)?;
  let secret: [u8; 64] = secret_bytes[0..64].try_into().unwrap_or_else(|_| {
    panic!(
      "SECRET_KEY_BASE is {} chars long but must be at least 128",
      secret_bytes.len() * 2
    )
  });

  let app_state = AppState {
    schema: graphql_schema,
    schema_data,
    db_conn,
  };

  let mut csrf_secret: [u8; 32] = Default::default();
  csrf_secret.clone_from_slice(&secret[0..32]);
  let protect = ChaCha20Poly1305CsrfProtection::from_key(csrf_secret);
  let session_layer = SessionWithDbStoreFromTxLayer::new(secret);
  let csrf_config = CsrfConfig {
    cookie_domain: None,
    cookie_http_only: true,
    cookie_len: 2048,
    cookie_name: "csrf-token".to_string(),
    cookie_path: Cow::from("/".to_string()),
    cookie_same_site: SameSite::Lax,
    cookie_secure: true,
    lifespan: Duration::from_secs(300),
    protect: Arc::new(protect),
  };

  let app = Router::new()
    .route(
      "/graphql",
      get(actions::graphql::graphql_playground).post(actions::graphql::graphql_handler),
    )
    .route(
      "/authenticity_tokens",
      get(actions::authenticity_tokens::authenticity_tokens),
    )
    .route("/users/sign_in", post(actions::authentication::sign_in))
    .route(
      "/reports/user_con_profiles/:user_con_profile_id",
      get(actions::reports::single_user_printable),
    )
    .fallback(actions::single_page_app_entry::single_page_app_entry)
    .layer(axum::middleware::from_fn(csrf_middleware))
    .layer(Extension(csrf_config))
    .layer(session_layer)
    .layer(from_fn_with_state(
      app_state.clone(),
      request_bound_transaction,
    ))
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
    .with_state(app_state)
    .into_make_service();
  Ok(app)
}

async fn graceful_shutdown(handle: Handle) {
  let duration = Duration::from_secs(
    env::var("SHUTDOWN_GRACE_PERIOD")
      .unwrap_or_else(|_| "30".to_string())
      .parse()
      .unwrap_or(30),
  );

  let immediate_shutdown_handle = handle.clone();
  tokio::spawn(async move {
    tokio::signal::ctrl_c().await.unwrap();
    info!("Received Ctrl-C; shutting down immediately");
    immediate_shutdown_handle.shutdown();
  });

  info!(
    "Starting graceful shutdown with {}-second timeout; press Ctrl-C again to shut down immediately",
    duration.as_secs()
  );

  handle.graceful_shutdown(Some(duration));

  while handle.connection_count() > 0 {
    sleep(Duration::from_secs(1)).await;
    info!("alive connections: {}", handle.connection_count());
  }
}

pub async fn serve(db: DatabaseConnection) -> async_graphql::Result<()> {
  let db_conn = Arc::new(db);
  let app = build_app::<hyper::Body>(db_conn)?;

  let addr = SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    env::var("PORT")
      .unwrap_or_else(|_| String::from("5901"))
      .parse()?,
  );

  let handle = Handle::new();
  let shutdown_handle = handle.clone();
  tokio::spawn(async move {
    tokio::signal::ctrl_c().await.unwrap();
    graceful_shutdown(shutdown_handle).await;
    info!("Shutting down server");
  });

  let server = build_axum_server(addr, handle).await;

  match server {
    Ok(server) => server.serve(app).await,
    Err(server) => server.serve(app).await,
  }?;

  shutdown_tracer_provider();

  Ok(())
}
