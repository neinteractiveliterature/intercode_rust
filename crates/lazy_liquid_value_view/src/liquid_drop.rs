use crate::{LiquidDropCache};
use liquid::{ObjectView, ValueView};
use std::{fmt::Display, hash::Hash};

// deleted: + Into<DropResult<Self>> - not sure why it's needed and it's making this non-object-safe
pub trait LiquidDrop: ValueView + ObjectView {
  type Cache: LiquidDropCache;

  fn get_cache(&self) -> &Self::Cache;
}

pub trait LiquidDropWithID {
  type ID: Eq + Hash + Copy + Display;

  fn id(&self) -> Self::ID;
}
