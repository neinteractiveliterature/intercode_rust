use async_graphql::*;
use intercode_entities::coupons;
use seawater::loaders::ExpectModels;

use crate::{model_backed_type, QueryData};

use super::{ModelBackedType, MoneyType, ProductType};
model_backed_type!(CouponType, coupons::Model);

#[Object(name = "Coupon")]
impl CouponType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn code(&self) -> &str {
    &self.model.code
  }

  #[graphql(name = "fixed_amount")]
  async fn fixed_amount(&self) -> Option<MoneyType> {
    MoneyType::from_cents_and_currency(
      self.model.fixed_amount_cents,
      self.model.fixed_amount_currency.as_deref(),
    )
  }

  #[graphql(name = "percent_discount")]
  async fn percent_discount(&self) -> Result<Option<f64>> {
    Ok(
      self
        .model
        .percent_discount
        .map(|decimal| decimal.try_into())
        .transpose()?,
    )
  }

  #[graphql(name = "provides_product")]
  async fn provides_product(&self, ctx: &Context<'_>) -> Result<Option<ProductType>> {
    let loader_result = ctx
      .data::<QueryData>()?
      .loaders()
      .coupon_provides_product()
      .load_one(self.model.id)
      .await?;

    Ok(loader_result.try_one().cloned().map(ProductType::new))
  }
}
