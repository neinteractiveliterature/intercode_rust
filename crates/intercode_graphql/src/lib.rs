use api::{MutationRoot, QueryRoot};
use async_graphql::{EmptySubscription, Schema};
use intercode_graphql_core::schema_data::SchemaData;

pub mod actions;
pub mod api;

pub fn build_intercode_graphql_schema(
  schema_data: SchemaData,
) -> Schema<QueryRoot, MutationRoot, EmptySubscription> {
  async_graphql::Schema::build(
    api::QueryRoot::default(),
    api::MutationRoot,
    EmptySubscription,
  )
  .extension(async_graphql::extensions::Tracing)
  .data(schema_data)
  .finish()
}
