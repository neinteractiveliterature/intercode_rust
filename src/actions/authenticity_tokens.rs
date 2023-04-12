use axum::response::{self, IntoResponse};
use std::collections::HashMap;

use crate::csrf::CsrfData;

pub async fn authenticity_tokens(token: CsrfData) -> impl IntoResponse {
  let value = token.authenticity_token();
  let response = vec![
    "graphql",
    "changePassword",
    "denyAuthorization",
    "grantAuthorization",
    "railsDirectUploads",
    "resetPassword",
    "signIn",
    "signOut",
    "signUp",
    "updateUser",
  ]
  .into_iter()
  .map(|field| (field, value.clone()))
  .collect::<HashMap<_, _>>();
  response::Json(response)
}
