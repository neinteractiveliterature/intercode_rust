use async_graphql::*;
use intercode_entities::{conventions, coupons};
use intercode_graphql_core::{
  load_one_by_model_id, model_backed_type,
  objects::MoneyType,
  scalars::{BigDecimalScalar, DateScalar},
  ModelBackedType,
};
use seawater::loaders::ExpectModel;

use crate::objects::ProductType;

model_backed_type!(CouponStoreFields, coupons::Model);

impl CouponStoreFields {
  pub async fn convention(&self, ctx: &Context<'_>) -> Result<conventions::Model> {
    let result = load_one_by_model_id!(coupon_convention, ctx, self)?;
    result.expect_one().cloned()
  }
}

#[Object]
impl CouponStoreFields {
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
  async fn percent_discount(&self) -> Result<Option<BigDecimalScalar>> {
    Ok(
      self
        .model
        .percent_discount
        .map(BigDecimalScalar::try_from)
        .transpose()
        .map_err(|err| err.into_server_error(Pos::default()))?,
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
