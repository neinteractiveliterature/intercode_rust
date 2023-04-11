use async_graphql::{Object, Result};
use intercode_entities::model_ext::orders::money_from_cents_and_currency;
use rusty_money::{iso::Currency, FormattableCurrency, Money};
use sea_orm::prelude::Decimal;

#[derive(Clone, Debug)]
pub struct MoneyType<'currency> {
  money: Money<'currency, Currency>,
}

impl<'currency> MoneyType<'currency> {
  pub fn new(money: Money<'currency, Currency>) -> Self {
    Self { money }
  }

  pub fn from_cents_and_currency<CentsType: Into<i64>>(
    cents: Option<CentsType>,
    currency: Option<&str>,
  ) -> Option<MoneyType<'currency>> {
    money_from_cents_and_currency(cents, currency).map(MoneyType::new)
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
