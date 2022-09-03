use liquid::{ObjectView, ValueView};

use crate::DropResult;

pub trait LiquidDrop: ValueView + ObjectView + Into<DropResult<Self>> {
  type Cache;

  fn get_cache(&self) -> &Self::Cache;
}
