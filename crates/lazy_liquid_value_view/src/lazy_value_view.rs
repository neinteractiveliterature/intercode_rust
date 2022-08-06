use std::fmt::Debug;

use async_trait::async_trait;
use liquid::ValueView;
use serde::Serialize;
use tokio::runtime::Handle;

#[async_trait]
pub trait LazyValueView {
  type Value: Serialize;
  type Error: From<liquid::Error>;

  async fn resolve(&self) -> Result<&Self::Value, Self::Error>;
  fn get_resolved(&self) -> Option<&Self::Value>;

  fn resolve_sync(&self) -> Result<&Self::Value, Self::Error> {
    tokio::task::block_in_place(|| Handle::current().block_on(async move { self.resolve().await }))
  }

  fn as_value_sync(&self) -> Result<liquid::model::Value, Self::Error> {
    Ok(liquid::model::to_value(self.resolve_sync()?)?)
  }
}

impl<V: Serialize, E: From<liquid::Error>> Debug for dyn LazyValueView<Value = V, Error = E> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let liquid_value =
      liquid::model::to_value(&self.get_resolved()).map_err(|_| std::fmt::Error)?;

    f.debug_tuple("LazyValueView")
      .field(liquid_value.as_debug())
      .finish()
  }
}
