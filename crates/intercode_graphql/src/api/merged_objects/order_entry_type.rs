use async_graphql::*;
use intercode_entities::order_entries;
use intercode_graphql_core::model_backed_type;
use intercode_store::partial_objects::{OrderEntryStoreExtensions, OrderEntryStoreFields};

use crate::merged_model_backed_type;

use super::{product_type::ProductType, product_variant_type::ProductVariantType, OrderType};

model_backed_type!(OrderEntryGlueFields, order_entries::Model);

impl OrderEntryStoreExtensions for OrderEntryGlueFields {}

#[Object]
impl OrderEntryGlueFields {
  pub async fn order(&self, ctx: &Context<'_>) -> Result<OrderType> {
    OrderEntryStoreExtensions::order(self, ctx).await
  }

  pub async fn product(&self, ctx: &Context<'_>) -> Result<ProductType> {
    OrderEntryStoreExtensions::product(self, ctx).await
  }

  pub async fn product_variant(&self, ctx: &Context<'_>) -> Result<Option<ProductVariantType>> {
    OrderEntryStoreExtensions::product_variant(self, ctx).await
  }
}

merged_model_backed_type!(
  OrderEntryType,
  order_entries::Model,
  "OrderEntry",
  OrderEntryGlueFields,
  OrderEntryStoreFields
);
