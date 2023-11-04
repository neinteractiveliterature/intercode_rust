#![recursion_limit = "256"]

extern crate chrono;
extern crate chrono_tz;
extern crate dotenv;
extern crate tracing;

mod actions;
mod database;
mod liquid_renderer;
mod server;

use async_graphql::*;
use clap::{command, FromArgMatches, Parser, Subcommand};
use database::connect_database;
use dotenv::dotenv;
use indicatif::ProgressBar;
use intercode_graphql::{build_intercode_graphql_schema, build_intercode_graphql_schema_minimal};
use intercode_graphql_core::schema_data::SchemaData;
use intercode_liquid_drops::check_liquid::LiquidChecker;
use intercode_server::i18n::build_language_loader;
use intercode_server::serve;
use opentelemetry::sdk::trace::{config, Tracer};
use opentelemetry::sdk::Resource;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use std::env;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tonic::metadata::{AsciiMetadataValue, MetadataMap};
use tonic::transport::ClientTlsConfig;
use tracing::{log::*, Subscriber};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::Filter;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

use crate::server::bootstrap_app;

// #[cfg(not(target_env = "msvc"))]
// use tikv_jemallocator::Jemalloc;

// #[cfg(not(target_env = "msvc"))]
// #[global_allocator]
// static GLOBAL: Jemalloc = Jemalloc;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

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

  let app = bootstrap_app().await?;
  serve(app).await
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
      let schema = build_intercode_graphql_schema_minimal().finish();

      println!("{}", schema.sdl());
    }
    Subcommands::Serve => {
      build_runtime().block_on(run())?;
    }
    Subcommands::CheckLiquid => {
      build_runtime().block_on(async {
        setup_tracing(EnvFilter::new("error"));

        let startup_bar = ProgressBar::new_spinner();

        startup_bar.set_message("Connecting to database...");
        let db = connect_database().await?;

        let schema_data = SchemaData {
          language_loader: Arc::new(build_language_loader()?),
        };
        let checker = LiquidChecker::new(
          build_intercode_graphql_schema(schema_data.clone()),
          Arc::new(db),
          schema_data,
        );
        checker.check_liquid(startup_bar).await
      })?;
    }
  }

  Ok(())
}
