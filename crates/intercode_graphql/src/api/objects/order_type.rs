use std::sync::Arc;

use async_graphql::*;
use intercode_entities::orders;
use intercode_graphql_core::{
  load_one_by_model_id, loader_result_to_many, model_backed_type, objects::MoneyType,
  scalars::DateScalar, ModelBackedType,
};
use intercode_graphql_loaders::LoaderManager;
use intercode_store::objects::CouponApplicationType;
use rusty_money::{iso, Money};
use seawater::loaders::{ExpectModel, ExpectModels};

use super::{OrderEntryType, UserConProfileType};
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
    let loader_result = load_one_by_model_id!(order_coupon_applications, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, CouponApplicationType))
  }

  #[graphql(name = "order_entries")]
  async fn order_entries(&self, ctx: &Context<'_>) -> Result<Vec<OrderEntryType>, Error> {
    let loader_result = load_one_by_model_id!(order_order_entries, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, OrderEntryType))
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
  async fn submitted_at(&self) -> Result<Option<DateScalar>> {
    self
      .model
      .submitted_at
      .map(DateScalar::try_from)
      .transpose()
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
      .data::<Arc<LoaderManager>>()?
      .order_user_con_profile()
      .load_one(self.model.id)
      .await?;

    Ok(UserConProfileType::new(loader_result.expect_one()?.clone()))
  }
}
