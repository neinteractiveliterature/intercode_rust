use async_graphql::*;
use intercode_entities::orders;
use intercode_graphql_core::model_backed_type;
use intercode_store::partial_objects::{OrderStoreExtensions, OrderStoreFields};

use crate::merged_model_backed_type;

use super::{coupon_application_type::CouponApplicationType, OrderEntryType, UserConProfileType};

model_backed_type!(OrderGlueFields, orders::Model);

impl OrderStoreExtensions for OrderGlueFields {}

#[Object]
impl OrderGlueFields {
  #[graphql(name = "coupon_applications")]
  pub async fn coupon_applications(&self, ctx: &Context<'_>) -> Result<Vec<CouponApplicationType>> {
    OrderStoreExtensions::coupon_applications(self, ctx).await
  }

  #[graphql(name = "order_entries")]
  pub async fn order_entries(&self, ctx: &Context<'_>) -> Result<Vec<OrderEntryType>, Error> {
    OrderStoreExtensions::order_entries(self, ctx).await
  }

  #[graphql(name = "user_con_profile")]
  pub async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType> {
    OrderStoreExtensions::user_con_profile(self, ctx).await
  }
}

merged_model_backed_type!(
  OrderType,
  orders::Model,
  "Order",
  OrderGlueFields,
  OrderStoreFields
);
