use async_graphql::*;
use intercode_entities::orders;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_store::partial_objects::OrderStoreFields;

use crate::merged_model_backed_type;

use super::{OrderEntryType, UserConProfileType};

model_backed_type!(OrderGlueFields, orders::Model);

#[Object]
impl OrderGlueFields {
  #[graphql(name = "order_entries")]
  pub async fn order_entries(&self, ctx: &Context<'_>) -> Result<Vec<OrderEntryType>, Error> {
    OrderStoreFields::from_type(self.clone())
      .order_entries(ctx)
      .await
      .map(|items| items.into_iter().map(OrderEntryType::from_type).collect())
  }

  #[graphql(name = "user_con_profile")]
  pub async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType> {
    OrderStoreFields::from_type(self.clone())
      .user_con_profile(ctx)
      .await
      .map(UserConProfileType::from_type)
  }
}

merged_model_backed_type!(
  OrderType,
  orders::Model,
  "Order",
  OrderGlueFields,
  OrderStoreFields
);
