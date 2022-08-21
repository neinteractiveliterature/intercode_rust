use crate::liquid_renderer::IntercodeLiquidRenderer;
use crate::middleware::QueryDataFromRequest;
use crate::Localizations;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::*;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::OriginalUri;
use axum::response::{self, IntoResponse};
use axum::routing::get;
use axum::{Extension, Router};
use futures_util::stream::StreamExt;
use i18n_embed::fluent::fluent_language_loader;
use i18n_embed::LanguageLoader;
use intercode_entities::cms_parent::CmsParentTrait;
use intercode_entities::events;
use intercode_graphql::cms_rendering_context::CmsRenderingContext;
use intercode_graphql::loaders::LoaderManager;
use intercode_graphql::{api, LiquidRenderer, SchemaData};
use liquid::object;
use regex::Regex;
use sea_orm::{ColumnTrait, DatabaseConnection, ModelTrait, QueryFilter};
use std::env;
use std::future::ready;
use std::io::BufReader;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tls_listener::TlsListener;
use tokio_rustls::TlsAcceptor;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::compression::CompressionLayer;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::log::*;

#[derive(Debug)]
struct FatalDatabaseError {
  #[allow(dead_code)]
  db_err: sea_orm::DbErr,
}

type IntercodeSchema = Schema<api::QueryRoot, EmptyMutation, EmptySubscription>;

async fn single_page_app_entry(
  OriginalUri(url): OriginalUri,
  schema_data: Extension<SchemaData>,
  QueryDataFromRequest(query_data): QueryDataFromRequest,
) -> Result<impl IntoResponse, ::http::StatusCode> {
  let event_path_regex: regex::Regex =
    Regex::new("^/events/(\\d+)").map_err(|_| ::http::StatusCode::INTERNAL_SERVER_ERROR)?;
  let db = &schema_data.db;
  let path = url.path();
  let page_scope = query_data.cms_parent.cms_page_for_path(path);

  let page = if let Some(page_scope) = page_scope {
    page_scope
      .one(db.as_ref())
      .await
      .map_err(|_db_err| ::http::StatusCode::INTERNAL_SERVER_ERROR)?
  } else {
    None
  };

  let event = if let Some(convention) = query_data.convention.as_ref() {
    if convention.site_mode == "single_event" {
      convention
        .find_related(events::Entity)
        .one(db.as_ref())
        .await
        .map_err(|_db_err| ::http::StatusCode::INTERNAL_SERVER_ERROR)?
    } else if let Some(event_captures) = event_path_regex.captures(path) {
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

  let liquid_renderer = IntercodeLiquidRenderer::new(&query_data, &schema_data);

  let cms_rendering_context = CmsRenderingContext::new(
    object!({}),
    &schema_data,
    &query_data,
    Arc::new(liquid_renderer),
  );
  let page_title = "TODO";

  Ok(response::Html(
    cms_rendering_context
      .render_app_root_content(&url, page_title, page.as_ref(), event.as_ref())
      .await,
  ))
}

async fn graphql_handler(
  schema: Extension<IntercodeSchema>,
  schema_data: Extension<SchemaData>,
  QueryDataFromRequest(query_data): QueryDataFromRequest,
  req: GraphQLRequest,
) -> GraphQLResponse {
  let liquid_renderer = IntercodeLiquidRenderer::new(&query_data, &schema_data);
  let req = req
    .into_inner()
    .data(query_data)
    .data::<Arc<dyn LiquidRenderer>>(Arc::new(liquid_renderer));

  schema.execute(req).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
  response::Html(playground_source(
    GraphQLPlaygroundConfig::new("/graphql").with_setting("schema.polling.interval", 10000),
  ))
}

pub async fn serve(db: DatabaseConnection) -> Result<()> {
  let db_arc = Arc::new(db);

  let language_loader = fluent_language_loader!();
  language_loader.load_languages(&Localizations, &[language_loader.fallback_language()])?;
  let language_loader_arc = Arc::new(language_loader);

  let schema_data = SchemaData {
    db: Arc::clone(&db_arc),
    language_loader: Arc::clone(&language_loader_arc),
    loaders: LoaderManager::new(&db_arc),
  };

  let graphql_schema =
    async_graphql::Schema::build(api::QueryRoot, EmptyMutation, EmptySubscription)
      .extension(async_graphql::extensions::Tracing)
      .data(schema_data.clone())
      .finish();

  let app = Router::new()
    .route("/graphql", get(graphql_playground).post(graphql_handler))
    .fallback(get(single_page_app_entry))
    .layer(Extension(schema_data))
    .layer(Extension(graphql_schema));

  let service = tower::ServiceBuilder::new()
    .concurrency_limit(
      env::var("MAX_CONCURRENCY")
        .unwrap_or_else(|_| "25".to_string())
        .parse()
        .unwrap_or(25),
    )
    .layer(CompressionLayer::new())
    .layer(
      TraceLayer::new_for_http().on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
    )
    .layer(CatchPanicLayer::new())
    .service(app);

  let addr = SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    env::var("PORT")
      .unwrap_or_else(|_| String::from("5901"))
      .parse()?,
  );
  let signal = async move {
    tokio::signal::ctrl_c()
      .await
      .expect("failed to listen to shutdown signal");
  };

  if let (Ok(cert_path), Ok(key_path)) = (env::var("TLS_CERT_PATH"), env::var("TLS_KEY_PATH")) {
    let cert_pem = std::fs::File::open(cert_path)?;
    let key_pem = std::fs::File::open(key_path)?;
    let mut cert_reader = BufReader::new(cert_pem);
    let mut key_reader = BufReader::new(key_pem);

    let key = tokio_rustls::rustls::PrivateKey(
      rustls_pemfile::rsa_private_keys(&mut key_reader)?[0].to_owned(),
    );
    let certs = rustls_pemfile::certs(&mut cert_reader)?
      .iter()
      .map(|der| tokio_rustls::rustls::Certificate(der.to_owned()))
      .collect();

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let acceptor: TlsAcceptor = Arc::new(
      tokio_rustls::rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap(),
    )
    .into();

    let incoming = TlsListener::new(acceptor, listener).filter(|conn| {
      if let Err(err) = conn {
        error!("Error: {:?}", err);
        ready(false)
      } else {
        ready(true)
      }
    });

    hyper::Server::builder(hyper::server::accept::from_stream(incoming))
      .serve(tower::make::Shared::new(service))
      .with_graceful_shutdown(signal)
      .await?;
  } else {
    warn!(
      "TLS_CERT_PATH and/or TLS_KEY_PATH not present in env.  Falling back to unencrypted HTTP."
    );
    let listener = std::net::TcpListener::bind(addr)?;
    hyper::Server::from_tcp(listener)?
      .serve(tower::make::Shared::new(service))
      .with_graceful_shutdown(signal)
      .await?;
  };

  Ok(())
}
