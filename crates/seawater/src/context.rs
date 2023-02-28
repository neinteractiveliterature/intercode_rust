use crate::{ConnectionWrapper, NormalizedDropCache};
use std::fmt::Debug;

pub trait Context: Send + Sync + Clone + Debug {
  fn db(&self) -> &ConnectionWrapper;
  fn with_drop_cache<R, F: FnOnce(&NormalizedDropCache<i64>) -> R>(&self, f: F) -> R;
}

pub trait ContextContainer {
  type Context: crate::Context;
}
