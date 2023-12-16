use async_graphql::Object;
use chrono::TimeZone;
use intercode_timespan::TimespanWithValue;
use rusty_money::{iso::Currency, Money};

use crate::scalars::DateScalar;

use super::MoneyType;

pub struct TimespanWithStringableValueType<
  StartTz: TimeZone,
  FinishTz: TimeZone,
  T: Clone + Into<String> + Send + Sync,
> where
  StartTz::Offset: Send + Sync,
  FinishTz::Offset: Send + Sync,
{
  timespan: TimespanWithValue<StartTz, FinishTz, Option<T>>,
}

impl<StartTz: TimeZone, FinishTz: TimeZone, T: Clone + Into<String> + Send + Sync>
  TimespanWithStringableValueType<StartTz, FinishTz, T>
where
  StartTz::Offset: Send + Sync,
  FinishTz::Offset: Send + Sync,
{
  pub fn new(timespan: TimespanWithValue<StartTz, FinishTz, Option<T>>) -> Self {
    Self { timespan }
  }
}

#[Object(name = "TimespanWithValue")]
impl<StartTz: TimeZone, FinishTz: TimeZone, T: Clone + Into<String> + Send + Sync>
  TimespanWithStringableValueType<StartTz, FinishTz, T>
where
  StartTz::Offset: Send + Sync,
  FinishTz::Offset: Send + Sync,
  DateScalar<FinishTz>: async_graphql::OutputType,
  DateScalar<StartTz>: async_graphql::OutputType,
{
  async fn finish(&self) -> Option<DateScalar<FinishTz>> {
    self.timespan.timespan.finish.clone().map(DateScalar)
  }

  async fn start(&self) -> Option<DateScalar<StartTz>> {
    self.timespan.timespan.start.clone().map(DateScalar)
  }

  async fn value(&self) -> String {
    self
      .timespan
      .value
      .clone()
      .map(|v| v.into())
      .unwrap_or_default()
  }
}

pub struct TimespanWithMoneyValueType<StartTz: TimeZone, FinishTz: TimeZone>
where
  StartTz::Offset: Send + Sync,
  FinishTz::Offset: Send + Sync,
{
  timespan: TimespanWithValue<StartTz, FinishTz, Option<Money<'static, Currency>>>,
}

impl<StartTz: TimeZone, FinishTz: TimeZone> TimespanWithMoneyValueType<StartTz, FinishTz>
where
  StartTz::Offset: Send + Sync,
  FinishTz::Offset: Send + Sync,
{
  pub fn new(
    timespan: TimespanWithValue<StartTz, FinishTz, Option<Money<'static, Currency>>>,
  ) -> Self {
    Self { timespan }
  }
}

#[Object(name = "TimespanWithMoneyValue")]
impl<StartTz: TimeZone, FinishTz: TimeZone> TimespanWithMoneyValueType<StartTz, FinishTz>
where
  StartTz::Offset: Send + Sync,
  FinishTz::Offset: Send + Sync,
  DateScalar<FinishTz>: async_graphql::OutputType,
  DateScalar<StartTz>: async_graphql::OutputType,
{
  async fn finish(&self) -> Option<DateScalar<FinishTz>> {
    self.timespan.timespan.finish.clone().map(DateScalar)
  }

  async fn start(&self) -> Option<DateScalar<StartTz>> {
    self.timespan.timespan.start.clone().map(DateScalar)
  }

  async fn value(&self) -> MoneyType<'static> {
    self
      .timespan
      .value
      .clone()
      .map(MoneyType::new)
      .unwrap_or_default()
  }
}
