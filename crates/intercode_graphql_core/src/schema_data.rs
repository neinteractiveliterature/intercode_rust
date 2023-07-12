use i18n_embed::fluent::FluentLanguageLoader;
use std::{fmt::Debug, sync::Arc};

#[derive(Clone)]
pub struct SchemaData {
  pub stripe_client: stripe::Client,
  pub language_loader: Arc<FluentLanguageLoader>,
}

impl Debug for SchemaData {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SchemaData")
      .field("language_loader", &self.language_loader)
      .finish_non_exhaustive()
  }
}
