use std::{
  fmt::{Debug, Display},
  sync::Arc,
};

use tokio::sync::SetError;

#[derive(Debug)]
pub enum DropError {
  GraphQLError(async_graphql::Error),
  LiquidError(liquid::Error),
  DbErr(sea_orm::DbErr),
  TokioSyncSetError(String),
}

impl Display for DropError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DropError::GraphQLError(err) => err.fmt(f),
      DropError::LiquidError(err) => std::fmt::Display::fmt(&err, f),
      DropError::DbErr(err) => std::fmt::Display::fmt(&err, f),
      DropError::TokioSyncSetError(err) => std::fmt::Display::fmt(&err, f),
    }
  }
}

impl From<async_graphql::Error> for DropError {
  fn from(err: async_graphql::Error) -> Self {
    DropError::GraphQLError(err)
  }
}

impl From<liquid::Error> for DropError {
  fn from(err: liquid::Error) -> Self {
    DropError::LiquidError(err)
  }
}

impl From<sea_orm::DbErr> for DropError {
  fn from(err: sea_orm::DbErr) -> Self {
    DropError::DbErr(err)
  }
}

impl From<Arc<sea_orm::DbErr>> for DropError {
  fn from(err: Arc<sea_orm::DbErr>) -> Self {
    DropError::DbErr(err.as_ref().clone())
  }
}

impl<T> From<SetError<T>> for DropError {
  fn from(err: SetError<T>) -> Self {
    DropError::TokioSyncSetError(err.to_string())
  }
}
