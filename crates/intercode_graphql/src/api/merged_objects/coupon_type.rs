use async_graphql::{Context, Object, Result};
use intercode_entities::coupons;
use intercode_graphql_core::model_backed_type;
use intercode_store::partial_objects::{CouponStoreExtensions, CouponStoreFields};

use crate::merged_model_backed_type;

use super::{product_type::ProductType, ConventionType};

model_backed_type!(CouponGlueFields, coupons::Model);

impl CouponStoreExtensions for CouponGlueFields {}

#[Object]
impl CouponGlueFields {
  pub async fn convention(&self, ctx: &Context<'_>) -> Result<ConventionType> {
    CouponStoreExtensions::convention(self, ctx).await
  }

  #[graphql(name = "provides_product")]
  pub async fn provides_product(&self, ctx: &Context<'_>) -> Result<Option<ProductType>> {
    CouponStoreExtensions::provides_product(self, ctx).await
  }
}

merged_model_backed_type!(
  CouponType,
  coupons::Model,
  "Coupon",
  CouponStoreFields,
  CouponGlueFields
);
