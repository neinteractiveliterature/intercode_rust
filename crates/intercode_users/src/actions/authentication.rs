use std::collections::HashMap;

use axum::{
  debug_handler,
  response::{self, IntoResponse},
  Extension, Form,
};
use axum_sessions::SessionHandle;
use http::StatusCode;
use intercode_entities::users;
use intercode_server::{enforce_csrf, CsrfData, FormOrMultipart, QueryDataFromRequest};
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use serde_json::json;

use crate::legacy_passwords::{verify_legacy_md5_password, verify_legacy_sha1_password};

pub const BCRYPT_COST: u32 = 10;

#[derive(Deserialize, Debug)]
pub struct SignInParams {
  #[serde(rename(deserialize = "user[email]"))]
  email: String,
  #[serde(rename(deserialize = "user[password]"))]
  password: String,
}

#[debug_handler]
pub async fn sign_in(
  token: CsrfData,
  QueryDataFromRequest(query_data): QueryDataFromRequest,
  session: Extension<SessionHandle>,
  form_or_multipart: FormOrMultipart<SignInParams>,
) -> Result<impl IntoResponse, StatusCode> {
  enforce_csrf(token)?;

  let params = match form_or_multipart {
    FormOrMultipart::Form(form) => form,
    FormOrMultipart::Multipart(mut multipart) => {
      let mut mp_params: HashMap<String, String> = Default::default();

      while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let value = field.text().await.unwrap();
        mp_params.insert(name, value);
      }

      let mp_value = serde_json::to_value(mp_params).unwrap();
      Form(serde_json::from_value::<SignInParams>(mp_value).unwrap())
    }
  };

  let user = users::Entity::find()
    .filter(users::Column::Email.eq(params.email.as_str()))
    .one(query_data.db().as_ref())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

  let password_matches = if !user.encrypted_password.is_empty() {
    bcrypt::verify(&params.password, &user.encrypted_password)
      .map_err(|_| StatusCode::NOT_ACCEPTABLE)?
  } else if let (Some(legacy_password_sha1), Some(legacy_password_sha1_salt)) =
    (user.legacy_password_sha1, user.legacy_password_sha1_salt)
  {
    verify_legacy_sha1_password(
      &params.password,
      &legacy_password_sha1,
      &legacy_password_sha1_salt,
    )
  } else if let Some(legacy_password_md5) = user.legacy_password_md5 {
    verify_legacy_md5_password(&params.password, &legacy_password_md5)
  } else {
    false
  };

  if !password_matches {
    return Err(StatusCode::NOT_ACCEPTABLE);
  }

  if user.encrypted_password.is_empty() {
    // upgrade the password while we have it in RAM
    let upgrade = users::ActiveModel {
      encrypted_password: ActiveValue::Set(bcrypt::hash(&params.password, BCRYPT_COST).unwrap()),
      legacy_password_md5: ActiveValue::Set(None),
      legacy_password_sha1: ActiveValue::Set(None),
      legacy_password_sha1_salt: ActiveValue::Set(None),
      ..Default::default()
    };

    users::Entity::update(upgrade)
      .exec(query_data.db().as_ref())
      .await
      .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  }

  let mut write_guard = session.write().await;
  write_guard
    .insert("current_user_id", user.id)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  Ok(response::Json(json!({ "status": "success" })))
}
