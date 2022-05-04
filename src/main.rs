extern crate dotenv;
extern crate tracing;

mod entities;
use async_graphql::dataloader::DataLoader;
pub use entities::*;
pub mod api;
pub mod loaders;

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::*;
use async_graphql_warp::GraphQLResponse;
use dotenv::dotenv;
use loaders::EntityIdLoader;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::convert::Infallible;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tracing::log::*;
use tracing_subscriber::EnvFilter;
use warp::http::Response as HttpResponse;
use warp::Filter;

pub struct SchemaData {
  pub db: Arc<DatabaseConnection>,
  pub convention_id_loader:
    DataLoader<EntityIdLoader<conventions::Entity, conventions::PrimaryKey>>,
}

async fn connect_database() -> Result<DatabaseConnection, DbErr> {
  dotenv().ok();
  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

  let mut connect_options = ConnectOptions::new(database_url);
  if let Ok(max_connections) = env::var("DB_MAX_CONNECTIONS") {
    connect_options.max_connections(
      max_connections
        .parse()
        .expect("DB_MAX_CONNECTIONS must be a number if set"),
    );
  }
  if let Ok(idle_timeout) = env::var("DB_IDLE_TIMEOUT") {
    connect_options.idle_timeout(Duration::new(
      idle_timeout
        .parse()
        .expect("DB_IDLE_TIMEOUT must be a number if set"),
      0,
    ));
  }
  info!("Connecting: {:#?}", connect_options);

  Database::connect(connect_options).await
}

async fn serve(db: DatabaseConnection) -> Result<()> {
  use crate::loaders::ToEntityIdLoader;
  let db_arc = Arc::new(db);

  let log = warp::log("intercode_rust::http");

  let convention_id_loader = conventions::Entity.to_entity_id_loader(Arc::clone(&db_arc));

  let graphql_schema =
    async_graphql::Schema::build(api::QueryRoot, EmptyMutation, EmptySubscription)
      .extension(async_graphql::extensions::Tracing)
      .data(SchemaData {
        db: Arc::clone(&db_arc),
        convention_id_loader: DataLoader::new(convention_id_loader, tokio::spawn),
      })
      .finish();

  let hi = warp::path("hello")
    .and(warp::path::param())
    .and(warp::get())
    .and(warp::header("user-agent"))
    .map(|param: String, agent: String| format!("Hello {}, whose agent is {}", param, agent));

  let graphql_post = warp::path("graphql")
    .and(warp::post())
    .and(async_graphql_warp::graphql(graphql_schema))
    .and_then(
      |(schema, request): (
        Schema<api::QueryRoot, EmptyMutation, EmptySubscription>,
        async_graphql::Request,
      )| async move { Ok::<_, Infallible>(GraphQLResponse::from(schema.execute(request).await)) },
    );

  let graphql_playground = warp::path::end().and(warp::get()).map(|| {
    HttpResponse::builder()
      .header("content-type", "text/html")
      .body(playground_source(
        GraphQLPlaygroundConfig::new("/graphql").with_setting("schema.polling.interval", 10000),
      ))
  });

  let routes = hi.or(graphql_playground).or(graphql_post).with(log);

  warp::serve(routes)
    .run(SocketAddr::new(
      IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
      5901,
    ))
    .await;

  Ok(())
}

async fn run() -> Result<()> {
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .with_test_writer()
    .init();

  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
  info!("Connecting: {}", database_url);

  let db = connect_database().await?;

  serve(db).await
}

fn main() -> Result<()> {
  dotenv().ok();

  let mut builder = tokio::runtime::Builder::new_multi_thread();
  builder.enable_all();

  if let Ok(worker_threads) = env::var("WORKER_THREADS") {
    builder.worker_threads(
      worker_threads
        .parse()
        .expect("WORKER_THREADS must be a number if set"),
    );
  }

  let rt = builder.build().unwrap();

  rt.block_on(run())
}
