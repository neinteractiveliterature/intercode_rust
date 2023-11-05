use std::fmt::Debug;

use async_graphql::{Object, OutputType};
use chrono::TimeZone;
use intercode_timespan::{ScheduledValue, TimespanWithValue};
use rusty_money::{iso::Currency, Money};

use crate::scalars::DateScalar;

use super::timespan_with_value_type::{
  TimespanWithMoneyValueType, TimespanWithStringableValueType,
};

pub struct ScheduledStringableValueType<
  Tz: TimeZone + Debug,
  V: Clone + Default + Debug + Into<String>,
> {
  scheduled_value: ScheduledValue<Tz, V>,
}

impl<Tz: TimeZone + Debug, V: Clone + Default + Debug + Into<String>>
  ScheduledStringableValueType<Tz, V>
{
  pub fn new(scheduled_value: ScheduledValue<Tz, V>) -> Self {
    Self { scheduled_value }
  }
}

#[Object(name = "ScheduledValue")]
impl<Tz: TimeZone + Debug + Send + Sync, V: Clone + Default + Debug + Send + Sync + Into<String>>
  ScheduledStringableValueType<Tz, V>
where
  Tz::Offset: Send + Sync,
  DateScalar<Tz>: OutputType,
{
  async fn timespans(&self) -> Vec<TimespanWithStringableValueType<Tz, Tz, V>> {
    self
      .scheduled_value
      .clone()
      .into_iter()
      .map(TimespanWithStringableValueType::new)
      .collect()
  }
}

pub struct ScheduledMoneyValueType<Tz: TimeZone + Debug> {
  scheduled_value: ScheduledValue<Tz, Option<Money<'static, Currency>>>,
}

impl<Tz: TimeZone + Debug> ScheduledMoneyValueType<Tz> {
  pub fn new(scheduled_value: ScheduledValue<Tz, Option<Money<'static, Currency>>>) -> Self {
    Self { scheduled_value }
  }
}

#[Object(name = "ScheduledMoneyValue")]
impl<Tz: TimeZone + Debug + Send + Sync> ScheduledMoneyValueType<Tz>
where
  Tz::Offset: Send + Sync,
  DateScalar<Tz>: OutputType,
{
  async fn timespans(&self) -> Vec<TimespanWithMoneyValueType<Tz, Tz>> {
    self
      .scheduled_value
      .clone()
      .into_iter()
      .map(|twv| {
        TimespanWithMoneyValueType::new(TimespanWithValue {
          timespan: twv.timespan,
          value: twv.value.flatten(),
        })
      })
      .collect()
  }
}
