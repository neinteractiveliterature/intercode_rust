extern crate chrono;
extern crate chrono_tz;
extern crate dotenv;
extern crate tracing;

mod drops;
mod liquid_renderer;
mod middleware;
mod server;

use async_graphql::*;
use clap::{command, FromArgMatches, Parser, Subcommand};
use dotenv::dotenv;
use intercode_graphql::api;
use opentelemetry::sdk::trace::{config, Tracer};
use opentelemetry::sdk::Resource;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use rust_embed::RustEmbed;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use server::serve;
use std::env;
use std::time::Duration;
use tonic::metadata::{AsciiMetadataValue, MetadataMap};
use tonic::transport::ClientTlsConfig;
use tracing::{log::*, Subscriber};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::Layered;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
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

#[cfg(feature = "flamegraph")]
fn setup_flamegraph_subscriber<
  S: Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
>(
  subscriber: S,
) -> (
  tracing_subscriber::layer::Layered<
    tracing_flame::FlameLayer<S, std::io::BufWriter<std::fs::File>>,
    S,
    S,
  >,
  impl Drop,
) {
  eprintln!("Installing flamegraph subscriber; will output to tracing.folded");
  let (flame_layer, _guard) = tracing_flame::FlameLayer::with_file("./tracing.folded").unwrap();

  (subscriber.with(flame_layer), _guard)
}

#[cfg(not(feature = "flamegraph"))]
fn setup_flamegraph_subscriber<S: Subscriber>(subscriber: S) -> (S, impl Drop) {
  (subscriber, Box::new("shim droppable value"))
}

fn setup_honeycomb_tracing<
  S: Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
>(
  subscriber: S,
) -> Result<Layered<OpenTelemetryLayer<S, Tracer>, S>> {
  let mut map = MetadataMap::with_capacity(1);
  map.insert(
    "x-honeycomb-team",
    AsciiMetadataValue::try_from_bytes(env::var("HONEYCOMB_API_KEY")?.as_bytes())?,
  );

  let tracer = opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(
      opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(env::var("OTEL_EXPORTER_OTLP_ENDPOINT")?)
        .with_tls_config(ClientTlsConfig::new())
        .with_metadata(map),
    )
    .with_trace_config(config().with_resource(Resource::new(vec![KeyValue::new(
      "service.name",
      env::var("OTEL_SERVICE_NAME")?,
    )])))
    .install_batch(opentelemetry::runtime::Tokio)?;
  let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

  Ok(subscriber.with(telemetry))
}

async fn run() -> Result<()> {
  let subscriber = tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .with_test_writer()
    .finish();

  let (subscriber, _guard) = setup_flamegraph_subscriber(subscriber);

  if env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok() {
    setup_honeycomb_tracing(subscriber)?.init();
  } else {
    subscriber.init();
  }

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
