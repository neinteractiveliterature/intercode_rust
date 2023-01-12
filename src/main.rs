#![recursion_limit = "256"]

extern crate chrono;
extern crate chrono_tz;
extern crate dotenv;
extern crate tracing;

mod check_liquid;
mod csrf;
mod db_sessions;
mod drops;
mod legacy_passwords;
mod liquid_renderer;
mod middleware;
mod server;

use async_graphql::*;
use check_liquid::check_liquid;
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
use tokio::runtime::Runtime;
use tonic::metadata::{AsciiMetadataValue, MetadataMap};
use tonic::transport::ClientTlsConfig;
use tracing::{log::*, Subscriber};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::Filter;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

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
  info!("Installing flamegraph subscriber; will output to tracing.folded");
  let (flame_layer, _guard) = tracing_flame::FlameLayer::with_file("./tracing.folded").unwrap();

  (subscriber.with(flame_layer), _guard)
}

#[cfg(not(feature = "flamegraph"))]
fn setup_flamegraph_subscriber<S: Subscriber>(subscriber: S) -> (S, impl Drop) {
  (subscriber, Box::new("shim droppable value"))
}

struct RequestFilter;
impl<S> Filter<S> for RequestFilter {
  fn enabled(
    &self,
    _meta: &tracing::Metadata<'_>,
    _ctx: &tracing_subscriber::layer::Context<'_, S>,
  ) -> bool {
    true
    // meta.
    // meta.name() == "request"
  }
}

fn setup_otlp<S: Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>>(
  _subscriber: &S,
  endpoint: String,
) -> Result<OpenTelemetryLayer<S, Tracer>> {
  let mut map = MetadataMap::with_capacity(1);
  if let Ok(value) = env::var("HONEYCOMB_API_KEY") {
    map.insert(
      "x-honeycomb-team",
      AsciiMetadataValue::try_from(value.as_bytes())?,
    );
  }

  let use_tls = endpoint.starts_with("https");
  let exporter = opentelemetry_otlp::new_exporter()
    .tonic()
    .with_endpoint(endpoint)
    .with_metadata(map);
  let exporter = if use_tls {
    exporter.with_tls_config(ClientTlsConfig::new())
  } else {
    exporter
  };

  let tracer = opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(exporter)
    .with_trace_config(config().with_resource(Resource::new(vec![KeyValue::new(
      "service.name",
      env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "intercode_rust".to_string()),
    )])))
    .install_batch(opentelemetry::runtime::Tokio)?;

  Ok(
    tracing_opentelemetry::layer().with_tracer(tracer), // .with_filter(RequestFilter),
  )
}

async fn run() -> Result<()> {
  setup_tracing(EnvFilter::from_default_env());

  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
  info!("Connecting: {}", database_url);

  let db = connect_database().await?;

  serve(db).await
}

fn setup_tracing(env_filter: EnvFilter) {
  let fmt_layer = tracing_subscriber::fmt::layer()
    .with_test_writer()
    .with_filter(env_filter);
  let subscriber = tracing_subscriber::registry().with(fmt_layer);
  let (subscriber, _guard) = setup_flamegraph_subscriber(subscriber);
  let otlp_layer = env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
    .map_err(Into::into)
    .and_then(|endpoint| setup_otlp(&subscriber, endpoint))
    .ok();
  let subscriber = subscriber.with(otlp_layer);
  #[cfg(feature = "tokio-console")]
  let subscriber = subscriber.with(console_subscriber::spawn());
  subscriber.init();
}

#[derive(Parser, Debug)]
enum Subcommands {
  Serve,
  CheckLiquid,
  ExportSchema,
}

fn build_runtime() -> Runtime {
  let mut builder = tokio::runtime::Builder::new_multi_thread();
  builder.enable_all();

  if let Ok(worker_threads) = env::var("WORKER_THREADS") {
    builder.worker_threads(
      worker_threads
        .parse()
        .expect("WORKER_THREADS must be a number if set"),
    );
  }

  builder.build().unwrap()
}

fn main() -> Result<()> {
  dotenv().ok();

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
      // just build the schema without any data or extras
      let schema =
        async_graphql::Schema::build(api::QueryRoot, EmptyMutation, EmptySubscription).finish();

      println!("{}", schema.sdl());
    }
    Subcommands::Serve => {
      build_runtime().block_on(run())?;
    }
    Subcommands::CheckLiquid => {
      build_runtime().block_on(check_liquid())?;
    }
  }

  Ok(())
}
