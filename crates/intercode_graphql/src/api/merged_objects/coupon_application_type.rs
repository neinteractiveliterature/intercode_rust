use async_graphql::{Context, Object, Result};
use intercode_entities::coupon_applications;
use intercode_graphql_core::model_backed_type;
use intercode_store::partial_objects::{
  CouponApplicationStoreExtensions, CouponApplicationStoreFields,
};

use crate::merged_model_backed_type;

use super::{coupon_type::CouponType, OrderType};

model_backed_type!(CouponApplicationGlueFields, coupon_applications::Model);

impl CouponApplicationStoreExtensions for CouponApplicationGlueFields {}

#[Object]
impl CouponApplicationGlueFields {
  pub async fn coupon(&self, ctx: &Context<'_>) -> Result<CouponType> {
    CouponApplicationStoreExtensions::coupon(self, ctx).await
  }

  pub async fn order(&self, ctx: &Context<'_>) -> Result<OrderType> {
    CouponApplicationStoreExtensions::order(self, ctx).await
  }
}

merged_model_backed_type!(
  CouponApplicationType,
  coupon_applications::Model,
  "CouponApplication",
  CouponApplicationStoreFields,
  CouponApplicationGlueFields
);
