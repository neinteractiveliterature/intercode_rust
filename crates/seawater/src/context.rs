use sea_orm::DatabaseConnection;
use std::fmt::Debug;

use crate::NormalizedDropCache;

pub trait Context: Send + Sync + Clone + Debug {
  fn db(&self) -> &DatabaseConnection;
  fn drop_cache(&self) -> &NormalizedDropCache<i64>;
}

pub trait ContextContainer {
  type Context: crate::Context;
}
