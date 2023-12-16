use async_graphql::*;
use intercode_entities::coupon_applications;
use intercode_graphql_core::{
  load_one_by_id, load_one_by_model_id, model_backed_type, objects::MoneyType, ModelBackedType,
};
use rusty_money::{
  iso::{self, USD},
  Money,
};
use seawater::loaders::{ExpectModel, ExpectModels};

use super::CouponStoreFields;

model_backed_type!(CouponApplicationStoreFields, coupon_applications::Model);

impl CouponApplicationStoreFields {
  pub async fn coupon(&self, ctx: &Context<'_>) -> Result<CouponStoreFields> {
    let loader_result = load_one_by_model_id!(coupon_application_coupon, ctx, self)?;
    Ok(CouponStoreFields::new(loader_result.expect_one()?.clone()))
  }
}

#[Object]
impl CouponApplicationStoreFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  async fn discount(&self, ctx: &Context<'_>) -> Result<MoneyType> {
    let coupon = load_one_by_model_id!(coupon_application_coupon, ctx, self)?;
    let coupon = coupon.expect_one()?;
    let discount = coupon.discount()?;
    let order = load_one_by_model_id!(coupon_application_order, ctx, self)?;
    let order = order.expect_one()?;
    let order_entries = load_one_by_id!(order_order_entries, ctx, order.id)?;
    let order_entries = order_entries.expect_models()?;
    let total_price = order_entries
      .iter()
      .fold(Money::from_minor(0, iso::USD), |acc, order_entry| {
        acc + order_entry.total_price()
      });
    Ok(MoneyType::new(
      discount
        .discount_amount(total_price)
        .unwrap_or_else(|| Money::from_minor(0, USD)),
    ))
  }
}
