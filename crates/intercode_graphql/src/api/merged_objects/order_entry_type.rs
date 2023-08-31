use async_graphql::*;
use intercode_entities::order_entries;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_store::partial_objects::OrderEntryStoreFields;

use crate::merged_model_backed_type;

use super::OrderType;

model_backed_type!(OrderEntryGlueFields, order_entries::Model);

#[Object]
impl OrderEntryGlueFields {
  pub async fn order(&self, ctx: &Context<'_>) -> Result<OrderType> {
    OrderEntryStoreFields::from_type(self.clone())
      .order(ctx)
      .await
      .map(OrderType::from_type)
  }
}

merged_model_backed_type!(
  OrderEntryType,
  order_entries::Model,
  "OrderEntry",
  OrderEntryGlueFields,
  OrderEntryStoreFields
);
