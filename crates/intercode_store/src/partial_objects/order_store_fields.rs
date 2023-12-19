use async_graphql::*;
use async_trait::async_trait;
use intercode_entities::{coupon_applications, order_entries, orders, user_con_profiles};
use intercode_graphql_core::{
  enums::OrderStatus, load_one_by_model_id, loader_result_to_many,
  loader_result_to_required_single, model_backed_type, objects::MoneyType, scalars::DateScalar,
  ModelBackedType,
};
use rusty_money::{iso, Money};
use seawater::loaders::ExpectModels;

#[async_trait]
pub trait OrderStoreExtensions
where
  Self: ModelBackedType<Model = orders::Model>,
{
  async fn coupon_applications<T: ModelBackedType<Model = coupon_applications::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>> {
    let loader_result = load_one_by_model_id!(order_coupon_applications, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, T))
  }

  async fn order_entries<T: ModelBackedType<Model = order_entries::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<Vec<T>, Error> {
    let loader_result = load_one_by_model_id!(order_order_entries, ctx, self)?;
    Ok(loader_result_to_many!(loader_result, T))
  }

  async fn user_con_profile<T: ModelBackedType<Model = user_con_profiles::Model>>(
    &self,
    ctx: &Context<'_>,
  ) -> Result<T> {
    let loader_result = load_one_by_model_id!(order_user_con_profile, ctx, self)?;
    Ok(loader_result_to_required_single!(loader_result, T))
  }
}

model_backed_type!(OrderStoreFields, orders::Model);

#[Object]
impl OrderStoreFields {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "charge_id")]
  async fn charge_id(&self) -> Option<&str> {
    self.model.charge_id.as_deref()
  }

  #[graphql(name = "paid_at")]
  async fn paid_at(&self) -> Result<Option<DateScalar>> {
    self.model.paid_at.map(DateScalar::try_from).transpose()
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

  async fn status(&self) -> Result<OrderStatus> {
    OrderStatus::try_from(self.model.status.as_str()).map_err(Error::from)
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
}
