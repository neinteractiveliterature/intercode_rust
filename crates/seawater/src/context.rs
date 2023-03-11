use crate::{ConnectionWrapper, DropStore};
use std::{
  fmt::{Debug, Display},
  hash::Hash,
};

pub trait Context: Send + Sync + Clone + Debug {
  type StoreID: Eq + Hash + Copy + Send + Sync + Display + Debug + 'static;

  fn db(&self) -> &ConnectionWrapper;
  fn with_drop_store<'store, R, F: FnOnce(&'store DropStore<Self::StoreID>) -> R>(&self, f: F)
    -> R;
}
