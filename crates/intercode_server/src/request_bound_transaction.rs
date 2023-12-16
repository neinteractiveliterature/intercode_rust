use std::sync::Arc;

use axum::{
  extract::{Request, State},
  middleware::Next,
  response::{IntoResponse, Response},
};
use http::StatusCode;
use sea_orm::{DatabaseConnection, DbErr, TransactionTrait};
use seawater::ConnectionWrapper;

async fn run_request_bound_transaction(
  mut request: Request,
  next: Next,
  db: &DatabaseConnection,
) -> Result<Response, DbErr> {
  let extensions_mut = request.extensions_mut();
  let tx = db.begin().await?;
  let tx_arc = Arc::new(tx);

  extensions_mut.insert(ConnectionWrapper::DatabaseTransaction(Arc::downgrade(
    &tx_arc,
  )));
  let response = next.run(request).await;
  let tx = Arc::try_unwrap(tx_arc).map_err(|arc| {
    DbErr::Custom(format!(
      "Cannot finish database transaction because it still has {} strong references",
      Arc::strong_count(&arc)
    ))
  })?;

  if response.status().is_success() {
    tx.commit().await?;
  } else {
    tx.rollback().await?;
  }

  Ok(response)
}

pub async fn request_bound_transaction(
  State(db_conn): State<Arc<DatabaseConnection>>,
  request: Request,
  next: Next,
) -> Response {
  run_request_bound_transaction(request, next, &db_conn)
    .await
    .unwrap_or_else(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())
}
