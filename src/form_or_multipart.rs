use std::error::Error;

use axum::{
  async_trait,
  body::{Bytes, HttpBody},
  extract::{FromRequest, Multipart},
  Form,
};
use http::{Request, StatusCode};
use once_cell::sync::Lazy;
use regex::bytes::Regex;

static CONTENT_TYPE_MIME_TYPE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("^([^;]+)").unwrap());

pub enum FormOrMultipart<T> {
  Form(Form<T>),
  Multipart(Multipart),
}

#[async_trait]
impl<T, S: Send + Sync, B: Send + Sync + 'static> FromRequest<S, B> for FormOrMultipart<T>
where
  Form<T>: FromRequest<S, B>,
  B: HttpBody,
  Bytes: From<<B as HttpBody>::Data>,
  <B as HttpBody>::Error: Error + Send + Sync,
{
  type Rejection = StatusCode;

  async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
    if let Some(content_type) = req.headers().get("content-type") {
      let content_type = content_type
        .to_str()
        .map_err(|_err| StatusCode::NOT_ACCEPTABLE)?;
      let mime_type = CONTENT_TYPE_MIME_TYPE_REGEX
        .find(content_type.as_bytes())
        .map(|mime_type| std::str::from_utf8(mime_type.as_bytes()));

      match mime_type {
        Some(Ok(v)) if v == "application/x-www-form-urlencoded" => {
          Form::<T>::from_request(req, state)
            .await
            .map(Self::Form)
            .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)
        }
        Some(Ok(v)) if v == "multipart/form-data" => Multipart::from_request(req, state)
          .await
          .map(Self::Multipart)
          .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR),
        _ => Err(StatusCode::NOT_ACCEPTABLE),
      }
    } else {
      Err(StatusCode::NOT_ACCEPTABLE)
    }
  }
}
