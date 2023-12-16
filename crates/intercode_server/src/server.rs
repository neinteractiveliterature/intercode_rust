use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use async_graphql::Result;
use axum_server::service::MakeService;
use axum_server::tls_rustls::{RustlsAcceptor, RustlsConfig};
use axum_server::{Handle, Server};
use http::Request;
use hyper::body::Incoming;
use opentelemetry::global::shutdown_tracer_provider;
use tokio::time::sleep;
use tracing::log::*;

#[derive(Debug)]
struct FatalDatabaseError {
  #[allow(dead_code)]
  db_err: sea_orm::DbErr,
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

pub async fn serve<M: MakeService<SocketAddr, Request<Incoming>>>(app: M) -> Result<()> {
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
