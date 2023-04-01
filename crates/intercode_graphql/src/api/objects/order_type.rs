use async_graphql::*;
use chrono::NaiveDateTime;
use intercode_entities::orders;
use seawater::loaders::ExpectModels;

use crate::{model_backed_type, QueryData};

use super::{money_type::MoneyType, ModelBackedType, OrderEntryType};
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
      self.model.payment_amount_cents.map(|cents| cents.into()),
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
}
