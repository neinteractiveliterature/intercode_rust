use std::{fmt::Debug, sync::Weak};

use i18n_embed::fluent::FluentLanguageLoader;
use intercode_graphql::{QueryData, SchemaData};
use seawater::{ConnectionWrapper, DropStore};

#[derive(Clone)]
pub struct DropContext {
  schema_data: SchemaData,
  query_data: QueryData,
  cache: Weak<DropStore<i64>>,
}

impl DropContext {
  pub fn new(schema_data: SchemaData, query_data: QueryData, cache: Weak<DropStore<i64>>) -> Self {
    DropContext {
      schema_data,
      query_data,
      cache,
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
  type StoreID = i64;

  fn with_drop_store<'store, R: 'store, F: FnOnce(&DropStore<i64>) -> R>(&self, f: F) -> R {
    let arc = self.cache.upgrade().unwrap();
    f(arc.as_ref())
  }

  fn db(&self) -> &ConnectionWrapper {
    self.query_data.db()
  }
}
