use async_graphql::{Context, Object, Result};
use intercode_entities::coupons;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_store::partial_objects::CouponStoreFields;

use crate::merged_model_backed_type;

use super::ConventionType;

model_backed_type!(CouponGlueFields, coupons::Model);

#[Object]
impl CouponGlueFields {
  pub async fn convention(&self, ctx: &Context<'_>) -> Result<ConventionType> {
    CouponStoreFields::from_type(self.clone())
      .convention(ctx)
      .await
      .map(ConventionType::new)
  }
}

merged_model_backed_type!(
  CouponType,
  coupons::Model,
  "Coupon",
  CouponStoreFields,
  CouponGlueFields
);
