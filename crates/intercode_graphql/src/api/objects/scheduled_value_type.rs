use std::fmt::Debug;

use async_graphql::{Object, OutputType};
use chrono::{DateTime, TimeZone};
use intercode_timespan::ScheduledValue;

use super::TimespanWithValueType;

pub struct ScheduledValueType<Tz: TimeZone + Debug, V: Clone + Default + Debug> {
  scheduled_value: ScheduledValue<Tz, V>,
}

impl<Tz: TimeZone + Debug, V: Clone + Default + Debug> ScheduledValueType<Tz, V> {
  pub fn new(scheduled_value: ScheduledValue<Tz, V>) -> Self {
    Self { scheduled_value }
  }
}

#[Object(name = "ScheduledValue")]
impl<Tz: TimeZone + Debug + Send + Sync, V: Clone + Default + Debug + Send + Sync + Into<String>>
  ScheduledValueType<Tz, V>
where
  Tz::Offset: Send + Sync,
  DateTime<Tz>: OutputType,
{
  async fn timespans(&self) -> Vec<TimespanWithValueType<Tz, Tz, V>> {
    self
      .scheduled_value
      .clone()
      .into_iter()
      .map(TimespanWithValueType::new)
      .collect()
  }
}
