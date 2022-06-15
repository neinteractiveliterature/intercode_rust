extern crate chrono;
extern crate chrono_tz;
extern crate dotenv;
extern crate tracing;

mod entities;
pub mod entity_relay_connection;
use cms_parent::CmsParent;
pub use entities::*;
pub mod api;
pub mod cms_parent;
pub mod inflections;
pub mod liquid_extensions;
pub mod loaders;
pub mod model_ext;
mod server;
pub mod timespan;

use async_graphql::*;
use dotenv::dotenv;
use i18n_embed::fluent::FluentLanguageLoader;
use loaders::LoaderManager;
use rust_embed::RustEmbed;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use server::serve;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tracing::log::*;
use tracing_subscriber::EnvFilter;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[derive(RustEmbed)]
#[folder = "i18n"] // path to the compiled localization resources
pub struct Localizations;

#[derive(Debug, Clone)]
pub struct SchemaData {
  pub db: Arc<DatabaseConnection>,
  pub language_loader: Arc<FluentLanguageLoader>,
  pub loaders: LoaderManager,
}

#[derive(Debug, Clone, Default)]
pub struct QueryData {
  pub cms_parent: Option<CmsParent>,
  pub current_user: Option<users::Model>,
  pub convention: Option<conventions::Model>,
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
