use std::error::Error;

use axum::{
  async_trait,
  body::{Bytes, HttpBody},
  extract::{FromRequest, Multipart},
  Form,
};
use http::{Request, StatusCode};

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
    match req.headers().get("content-type") {
      Some(v) if v == "application/x-www-form-urlencoded" => Form::<T>::from_request(req, state)
        .await
        .map(Self::Form)
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR),
      Some(v) if v == "multipart/form-data" => Multipart::from_request(req, state)
        .await
        .map(Self::Multipart)
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR),
      _ => Err(StatusCode::NOT_ACCEPTABLE),
    }
  }
}
