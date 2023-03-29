use async_graphql::Object;
use chrono::{DateTime, TimeZone};
use intercode_timespan::TimespanWithValue;

pub struct TimespanWithValueType<
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
  TimespanWithValueType<StartTz, FinishTz, T>
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
  TimespanWithValueType<StartTz, FinishTz, T>
where
  StartTz::Offset: Send + Sync,
  FinishTz::Offset: Send + Sync,
  DateTime<FinishTz>: async_graphql::OutputType,
  DateTime<StartTz>: async_graphql::OutputType,
{
  async fn finish(&self) -> Option<&DateTime<FinishTz>> {
    self.timespan.timespan.finish.as_ref()
  }

  async fn start(&self) -> Option<&DateTime<StartTz>> {
    self.timespan.timespan.start.as_ref()
  }

  async fn value(&self) -> Option<String> {
    self.timespan.value.clone().map(|v| v.into())
  }
}
