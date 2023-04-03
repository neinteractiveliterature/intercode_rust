use async_graphql::*;
use intercode_entities::coupons;

use crate::model_backed_type;

use super::ModelBackedType;
model_backed_type!(CouponType, coupons::Model);

#[Object(name = "Coupon")]
impl CouponType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn code(&self) -> &str {
    &self.model.code
  }
}
