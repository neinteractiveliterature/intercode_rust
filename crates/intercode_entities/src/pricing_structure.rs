use std::fmt::Display;

use chrono::{DateTime, TimeZone, Utc};
use intercode_timespan::{ScheduledValue, TimespanWithValue};
use rusty_money::{
  iso::{self, Currency},
  FormattableCurrency, Money,
};
use sea_orm::prelude::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug)]
pub struct UnknownCurrencyCode(String);

impl Display for UnknownCurrencyCode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("UnknownCurrencyCode({})", self.0))
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializedMoney {
  pub fractional: i64,
  pub currency_code: String,
}

impl From<Money<'static, Currency>> for SerializedMoney {
  fn from(value: Money<'static, Currency>) -> Self {
    let fractional = (value.amount() * (Decimal::new(10_i64.pow(value.currency().exponent()), 0)))
      .try_into()
      .unwrap();

    SerializedMoney {
      fractional,
      currency_code: value.currency().code().to_owned(),
    }
  }
}

impl TryFrom<SerializedMoney> for Money<'static, Currency> {
  type Error = UnknownCurrencyCode;

  fn try_from(value: SerializedMoney) -> Result<Self, Self::Error> {
    let currency =
      iso::find(&value.currency_code).ok_or(UnknownCurrencyCode(value.currency_code))?;
    Ok(Money::from_minor(value.fractional, currency))
  }
}

#[derive(Serialize, Deserialize)]
struct SerializedPayWhatYouWantValue {
  pub minimum_amount: Option<SerializedMoney>,
  pub suggested_amount: Option<SerializedMoney>,
  pub maximum_amount: Option<SerializedMoney>,
}

impl From<PayWhatYouWantValue> for SerializedPayWhatYouWantValue {
  fn from(value: PayWhatYouWantValue) -> Self {
    Self {
      minimum_amount: value.minimum_amount.map(SerializedMoney::from),
      suggested_amount: value.suggested_amount.map(SerializedMoney::from),
      maximum_amount: value.maximum_amount.map(SerializedMoney::from),
    }
  }
}

impl TryFrom<SerializedPayWhatYouWantValue> for PayWhatYouWantValue {
  type Error = UnknownCurrencyCode;

  fn try_from(value: SerializedPayWhatYouWantValue) -> Result<Self, Self::Error> {
    Ok(Self {
      minimum_amount: value.minimum_amount.map(|v| v.try_into()).transpose()?,
      suggested_amount: value.suggested_amount.map(|v| v.try_into()).transpose()?,
      maximum_amount: value.maximum_amount.map(|v| v.try_into()).transpose()?,
    })
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(
  try_from = "SerializedPayWhatYouWantValue",
  into = "SerializedPayWhatYouWantValue"
)]
pub struct PayWhatYouWantValue {
  pub minimum_amount: Option<Money<'static, Currency>>,
  pub suggested_amount: Option<Money<'static, Currency>>,
  pub maximum_amount: Option<Money<'static, Currency>>,
}

fn serialize_money<S: Serializer>(
  value: &Money<'static, Currency>,
  serializer: S,
) -> Result<S::Ok, S::Error> {
  let serialized_money: SerializedMoney = value.clone().into();
  SerializedMoney::serialize(&serialized_money, serializer)
}

fn deserialize_money<'de, D>(deserializer: D) -> Result<Money<'static, Currency>, D::Error>
where
  D: Deserializer<'de>,
{
  let serialized_money = SerializedMoney::deserialize(deserializer)?;
  Ok(serialized_money.try_into().unwrap())
}

fn serialize_scheduled_money_value<S: Serializer>(
  value: &ScheduledValue<Utc, Option<Money<'static, Currency>>>,
  ser: S,
) -> Result<S::Ok, S::Error> {
  let scheduled_serialized_value = value
    .into_iter()
    .map(|twv| TimespanWithValue {
      timespan: twv.timespan,
      value: twv.value.flatten().map(SerializedMoney::from),
    })
    .collect::<ScheduledValue<_, _>>();

  Serialize::serialize(&scheduled_serialized_value, ser)
}

fn deserialize_scheduled_money_value<'de, D>(
  deserializer: D,
) -> Result<ScheduledValue<Utc, Option<Money<'static, Currency>>>, D::Error>
where
  D: Deserializer<'de>,
{
  let value_with_serialized_money =
    ScheduledValue::<Utc, Option<SerializedMoney>>::deserialize(deserializer)?;

  value_with_serialized_money
    .into_iter()
    .map(|twv| {
      Ok(TimespanWithValue {
        timespan: twv.timespan,
        value: twv
          .value
          .flatten()
          .map(Money::try_from)
          .transpose()
          .unwrap(),
      })
    })
    .collect::<Result<ScheduledValue<_, _>, _>>()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "pricing_strategy", content = "value", rename_all = "snake_case")]
pub enum PricingStructure {
  #[serde(
    serialize_with = "serialize_money",
    deserialize_with = "deserialize_money"
  )]
  Fixed(Money<'static, Currency>),
  #[serde(
    serialize_with = "serialize_scheduled_money_value",
    deserialize_with = "deserialize_scheduled_money_value",
    alias = "scheduled_value"
  )]
  Scheduled(ScheduledValue<Utc, Option<Money<'static, Currency>>>),
  PayWhatYouWant(PayWhatYouWantValue),
}

impl PricingStructure {
  pub fn price<Tz: TimeZone>(&self, time: DateTime<Tz>) -> Option<Money<'static, Currency>> {
    match self {
      PricingStructure::Fixed(value) => Some(value.to_owned()),
      PricingStructure::Scheduled(scheduled_value) => scheduled_value.value_at(time).flatten(),
      PricingStructure::PayWhatYouWant(value) => Some(
        value
          .suggested_amount
          .as_ref()
          .or(value.minimum_amount.as_ref())
          .cloned()
          .unwrap_or_else(|| Money::from_minor(0, iso::USD)),
      ),
    }
  }
}
