use crate::{Context, DropResult};
use liquid::{ObjectView, ValueView};
use std::{
  fmt::{Debug, Display},
  hash::Hash,
};

pub trait LiquidDrop: ValueView + ObjectView + Clone + Into<DropResult<Self>> {
  type Cache: LiquidDropCache;
  type ID: Eq + Hash + Copy + Display + Send + Sync + Debug;
  type Context: Context;

  fn id(&self) -> Self::ID;
  fn get_context(&self) -> Self::Context;
}

pub trait LiquidDropCache: Send + Sync {
  fn new() -> Self;
}
