use std::fmt::Debug;

use i18n_embed::fluent::FluentLanguageLoader;
use intercode_graphql::SchemaData;
use seawater::NormalizedDropCache;

#[derive(Clone)]
pub struct DropContext {
  schema_data: SchemaData,
  cache: NormalizedDropCache<i64>,
}

impl DropContext {
  pub fn new(schema_data: SchemaData) -> Self {
    DropContext {
      schema_data,
      cache: Default::default(),
    }
  }

  pub fn language_loader(&self) -> &FluentLanguageLoader {
    self.schema_data.language_loader.as_ref()
  }
}

impl Debug for DropContext {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DropContext")
      .field("schema_data", &self.schema_data)
      .finish_non_exhaustive()
  }
}

impl seawater::Context for DropContext {
  fn drop_cache(&self) -> &NormalizedDropCache<i64> {
    &self.cache
  }

  fn db(&self) -> &sea_orm::DatabaseConnection {
    self.schema_data.db.as_ref()
  }
}
