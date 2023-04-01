use async_graphql::*;
use intercode_entities::order_entries;

use crate::model_backed_type;

use super::money_type::MoneyType;
model_backed_type!(OrderEntryType, order_entries::Model);

#[Object(name = "OrderEntry")]
impl OrderEntryType {
  async fn id(&self) -> ID {
    self.model.id.into()
  }

  #[graphql(name = "price_per_item")]
  async fn price_per_item(&self) -> Option<MoneyType<'_>> {
    MoneyType::from_cents_and_currency(
      self.model.price_per_item_cents.map(|cents| cents.into()),
      self.model.price_per_item_currency.as_deref(),
    )
  }

  async fn quantity(&self) -> Option<i32> {
    self.model.quantity
  }
}
