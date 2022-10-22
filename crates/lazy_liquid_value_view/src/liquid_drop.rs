use crate::DropResult;
use liquid::{ObjectView, ValueView};
use std::hash::Hash;

pub trait LiquidDrop: ValueView + ObjectView + Into<DropResult<Self>> {
  type Cache;

  fn get_cache(&self) -> &Self::Cache;
}

pub trait LiquidDropWithID {
  type ID: Eq + Hash + Copy;

  fn id(&self) -> Self::ID;
}
