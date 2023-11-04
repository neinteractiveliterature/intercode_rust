use api::{interfaces::CmsParentInterface, MutationRoot, QueryRoot};
use async_graphql::{EmptySubscription, Schema, SchemaBuilder};
use intercode_graphql_core::schema_data::SchemaData;

pub mod actions;
pub mod api;

pub fn build_intercode_graphql_schema_minimal(
) -> SchemaBuilder<QueryRoot, MutationRoot, EmptySubscription> {
  async_graphql::Schema::build(
    api::QueryRoot::default(),
    api::MutationRoot,
    EmptySubscription,
  )
  .register_output_type::<CmsParentInterface>()
}

pub fn build_intercode_graphql_schema(
  schema_data: SchemaData,
) -> Schema<QueryRoot, MutationRoot, EmptySubscription> {
  build_intercode_graphql_schema_minimal()
    .extension(async_graphql::extensions::Tracing)
    .data(schema_data)
    .finish()
}
