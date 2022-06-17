use crate::Localizations;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::*;
use async_graphql_warp::GraphQLResponse;
use futures_util::stream::StreamExt;
use i18n_embed::fluent::fluent_language_loader;
use i18n_embed::LanguageLoader;
use intercode_entities::cms_parent::CmsParent;
use intercode_entities::conventions;
use intercode_graphql::loaders::LoaderManager;
use intercode_graphql::{api, QueryData, SchemaData};
use sea_orm::DatabaseConnection;
use std::convert::Infallible;
use std::env;
use std::future::ready;
use std::io::BufReader;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tls_listener::TlsListener;
use tokio_rustls::TlsAcceptor;
use tower_http::compression::CompressionLayer;
use tracing::log::*;
use warp::http::Response as HttpResponse;
use warp::Filter;

pub async fn serve(db: DatabaseConnection) -> Result<()> {
  let db_arc = Arc::new(db);

  let log = warp::log("intercode_rust::http");

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

  let graphql_post = warp::path("graphql")
    .and(warp::post())
    .and(warp::host::optional())
    .and(async_graphql_warp::graphql(graphql_schema))
    .and_then(
      move |authority: Option<warp::host::Authority>,
            (schema, request): (
        Schema<api::QueryRoot, EmptyMutation, EmptySubscription>,
        async_graphql::Request,
      )| {
        let db = Arc::clone(&db_arc);

        async move {
          use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

          let convention = Arc::new(match authority {
            Some(authority) => conventions::Entity::find()
              .filter(conventions::Column::Domain.eq(authority.host()))
              .one(db.as_ref())
              .await
              .unwrap_or_else(|error| {
                warn!("Error while querying for convention: {}", error);
                None
              }),
            None => None,
          });
          let cms_parent: Arc<Option<CmsParent>> =
            Arc::new(convention.as_ref().as_ref().map(|c| c.clone().into()));

          let query_data = QueryData::new(cms_parent, Arc::new(None), convention);
          let request = request.data(query_data);

          Ok::<_, Infallible>(GraphQLResponse::from(schema.execute(request).await))
        }
      },
    );

  let graphql_playground = warp::path::end().and(warp::get()).map(|| {
    HttpResponse::builder()
      .header("content-type", "text/html")
      .body(playground_source(
        GraphQLPlaygroundConfig::new("/graphql").with_setting("schema.polling.interval", 10000),
      ))
  });

  let routes = graphql_playground.or(graphql_post).with(log);
  let warp_service = warp::service(routes);

  let service = tower::ServiceBuilder::new()
    .concurrency_limit(
      env::var("MAX_CONCURRENCY")
        .unwrap_or_else(|_| "25".to_string())
        .parse()
        .unwrap_or(25),
    )
    .layer(CompressionLayer::new())
    .service(warp_service);

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
    let listener = std::net::TcpListener::bind(addr)?;
    hyper::Server::from_tcp(listener)?
      .serve(tower::make::Shared::new(service))
      .with_graceful_shutdown(signal)
      .await?;
  };

  Ok(())
}
