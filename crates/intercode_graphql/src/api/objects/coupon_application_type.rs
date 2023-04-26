use async_graphql::*;
use intercode_entities::coupon_applications;
use rusty_money::{iso, Money};
use seawater::loaders::{ExpectModel, ExpectModels};

use crate::{load_one_by_id, load_one_by_model_id, model_backed_type, QueryData};

use super::{CouponType, ModelBackedType, MoneyType};
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

  async fn discount(&self, ctx: &Context<'_>) -> Result<Option<MoneyType>> {
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
    Ok(discount.discount_amount(total_price).map(MoneyType::new))
  }
}
