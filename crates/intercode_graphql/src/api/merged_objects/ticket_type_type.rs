use async_graphql::*;
use intercode_entities::ticket_types;
use intercode_graphql_core::model_backed_type;
use intercode_store::partial_objects::{TicketTypeStoreExtensions, TicketTypeStoreFields};

use crate::merged_model_backed_type;

use super::{product_type::ProductType, ConventionType, EventType};

model_backed_type!(TicketTypeGlueFields, ticket_types::Model);

impl TicketTypeStoreExtensions for TicketTypeGlueFields {}

#[Object]
impl TicketTypeGlueFields {
  pub async fn convention(&self, ctx: &Context<'_>) -> Result<Option<ConventionType>, Error> {
    TicketTypeStoreExtensions::convention(self, ctx).await
  }

  pub async fn event(&self, ctx: &Context<'_>) -> Result<Option<EventType>, Error> {
    TicketTypeStoreExtensions::event(self, ctx).await
  }

  #[graphql(name = "providing_products")]
  pub async fn providing_products(&self, ctx: &Context<'_>) -> Result<Vec<ProductType>, Error> {
    TicketTypeStoreExtensions::providing_products(self, ctx).await
  }
}

merged_model_backed_type!(
  TicketTypeType,
  ticket_types::Model,
  "TicketType",
  TicketTypeGlueFields,
  TicketTypeStoreFields
);
