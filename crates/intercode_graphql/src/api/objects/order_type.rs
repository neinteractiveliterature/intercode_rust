use async_graphql::*;
use chrono::NaiveDateTime;
use intercode_entities::orders;
use rusty_money::{iso, Money};
use seawater::loaders::ExpectModels;

use crate::{load_one_by_model_id, model_backed_type, QueryData};

use super::{
  money_type::MoneyType, CouponApplicationType, ModelBackedType, OrderEntryType, UserConProfileType,
};
model_backed_type!(OrderType, orders::Model);

#[Object(name = "Order")]
impl OrderType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "charge_id")]
  async fn charge_id(&self) -> Option<&str> {
    self.model.charge_id.as_deref()
  }

  #[graphql(name = "coupon_applications")]
  async fn coupon_applications(&self, ctx: &Context<'_>) -> Result<Vec<CouponApplicationType>> {
    let loader_result = ctx
      .data::<QueryData>()?
      .loaders()
      .order_coupon_applications()
      .load_one(self.model.id)
      .await?;

    Ok(
      loader_result
        .expect_models()?
        .iter()
        .cloned()
        .map(CouponApplicationType::new)
        .collect(),
    )
  }

  #[graphql(name = "order_entries")]
  async fn order_entries(&self, ctx: &Context<'_>) -> Result<Vec<OrderEntryType>, Error> {
    let loader = &ctx.data::<QueryData>()?.loaders().order_order_entries();

    Ok(
      loader
        .load_one(self.model.id)
        .await?
        .expect_models()?
        .iter()
        .map(|order_entry| OrderEntryType::new(order_entry.to_owned()))
        .collect(),
    )
  }

  #[graphql(name = "payment_amount")]
  async fn payment_amount(&self) -> Option<MoneyType> {
    MoneyType::from_cents_and_currency(
      self.model.payment_amount_cents,
      self.model.payment_amount_currency.as_deref(),
    )
  }

  #[graphql(name = "payment_note")]
  async fn payment_note(&self) -> Option<&str> {
    self.model.payment_note.as_deref()
  }

  async fn status(&self) -> &str {
    &self.model.status
  }

  #[graphql(name = "submitted_at")]
  async fn submitted_at(&self) -> Option<&NaiveDateTime> {
    self.model.submitted_at.as_ref()
  }

  #[graphql(name = "total_price")]
  async fn total_price(&self, ctx: &Context<'_>) -> Result<MoneyType> {
    let loader_result = load_one_by_model_id!(order_order_entries, ctx, self)?;

    let total = loader_result
      .expect_models()?
      .iter()
      .fold(Money::from_minor(0, iso::USD), |acc, order_entry| {
        acc + order_entry.total_price()
      });

    Ok(MoneyType::new(total))
  }

  #[graphql(name = "user_con_profile")]
  async fn user_con_profile(&self, ctx: &Context<'_>) -> Result<UserConProfileType> {
    let loader_result = ctx
      .data::<QueryData>()?
      .loaders()
      .order_user_con_profile()
      .load_one(self.model.id)
      .await?;

    Ok(UserConProfileType::new(loader_result.expect_one()?.clone()))
  }
}
