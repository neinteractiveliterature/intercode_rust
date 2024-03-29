use std::{
  fmt::Debug,
  fmt::Display,
  sync::{Arc, PoisonError},
};

#[derive(Debug, Clone)]
pub enum DropError {
  GraphQLError(async_graphql::Error),
  LiquidError(liquid::Error),
  DbErr(Arc<sea_orm::DbErr>),
  ExpectedEntityNotFound(String),
  PoisonError(String),
}

impl DropError {
  pub fn message(&self) -> String {
    match self {
      DropError::GraphQLError(err) => err.message.clone(),
      DropError::LiquidError(err) => err.to_string(),
      DropError::DbErr(err) => err.to_string(),
      DropError::ExpectedEntityNotFound(err) => err.to_string(),
      DropError::PoisonError(err) => err.to_string(),
    }
  }
}

impl Display for DropError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::GraphQLError(err) => f.write_fmt(format_args!("GraphQLError({})", err.message)),
      Self::LiquidError(err) => f.write_fmt(format_args!("LiquidError({})", err)),
      Self::DbErr(err) => f.write_fmt(format_args!("DbErr({})", err)),
      Self::ExpectedEntityNotFound(err) => {
        f.write_fmt(format_args!("ExpectedEntityNotFound({})", err))
      }
      Self::PoisonError(err) => f.write_fmt(format_args!("PoisonError({})", err)),
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
    DropError::DbErr(Arc::new(err))
  }
}

impl From<Arc<sea_orm::DbErr>> for DropError {
  fn from(err: Arc<sea_orm::DbErr>) -> Self {
    DropError::DbErr(err)
  }
}

impl<Guard> From<PoisonError<Guard>> for DropError {
  fn from(err: PoisonError<Guard>) -> Self {
    DropError::PoisonError(err.to_string())
  }
}
