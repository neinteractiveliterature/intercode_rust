use async_graphql::{Context, Object, Result};
use intercode_entities::coupon_applications;
use intercode_graphql_core::{model_backed_type, ModelBackedType};
use intercode_store::partial_objects::CouponApplicationStoreFields;

use crate::merged_model_backed_type;

use super::coupon_type::CouponType;

model_backed_type!(CouponApplicationGlueFields, coupon_applications::Model);

#[Object]
impl CouponApplicationGlueFields {
  pub async fn coupon(&self, ctx: &Context<'_>) -> Result<CouponType> {
    CouponApplicationStoreFields::from_type(self.clone())
      .coupon(ctx)
      .await
      .map(CouponType::from_type)
  }
}

merged_model_backed_type!(
  CouponApplicationType,
  coupon_applications::Model,
  "CouponApplication",
  CouponApplicationStoreFields,
  CouponApplicationGlueFields
);
