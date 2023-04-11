use rusty_money::{
  iso::{self, Currency},
  Money,
};

use crate::order_entries;

pub fn money_from_cents_and_currency<CentsType: Into<i64>>(
  cents: Option<CentsType>,
  currency: Option<&str>,
) -> Option<Money<'static, Currency>> {
  if let (Some(cents), Some(currency)) = (cents, currency) {
    iso::find(currency).map(|currency| Money::from_minor(cents.into(), currency))
  } else {
    None
  }
}

impl order_entries::Model {
  pub fn price_per_item(&self) -> Money<'static, Currency> {
    money_from_cents_and_currency(
      self.price_per_item_cents,
      self.price_per_item_currency.as_deref(),
    )
    .unwrap_or(Money::from_minor(0, iso::USD))
  }

  pub fn total_price(&self) -> Money<'static, Currency> {
    self.price_per_item() * self.quantity.unwrap_or(1)
  }
}
