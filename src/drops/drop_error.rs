use std::{fmt::Debug, sync::Arc};

use sea_orm::strum::Display;

#[derive(Debug, Display)]
pub enum DropError {
  GraphQLError(async_graphql::Error),
  LiquidError(liquid::Error),
  DbErr(sea_orm::DbErr),
  ExpectedEntityNotFound(String),
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
