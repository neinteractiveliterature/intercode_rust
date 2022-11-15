use crate::{ConnectionWrapper, NormalizedDropCache};
use std::fmt::Debug;

pub trait Context: Send + Sync + Clone + Debug {
  fn db(&self) -> &ConnectionWrapper;
  fn drop_cache(&self) -> &NormalizedDropCache<i64>;
}

pub trait ContextContainer {
  type Context: crate::Context;
}
