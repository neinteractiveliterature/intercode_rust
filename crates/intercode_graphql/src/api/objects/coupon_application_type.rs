use async_graphql::*;
use intercode_entities::coupon_applications;
use seawater::loaders::ExpectModels;

use crate::{model_backed_type, QueryData};

use super::{CouponType, ModelBackedType};
model_backed_type!(CouponApplicationType, coupon_applications::Model);

#[Object(name = "CouponApplication")]
impl CouponApplicationType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn coupon(&self, ctx: &Context<'_>) -> Result<CouponType> {
    let loader_result = ctx
      .data::<QueryData>()?
      .loaders()
      .coupon_application_coupon()
      .load_one(self.model.id)
      .await?;

    Ok(CouponType::new(loader_result.expect_one()?.clone()))
  }
}
