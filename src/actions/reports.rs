use axum::{debug_handler, extract::Path, response::IntoResponse};

use crate::middleware::QueryDataFromRequest;

#[debug_handler]
pub async fn single_user_printable(
  QueryDataFromRequest(query_data): QueryDataFromRequest,
  Path(user_con_profile_id): Path<i64>,
) -> Result<impl IntoResponse, http::StatusCode> {
  intercode_reporting::actions::single_user_printable(query_data, user_con_profile_id).await
}
