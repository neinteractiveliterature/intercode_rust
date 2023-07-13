use super::{MoneyType, ProductType};
use async_graphql::*;
use intercode_entities::coupons;
use intercode_graphql_core::{
  load_one_by_model_id, model_backed_type, scalars::DateScalar, ModelBackedType,
};
use seawater::loaders::ExpectModel;

model_backed_type!(CouponType, coupons::Model);

#[Object(name = "Coupon")]
impl CouponType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn code(&self) -> &str {
    &self.model.code
  }

  #[graphql(name = "expires_at")]
  async fn expires_at(&self) -> Result<Option<DateScalar>> {
    self.model.expires_at.map(DateScalar::try_from).transpose()
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
    let loader_result = load_one_by_model_id!(coupon_provides_product, ctx, self)?;
    Ok(loader_result.try_one().cloned().map(ProductType::new))
  }

  #[graphql(name = "usage_limit")]
  async fn usage_limit(&self) -> Option<i32> {
    self.model.usage_limit
  }
}
