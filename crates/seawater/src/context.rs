use crate::{ConnectionWrapper, DropStore};
use std::{
  fmt::{Debug, Display},
  hash::Hash,
};

pub trait Context: Send + Sync + Clone + Debug {
  fn db(&self) -> &ConnectionWrapper;
  fn with_drop_store<
    'store,
    R,
    ID: Eq + Hash + Copy + Display + Debug + 'store,
    F: FnOnce(&'store DropStore<ID>) -> R,
  >(
    &self,
    f: F,
  ) -> R;
}
