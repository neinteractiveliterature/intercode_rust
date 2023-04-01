use async_graphql::{Object, Result};
use rusty_money::{
  iso::{self, Currency},
  FormattableCurrency, Money,
};
use sea_orm::prelude::Decimal;

pub struct MoneyType<'currency> {
  money: Money<'currency, Currency>,
}

impl<'currency> MoneyType<'currency> {
  pub fn new(money: Money<'currency, Currency>) -> Self {
    Self { money }
  }

  pub fn from_cents_and_currency(
    cents: Option<i64>,
    currency: Option<&str>,
  ) -> Option<MoneyType<'currency>> {
    if let (Some(cents), Some(currency)) = (cents, currency) {
      iso::find(currency).map(|currency| MoneyType::new(Money::from_minor(cents, currency)))
    } else {
      None
    }
  }
}

#[Object(name = "Money")]
impl<'currency> MoneyType<'currency> {
  #[graphql(name = "currency_code")]
  pub async fn currency_code(&self) -> &str {
    self.money.currency().code()
  }

  pub async fn fractional(&self) -> Result<i64> {
    Ok(
      (self.money.amount() * (Decimal::new(10_i64.pow(self.money.currency().exponent()), 0)))
        .try_into()?,
    )
  }
}
