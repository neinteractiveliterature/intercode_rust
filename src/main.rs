extern crate chrono;
extern crate chrono_tz;
extern crate dotenv;
extern crate tracing;

use clap::{command, FromArgMatches, Parser, Subcommand};
mod filters;
mod server;

use async_graphql::*;
use dotenv::dotenv;
use intercode_graphql::api;
use rust_embed::RustEmbed;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use server::serve;
use std::env;
use std::time::Duration;
use tracing::log::*;
use tracing_subscriber::EnvFilter;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[derive(RustEmbed)]
#[folder = "i18n"] // path to the compiled localization resources
pub struct Localizations;

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

#[derive(Parser, Debug)]
enum Subcommands {
  Serve,
  ExportSchema,
}

fn main() -> Result<()> {
  #[cfg(feature = "dhat-heap")]
  let _profiler = dhat::Profiler::new_heap();

  let cli = command!()
    .about("A one-stop web application for conventions")
    .infer_subcommands(true);
  // Augment with derived subcommands
  let cli = Subcommands::augment_subcommands(cli);

  let matches = cli.get_matches();
  let derived_subcommands = Subcommands::from_arg_matches(&matches)
    .map_err(|err| err.exit())
    .unwrap_or(Subcommands::Serve);

  match derived_subcommands {
    Subcommands::ExportSchema => {
      let schema =
        async_graphql::Schema::build(api::QueryRoot, EmptyMutation, EmptySubscription).finish();

      println!("{}", schema.sdl());
    }
    Subcommands::Serve => {
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
    }
  }

  Ok(())
}
