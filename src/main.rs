extern crate chrono;
extern crate chrono_tz;
extern crate dotenv;
extern crate tracing;

mod entities;
pub mod entity_relay_connection;
use async_graphql::dataloader::DataLoader;
use cms_parent::CmsParent;
pub use entities::*;
pub mod api;
pub mod cms_parent;
pub mod inflections;
pub mod liquid_extensions;
pub mod loaders;
pub mod model_ext;
pub mod timespan;

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::*;
use async_graphql_warp::GraphQLResponse;
use dotenv::dotenv;
use i18n_embed::fluent::{fluent_language_loader, FluentLanguageLoader};
use i18n_embed::LanguageLoader;
use loaders::{EntityIdLoader, ToEntityIdLoader};
use rust_embed::RustEmbed;
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

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[derive(RustEmbed)]
#[folder = "i18n"] // path to the compiled localization resources
pub struct Localizations;

pub struct SchemaData {
  pub db: Arc<DatabaseConnection>,
  pub convention_id_loader:
    DataLoader<EntityIdLoader<conventions::Entity, conventions::PrimaryKey>>,
  pub language_loader: Arc<FluentLanguageLoader>,
  pub user_id_loader: DataLoader<EntityIdLoader<users::Entity, users::PrimaryKey>>,
}

impl Clone for SchemaData {
  fn clone(&self) -> Self {
    let convention_id_loader = conventions::Entity.to_entity_id_loader(self.db.clone());
    let user_id_loader = users::Entity.to_entity_id_loader(self.db.clone());

    SchemaData {
      db: self.db.clone(),
      language_loader: self.language_loader.clone(),
      convention_id_loader: DataLoader::new(convention_id_loader, tokio::spawn),
      user_id_loader: DataLoader::new(user_id_loader, tokio::spawn),
    }
  }
}

impl std::fmt::Debug for SchemaData {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SchemaData")
      .field("db", &self.db)
      .field(
        "convention_id_loader.loader()",
        &self.convention_id_loader.loader(),
      )
      .finish()
  }
}

#[derive(Debug, Clone)]
pub struct QueryData {
  pub cms_parent: Option<CmsParent>,
  pub current_user: Option<users::Model>,
  pub convention: Option<conventions::Model>,
}

impl Default for QueryData {
  fn default() -> Self {
    Self {
      cms_parent: None,
      current_user: None,
      convention: None,
    }
  }
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
  let db_arc = Arc::new(db);

  let log = warp::log("intercode_rust::http");

  let convention_id_loader = conventions::Entity.to_entity_id_loader(Arc::clone(&db_arc));
  let language_loader = fluent_language_loader!();
  language_loader.load_languages(&Localizations, &[language_loader.fallback_language()])?;
  let language_loader_arc = Arc::new(language_loader);
  let user_id_loader = users::Entity.to_entity_id_loader(Arc::clone(&db_arc));

  let graphql_schema =
    async_graphql::Schema::build(api::QueryRoot, EmptyMutation, EmptySubscription)
      .extension(async_graphql::extensions::Tracing)
      .data(SchemaData {
        db: Arc::clone(&db_arc),
        convention_id_loader: DataLoader::new(convention_id_loader, tokio::spawn),
        language_loader: language_loader_arc,
        user_id_loader: DataLoader::new(user_id_loader, tokio::spawn),
      })
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

          let convention = match authority {
            Some(authority) => entities::conventions::Entity::find()
              .filter(entities::conventions::Column::Domain.eq(authority.host()))
              .one(db.as_ref())
              .await
              .unwrap_or_else(|error| {
                warn!("Error while querying for convention: {}", error);
                None
              }),
            None => None,
          };

          let request = request.data(QueryData {
            convention: convention.clone(),
            current_user: None,
            cms_parent: convention.and_then(|c| Some(CmsParent::Convention(c))),
          });

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
  let addr = SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    env::var("PORT").unwrap_or(String::from("5901")).parse()?,
  );
  let signal = async move {
    tokio::signal::ctrl_c()
      .await
      .expect("failed to listen to shutdown signal");
  };

  let server = warp::serve(routes);
  if let Ok(cert_path) = env::var("TLS_CERT_PATH") {
    if let Ok(key_path) = env::var("TLS_KEY_PATH") {
      let (_addr, fut) = server
        .tls()
        .cert_path(cert_path)
        .key_path(key_path)
        .bind_with_graceful_shutdown(addr, signal);
      fut.await;
    } else {
      let (_addr, fut) = server.bind_with_graceful_shutdown(addr, signal);
      fut.await;
    }
  } else {
    let (_addr, fut) = server.bind_with_graceful_shutdown(addr, signal);
    fut.await;
  };

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
  #[cfg(feature = "dhat-heap")]
  let _profiler = dhat::Profiler::new_heap();

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
  rt.block_on(run())?;

  Ok(())
}
