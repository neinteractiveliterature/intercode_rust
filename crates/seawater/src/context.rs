use crate::{ConnectionWrapper, DropStore};
use std::fmt::Debug;

pub trait Context: Send + Sync + Clone + Debug {
  fn db(&self) -> &ConnectionWrapper;
  fn with_drop_store<R, F: FnOnce(&DropStore<i64>) -> R>(&self, f: F) -> R;
}

pub trait ContextContainer {
  type Context: crate::Context;
}
